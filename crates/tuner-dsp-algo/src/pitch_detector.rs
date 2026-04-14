use tuner_core::PitchDetectionResult;

use crate::interpolate::parabolic_interpolate;
use crate::nsdf::compute_nsdf;
use crate::peak_detection::{
    PeakCandidate, filter_peak_candidates, find_local_maxima, select_best_peak,
};
use crate::preprocess::preprocess_frame;
use crate::smoothing::DetectionSmoother;

const OCTAVE_AMBIGUITY_FREQ_MIN_HZ: f32 = 120.0;
const OCTAVE_AMBIGUITY_FREQ_MAX_HZ: f32 = 260.0;
const OCTAVE_AMBIGUITY_MAX_PEAK_CLARITY: f32 = 0.90;
const OCTAVE_AMBIGUITY_CLARITY_DELTA: f32 = 0.05;
const OCTAVE_AMBIGUITY_TAU_TOLERANCE_RATIO: f32 = 0.04;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchDetectorConfig {
    pub sample_rate: u32,
    pub frame_size: usize,
    pub hop_size: usize,
    pub min_frequency_hz: f32,
    pub max_frequency_hz: f32,
    pub min_rms: f32,
    pub min_clarity: f32,
    pub smoothing_window_size: usize,
    pub apply_hann_window: bool,
}

impl Default for PitchDetectorConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44_100,
            frame_size: 4096,
            hop_size: 1024,
            min_frequency_hz: 60.0,
            max_frequency_hz: 400.0,
            min_rms: 0.01,
            min_clarity: 0.60,
            smoothing_window_size: 3,
            apply_hann_window: true,
        }
    }
}

impl PitchDetectorConfig {
    pub fn tau_range(&self) -> (usize, usize) {
        let tau_min = (self.sample_rate as f32 / self.max_frequency_hz).floor() as usize;
        let tau_max = (self.sample_rate as f32 / self.min_frequency_hz).ceil() as usize;
        (tau_min.max(1), tau_max.max(1))
    }
}

#[derive(Debug, Clone)]
pub struct PitchDetector {
    config: PitchDetectorConfig,
    smoother: DetectionSmoother,
}

impl PitchDetector {
    pub fn new(config: PitchDetectorConfig) -> Self {
        Self {
            smoother: DetectionSmoother::new(config.smoothing_window_size),
            config,
        }
    }

    pub fn config(&self) -> &PitchDetectorConfig {
        &self.config
    }

    pub fn set_min_rms(&mut self, min_rms: f32) -> bool {
        if !min_rms.is_finite() || min_rms < 0.0 {
            return false;
        }

        self.config.min_rms = min_rms;
        true
    }

    pub fn set_min_clarity(&mut self, min_clarity: f32) -> bool {
        if !min_clarity.is_finite() || !(0.0..=1.0).contains(&min_clarity) {
            return false;
        }

        self.config.min_clarity = min_clarity;
        true
    }

    pub fn detect_pitch(&mut self, frame: &[f32]) -> Option<PitchDetectionResult> {
        if frame.len() < self.config.frame_size {
            return None;
        }

        let (tau_min, tau_max) = self.config.tau_range();
        if tau_max >= frame.len() {
            return None;
        }

        let (processed, metrics) = preprocess_frame(frame, self.config.apply_hann_window);
        if metrics.rms < self.config.min_rms {
            return None;
        }

        let nsdf = compute_nsdf(&processed, tau_min, tau_max);
        let peaks = find_local_maxima(&nsdf, tau_min);
        let filtered_peaks =
            filter_peak_candidates(&peaks, self.config.min_clarity, tau_min, tau_max);
        let peak = select_best_peak(&filtered_peaks)?;
        let peak = resolve_octave_ambiguity(
            peak,
            &filtered_peaks,
            self.config.sample_rate,
            tau_max,
            self.config.min_frequency_hz,
        );
        let peak_index = peak.tau.checked_sub(tau_min)?;

        let interpolation = parabolic_interpolate(&nsdf, peak_index, tau_min)?;
        if interpolation.refined_tau <= 0.0 {
            return None;
        }

        let raw_frequency_hz = self.config.sample_rate as f32 / interpolation.refined_tau;
        if raw_frequency_hz < self.config.min_frequency_hz
            || raw_frequency_hz > self.config.max_frequency_hz
        {
            return None;
        }

        let smoothed_frequency_hz = self.smoother.push(raw_frequency_hz);

        Some(PitchDetectionResult {
            frequency_hz: smoothed_frequency_hz,
            confidence: interpolation.refined_clarity.clamp(0.0, 1.0),
            clarity: interpolation.refined_clarity,
            rms: metrics.rms,
        })
    }
}

fn resolve_octave_ambiguity(
    selected_peak: PeakCandidate,
    peaks: &[PeakCandidate],
    sample_rate: u32,
    tau_max: usize,
    min_frequency_hz: f32,
) -> PeakCandidate {
    if selected_peak.clarity >= OCTAVE_AMBIGUITY_MAX_PEAK_CLARITY {
        return selected_peak;
    }

    let selected_frequency_hz = sample_rate as f32 / selected_peak.tau as f32;
    if !(OCTAVE_AMBIGUITY_FREQ_MIN_HZ..=OCTAVE_AMBIGUITY_FREQ_MAX_HZ)
        .contains(&selected_frequency_hz)
    {
        return selected_peak;
    }

    let Some(target_tau) = selected_peak.tau.checked_mul(2) else {
        return selected_peak;
    };
    if target_tau > tau_max {
        return selected_peak;
    }

    let tau_tolerance = (target_tau as f32 * OCTAVE_AMBIGUITY_TAU_TOLERANCE_RATIO).max(1.0);
    let Some(subharmonic_peak) = peaks
        .iter()
        .copied()
        .filter(|peak| peak.tau > selected_peak.tau)
        .filter(|peak| (peak.tau as f32 - target_tau as f32).abs() <= tau_tolerance)
        .max_by(|left, right| left.clarity.total_cmp(&right.clarity))
    else {
        return selected_peak;
    };

    let subharmonic_frequency_hz = sample_rate as f32 / subharmonic_peak.tau as f32;
    if subharmonic_frequency_hz < min_frequency_hz {
        return selected_peak;
    }

    if subharmonic_peak.clarity + OCTAVE_AMBIGUITY_CLARITY_DELTA >= selected_peak.clarity {
        return subharmonic_peak;
    }

    selected_peak
}

#[cfg(test)]
mod tests {
    use super::{PitchDetector, PitchDetectorConfig};
    use crate::nsdf::compute_nsdf;
    use crate::peak_detection::{filter_peak_candidates, find_local_maxima};
    use crate::preprocess::preprocess_frame;
    use std::f32::consts::PI;

    fn sine_wave(
        frequency_hz: f32,
        sample_rate: u32,
        frame_size: usize,
        amplitude: f32,
    ) -> Vec<f32> {
        (0..frame_size)
            .map(|index| {
                let t = index as f32 / sample_rate as f32;
                amplitude * (2.0 * PI * frequency_hz * t).sin()
            })
            .collect()
    }

    fn guitar_like_wave(
        frequency_hz: f32,
        sample_rate: u32,
        frame_size: usize,
        fundamental_amplitude: f32,
    ) -> Vec<f32> {
        (0..frame_size)
            .map(|index| {
                let t = index as f32 / sample_rate as f32;
                let envelope = (1.0 - index as f32 / frame_size as f32).max(0.2);
                let fundamental =
                    envelope * fundamental_amplitude * (2.0 * PI * frequency_hz * t).sin();
                let second_harmonic = envelope * 0.55 * (2.0 * PI * frequency_hz * 2.0 * t).sin();
                let third_harmonic = envelope * 0.30 * (2.0 * PI * frequency_hz * 3.0 * t).sin();
                let pick_transient = if index < 64 {
                    0.15 * (2.0 * PI * 1_800.0 * t).sin()
                } else {
                    0.0
                };
                let pseudo_noise =
                    0.015 * (2.0 * PI * 733.0 * t).sin() + 0.01 * (2.0 * PI * 1_177.0 * t).sin();

                fundamental + second_harmonic + third_harmonic + pick_transient + pseudo_noise
            })
            .collect()
    }

    fn octave_ambiguous_wave(
        frequency_hz: f32,
        sample_rate: u32,
        frame_size: usize,
        fundamental_amplitude: f32,
    ) -> Vec<f32> {
        (0..frame_size)
            .map(|index| {
                let t = index as f32 / sample_rate as f32;
                let envelope = (1.0 - index as f32 / frame_size as f32).max(0.25);
                let fundamental =
                    envelope * fundamental_amplitude * (2.0 * PI * frequency_hz * t).sin();
                let second_harmonic = envelope * 0.95 * (2.0 * PI * frequency_hz * 2.0 * t).sin();
                let third_harmonic = envelope * 0.35 * (2.0 * PI * frequency_hz * 3.0 * t).sin();
                let pick_transient = if index < 48 {
                    0.12 * (2.0 * PI * 1_450.0 * t).sin()
                } else {
                    0.0
                };
                let pseudo_noise =
                    0.012 * (2.0 * PI * 811.0 * t).sin() + 0.008 * (2.0 * PI * 1_307.0 * t).sin();

                fundamental + second_harmonic + third_harmonic + pick_transient + pseudo_noise
            })
            .collect()
    }

    fn noisy_frame(frame_size: usize, amplitude: f32) -> Vec<f32> {
        let mut state = 0x1234_5678_u32;

        (0..frame_size)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let normalized = (state as f32 / u32::MAX as f32) * 2.0 - 1.0;
                amplitude * normalized
            })
            .collect()
    }

    fn approx_eq(left: f32, right: f32, epsilon: f32) {
        assert!(
            (left - right).abs() <= epsilon,
            "left={left}, right={right}, epsilon={epsilon}"
        );
    }

    #[test]
    fn detects_a2_from_sine_wave() {
        let config = PitchDetectorConfig::default();
        let frame = sine_wave(110.0, config.sample_rate, config.frame_size, 0.8);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();
        approx_eq(result.frequency_hz, 110.0, 1.0);
        assert!(result.clarity >= config.min_clarity);
    }

    #[test]
    fn detects_low_e_string_from_sine_wave() {
        let config = PitchDetectorConfig::default();
        let frame = sine_wave(82.41, config.sample_rate, config.frame_size, 0.8);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();
        approx_eq(result.frequency_hz, 82.41, 1.0);
    }

    #[test]
    fn detects_high_e_string_from_sine_wave() {
        let config = PitchDetectorConfig::default();
        let frame = sine_wave(329.63, config.sample_rate, config.frame_size, 0.8);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();
        approx_eq(result.frequency_hz, 329.63, 1.5);
    }

    #[test]
    fn rejects_silence() {
        let config = PitchDetectorConfig::default();
        let frame = vec![0.0; config.frame_size];
        assert!(PitchDetector::new(config).detect_pitch(&frame).is_none());
    }

    #[test]
    fn rejects_too_low_amplitude() {
        let config = PitchDetectorConfig::default();
        let frame = sine_wave(110.0, config.sample_rate, config.frame_size, 0.001);
        assert!(PitchDetector::new(config).detect_pitch(&frame).is_none());
    }

    #[test]
    fn detects_fundamental_on_harmonic_rich_low_e_signal() {
        let config = PitchDetectorConfig::default();
        let frame = guitar_like_wave(82.41, config.sample_rate, config.frame_size, 0.45);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();

        approx_eq(result.frequency_hz, 82.41, 1.5);
    }

    #[test]
    fn detects_fundamental_on_harmonic_rich_d3_signal() {
        let config = PitchDetectorConfig::default();
        let frame = guitar_like_wave(146.83, config.sample_rate, config.frame_size, 0.38);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();

        approx_eq(result.frequency_hz, 146.83, 1.5);
    }

    #[test]
    fn rejects_non_periodic_noise() {
        let config = PitchDetectorConfig::default();
        let frame = noisy_frame(config.frame_size, 0.12);

        assert!(PitchDetector::new(config).detect_pitch(&frame).is_none());
    }

    #[test]
    fn keeps_low_e_fundamental_on_octave_ambiguous_signal() {
        let configs = [
            PitchDetectorConfig::default(),
            PitchDetectorConfig {
                sample_rate: 48_000,
                frame_size: 2_048,
                hop_size: 512,
                ..PitchDetectorConfig::default()
            },
        ];
        let amplitudes = [0.30, 0.22, 0.16, 0.10, 0.06];

        for config in configs {
            for amplitude in amplitudes {
                let frame =
                    octave_ambiguous_wave(82.41, config.sample_rate, config.frame_size, amplitude);
                let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();
                let deviation = (result.frequency_hz - 82.41).abs();
                let mut summary = String::new();
                if deviation > 2.0 {
                    let (tau_min, tau_max) = config.tau_range();
                    let (processed, _) = preprocess_frame(&frame, config.apply_hann_window);
                    let nsdf = compute_nsdf(&processed, tau_min, tau_max);
                    let peaks = find_local_maxima(&nsdf, tau_min);
                    let filtered =
                        filter_peak_candidates(&peaks, config.min_clarity, tau_min, tau_max);
                    let mut ranked = filtered.clone();
                    ranked.sort_by(|left, right| right.clarity.total_cmp(&left.clarity));
                    summary = format!(
                        " peaks={:?}",
                        ranked.into_iter().take(4).collect::<Vec<_>>()
                    );
                }
                assert!(
                    deviation <= 2.0,
                    "sr={}, frame={}, amp={}, detected={}{}",
                    config.sample_rate,
                    config.frame_size,
                    amplitude,
                    result.frequency_hz,
                    summary
                );
            }
        }
    }

    #[test]
    fn keeps_g3_without_octave_drop_on_short_web_frame() {
        let config = PitchDetectorConfig {
            sample_rate: 48_000,
            frame_size: 2_048,
            hop_size: 512,
            ..PitchDetectorConfig::default()
        };
        let frame = guitar_like_wave(196.00, config.sample_rate, config.frame_size, 0.32);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();

        approx_eq(result.frequency_hz, 196.0, 2.0);
    }

    #[test]
    fn keeps_high_e_without_octave_drop_on_short_web_frame() {
        let config = PitchDetectorConfig {
            sample_rate: 48_000,
            frame_size: 2_048,
            hop_size: 512,
            ..PitchDetectorConfig::default()
        };
        let frame = sine_wave(329.63, config.sample_rate, config.frame_size, 0.75);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();

        approx_eq(result.frequency_hz, 329.63, 1.5);
    }
}
