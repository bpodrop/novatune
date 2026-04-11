use crate::app::SessionMode;
use std::collections::VecDeque;
use tuner_core::{
    Cents, MeasuredPitch, Note, NoteEstimate, PresetId, TunerMode, TunerOutput, TuningTarget, UiState,
    default_preset, preset_by_id,
};
use tuner_dsp_algo::{PitchDetector, PitchDetectorConfig};
use tuner_dsp_native::TuningSession;

const NOTE_HYSTERESIS_CENTS: f32 = 35.0;

#[derive(Debug, Clone)]
pub struct TunerConfig {
    pub session_mode: SessionMode,
    pub min_rms: f32,
    pub min_clarity: f32,
    pub in_tune_threshold_cents: f32,
    pub calibration_hz: f32,
    pub preset_match_window_cents: f32,
    pub stable_frames_required: usize,
}

impl Default for TunerConfig {
    fn default() -> Self {
        Self {
            session_mode: SessionMode::chromatic(),
            min_rms: 0.01,
            min_clarity: 0.60,
            in_tune_threshold_cents: 3.0,
            calibration_hz: 440.0,
            preset_match_window_cents: 100.0,
            stable_frames_required: 3,
        }
    }
}

impl TunerConfig {
    pub fn from_detector_config(config: PitchDetectorConfig, session_mode: SessionMode) -> Self {
        Self {
            session_mode,
            min_rms: config.min_rms,
            min_clarity: config.min_clarity,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct TunerEngine {
    detector: PitchDetector,
    tuning_session: TuningSession,
    config: TunerConfig,
    last_preset_id: PresetId,
    published_frequencies_hz: VecDeque<f32>,
    last_note: Option<NoteEstimate>,
    stable_pitch_count: usize,
    stable_string_count: usize,
    last_matched_string_index: Option<u8>,
}

impl TunerEngine {
    pub fn new(detector_config: PitchDetectorConfig, config: TunerConfig) -> Self {
        let window_size = detector_config
            .smoothing_window_size
            .max(config.stable_frames_required);
        let initial_preset_id = config
            .session_mode
            .current_preset()
            .unwrap_or(default_preset().id);
        let mut tuning_session = TuningSession::new();
        let _ = tuning_session.set_preset(initial_preset_id.as_str());
        let _ = tuning_session.set_calibration_hz(config.calibration_hz);

        Self {
            detector: PitchDetector::new(detector_config),
            tuning_session,
            last_preset_id: initial_preset_id,
            config,
            published_frequencies_hz: VecDeque::with_capacity(window_size),
            last_note: None,
            stable_pitch_count: 0,
            stable_string_count: 0,
            last_matched_string_index: None,
        }
    }

    pub fn process_frame(&mut self, frame: &[f32]) -> TunerOutput {
        let detection = self.detector.detect_pitch(frame);
        self.process_detection(detection)
    }

    pub fn session_mode(&self) -> SessionMode {
        self.config.session_mode
    }

    pub fn set_session_mode(&mut self, mode: SessionMode) {
        if let Some(preset_id) = mode.current_preset() {
            self.last_preset_id = preset_id;
            let _ = self.tuning_session.set_preset(preset_id.as_str());
        }
        self.config.session_mode = mode;
        self.reset_tracking();
    }

    pub fn toggle_analyze_mode(&mut self) -> Option<SessionMode> {
        let next_mode = match self.config.session_mode {
            SessionMode::Analyze(TunerMode::Chromatic) => SessionMode::preset(self.last_preset_id),
            SessionMode::Analyze(TunerMode::Preset(preset_id)) => {
                self.last_preset_id = preset_id;
                SessionMode::chromatic()
            }
            SessionMode::Target(_) => return None,
        };

        self.set_session_mode(next_mode);
        Some(next_mode)
    }

    pub fn select_preset(&mut self, preset_id: PresetId) -> PresetId {
        self.set_session_mode(SessionMode::preset(preset_id));
        preset_id
    }

    pub fn process_detection(&mut self, detection: Option<MeasuredPitch>) -> TunerOutput {
        let Some(detection) = detection else {
            self.reset_tracking();
            return self.empty_output(UiState::NoSignal);
        };

        if detection.rms < self.config.min_rms {
            self.reset_tracking();
            return self.empty_output(UiState::NoSignal);
        }

        let detected_note = self.apply_note_hysteresis(detection.frequency_hz);
        let previous_midi = self.last_note.as_ref().map(|note| note.midi);

        if let Some(note_estimate) = &detected_note {
            self.record_published_frequency(detection.frequency_hz);

            if previous_midi == Some(note_estimate.midi) || previous_midi.is_none() {
                self.stable_pitch_count = self.stable_pitch_count.saturating_add(1);
            } else {
                self.stable_pitch_count = 1;
            }
            self.last_note = Some(note_estimate.clone());
        } else {
            self.published_frequencies_hz.clear();
            self.stable_pitch_count = 0;
        }

        if detection.clarity < self.config.min_clarity {
            return self.output_for_detection(
                detection,
                detected_note,
                None,
                None,
                UiState::Searching,
            );
        }

        match self.config.session_mode {
            SessionMode::Analyze(TunerMode::Chromatic) => {
                let Some(note_estimate) = detected_note.clone() else {
                    return self.output_for_detection(
                        detection,
                        None,
                        None,
                        None,
                        UiState::Searching,
                    );
                };

                let target = TuningTarget {
                    note_name: note_estimate.note_name.clone(),
                    frequency_hz: note_estimate.target_frequency_hz,
                    preset_id: None,
                    string: None,
                };
                let display_cents = Some(note_estimate.cents_offset);
                let ui_state = self.resolve_tuning_state(
                    self.stable_pitch_count >= self.config.stable_frames_required,
                    note_estimate.cents_offset,
                );

                self.output_for_detection(
                    detection,
                    Some(note_estimate),
                    Some(target),
                    display_cents,
                    ui_state,
                )
            }
            SessionMode::Analyze(TunerMode::Preset(preset_id)) => {
                let mapped = self.tuning_session.map_detection(detection);
                let preset = preset_by_id(preset_id);
                let maybe_target_string = preset
                    .strings
                    .iter()
                    .find(|target| target.label == mapped.note_name);
                let is_within_window = mapped.cents_off.abs() <= self.config.preset_match_window_cents;

                let Some(target_string) = maybe_target_string.copied().filter(|_| is_within_window) else {
                    self.stable_string_count = 0;
                    self.last_matched_string_index = None;
                    return self.output_for_detection(
                        detection,
                        detected_note,
                        None,
                        None,
                        UiState::Unstable,
                    );
                };

                if self.last_matched_string_index == Some(target_string.index) {
                    self.stable_string_count = self.stable_string_count.saturating_add(1);
                } else {
                    self.last_matched_string_index = Some(target_string.index);
                    self.stable_string_count = 1;
                }

                let calibrated_target_hz = target_string.frequency_hz * (self.config.calibration_hz / 440.0);
                let target = TuningTarget {
                    note_name: target_string.label.to_string(),
                    frequency_hz: calibrated_target_hz,
                    preset_id: Some(preset_id),
                    string: Some(target_string),
                };
                let is_stable = self.stable_pitch_count >= self.config.stable_frames_required
                    && self.stable_string_count >= self.config.stable_frames_required;
                let ui_state = self.resolve_tuning_state(is_stable, mapped.cents_off);

                self.output_for_detection(
                    detection,
                    detected_note,
                    Some(target),
                    Some(mapped.cents_off),
                    ui_state,
                )
            }
            SessionMode::Target(target_note) => {
                let cents_off =
                    Cents::between(detection.frequency_hz, target_note.target_frequency_hz())
                        .map(|value| value.value())
                        .unwrap_or(0.0);
                let is_stable = self.stable_pitch_count >= self.config.stable_frames_required;
                let ui_state = self.resolve_tuning_state(is_stable, cents_off);
                let target = TuningTarget {
                    note_name: target_note.label(),
                    frequency_hz: target_note.target_frequency_hz(),
                    preset_id: None,
                    string: None,
                };

                self.output_for_detection(
                    detection,
                    detected_note,
                    Some(target),
                    Some(cents_off),
                    ui_state,
                )
            }
        }
    }

    fn output_for_detection(
        &self,
        detection: MeasuredPitch,
        detected_note: Option<NoteEstimate>,
        target: Option<TuningTarget>,
        display_cents: Option<f32>,
        ui_state: UiState,
    ) -> TunerOutput {
        TunerOutput {
            mode: match self.config.session_mode {
                SessionMode::Analyze(mode) => mode,
                SessionMode::Target(_) => TunerMode::Chromatic,
            },
            measured_frequency_hz: Some(detection.frequency_hz),
            confidence: detection.confidence,
            detected_note,
            display_cents,
            target,
            ui_state,
        }
    }

    fn empty_output(&self, ui_state: UiState) -> TunerOutput {
        TunerOutput {
            mode: match self.config.session_mode {
                SessionMode::Analyze(mode) => mode,
                SessionMode::Target(_) => TunerMode::Chromatic,
            },
            measured_frequency_hz: None,
            confidence: 0.0,
            detected_note: None,
            display_cents: None,
            target: None,
            ui_state,
        }
    }

    fn resolve_tuning_state(&self, is_stable: bool, cents_off: f32) -> UiState {
        if !is_stable {
            return UiState::Unstable;
        }

        if cents_off < -self.config.in_tune_threshold_cents {
            UiState::TooLow
        } else if cents_off > self.config.in_tune_threshold_cents {
            UiState::TooHigh
        } else {
            UiState::InTune
        }
    }

    fn apply_note_hysteresis(&self, frequency_hz: f32) -> Option<NoteEstimate> {
        let estimated_note = Note::estimate(frequency_hz)?;

        let Some(previous_note) = &self.last_note else {
            return Some(estimated_note);
        };

        if previous_note.midi == estimated_note.midi {
            return Some(estimated_note);
        }

        let cents_from_previous =
            Cents::between(frequency_hz, previous_note.target_frequency_hz)?.value();

        if cents_from_previous.abs() <= NOTE_HYSTERESIS_CENTS {
            return Some(NoteEstimate {
                note: previous_note.note,
                note_name: previous_note.note_name.clone(),
                midi: previous_note.midi,
                target_frequency_hz: previous_note.target_frequency_hz,
                cents_offset: cents_from_previous,
            });
        }

        Some(estimated_note)
    }

    fn record_published_frequency(&mut self, frequency_hz: f32) {
        if self.published_frequencies_hz.len() == self.published_frequencies_hz.capacity() {
            self.published_frequencies_hz.pop_front();
        }

        self.published_frequencies_hz.push_back(frequency_hz);
    }
    fn reset_tracking(&mut self) {
        self.published_frequencies_hz.clear();
        self.last_note = None;
        self.stable_pitch_count = 0;
        self.stable_string_count = 0;
        self.last_matched_string_index = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{SessionMode, TunerConfig, TunerEngine};
    use tuner_core::{MeasuredPitch, PresetId, UiState};
    use tuner_dsp_algo::PitchDetectorConfig;

    fn measured_pitch(frequency_hz: f32, confidence: f32, clarity: f32) -> MeasuredPitch {
        MeasuredPitch {
            frequency_hz,
            confidence,
            clarity,
            rms: 0.2,
        }
    }

    #[test]
    fn returns_no_signal_when_detection_is_absent() {
        let detector_config = PitchDetectorConfig::default();
        let config = TunerConfig::from_detector_config(detector_config, SessionMode::chromatic());
        let mut engine = TunerEngine::new(detector_config, config);

        let output = engine.process_detection(None);

        assert_eq!(output.ui_state, UiState::NoSignal);
        assert!(output.measured_frequency_hz.is_none());
    }

    #[test]
    fn chromatic_mode_locks_after_stable_frames() {
        let detector_config = PitchDetectorConfig::default();
        let config = TunerConfig::from_detector_config(detector_config, SessionMode::chromatic());
        let mut engine = TunerEngine::new(detector_config, config);

        let output1 = engine.process_detection(Some(measured_pitch(110.0, 0.9, 0.9)));
        let output2 = engine.process_detection(Some(measured_pitch(110.0, 0.9, 0.9)));
        let output3 = engine.process_detection(Some(measured_pitch(110.0, 0.9, 0.9)));

        assert_eq!(output1.ui_state, UiState::Unstable);
        assert_eq!(output2.ui_state, UiState::Unstable);
        assert_eq!(output3.ui_state, UiState::InTune);
        assert_eq!(output3.target.unwrap().note_name, "A2");
    }

    #[test]
    fn preset_mode_matches_target_string() {
        let detector_config = PitchDetectorConfig::default();
        let config = TunerConfig::from_detector_config(
            detector_config,
            SessionMode::preset(PresetId::DropD),
        );
        let mut engine = TunerEngine::new(detector_config, config);

        let _ = engine.process_detection(Some(measured_pitch(73.42, 0.95, 0.95)));
        let _ = engine.process_detection(Some(measured_pitch(73.42, 0.95, 0.95)));
        let output = engine.process_detection(Some(measured_pitch(73.42, 0.95, 0.95)));

        let target = output.target.unwrap();
        assert_eq!(output.ui_state, UiState::InTune);
        assert_eq!(target.preset_id, Some(PresetId::DropD));
        assert_eq!(target.string.unwrap().label, "D2");
    }

    #[test]
    fn preset_mode_marks_out_of_window_as_unstable() {
        let detector_config = PitchDetectorConfig::default();
        let config = TunerConfig::from_detector_config(
            detector_config,
            SessionMode::preset(PresetId::EStandard),
        );
        let mut engine = TunerEngine::new(detector_config, config);

        let output = engine.process_detection(Some(measured_pitch(98.0, 0.9, 0.9)));

        assert_eq!(output.ui_state, UiState::Unstable);
        assert!(output.target.is_none());
    }

    #[test]
    fn toggle_analyze_mode_reuses_last_preset() {
        let detector_config = PitchDetectorConfig::default();
        let config = TunerConfig::from_detector_config(
            detector_config,
            SessionMode::preset(PresetId::DropD),
        );
        let mut engine = TunerEngine::new(detector_config, config);

        assert_eq!(engine.toggle_analyze_mode(), Some(SessionMode::chromatic()));
        assert_eq!(
            engine.toggle_analyze_mode(),
            Some(SessionMode::preset(PresetId::DropD))
        );
    }
}
