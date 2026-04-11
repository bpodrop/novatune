use crate::app::SessionMode;
use crate::tui::state::{SignalPhase, TunerUiState};
use std::time::{Duration, Instant};
use tuner_core::PresetId;

pub struct DemoState {
    started_at: Instant,
}

struct DemoTunerState {
    note_label: &'static str,
    target_label: &'static str,
    string_label: Option<&'static str>,
    cents_off: f32,
    phase: SignalPhase,
    frequency_hz: Option<f32>,
    target_hz: Option<f32>,
    confidence: Option<f32>,
    clarity: Option<f32>,
    noise_percent: u8,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    pub fn current_frame(&self) -> TunerUiState {
        let elapsed = self.started_at.elapsed();
        let cycle = elapsed.as_secs() % 8;
        let history = demo_history(elapsed);
        let animation_tick = (elapsed.as_millis() / 120) as u64;
        let preset_mode = SessionMode::preset(PresetId::DropD);

        match cycle {
            0 => TunerUiState::no_signal(preset_mode, 48_000, history, animation_tick),
            1 => tuner_state(
                DemoTunerState {
                    note_label: "D2",
                    target_label: "D2",
                    string_label: Some("6D"),
                    cents_off: 0.0,
                    phase: SignalPhase::Searching,
                    frequency_hz: Some(73.10),
                    target_hz: Some(73.42),
                    confidence: Some(0.41),
                    clarity: Some(0.32),
                    noise_percent: 38,
                },
                history,
                animation_tick,
            ),
            2 | 3 => tuner_state(
                DemoTunerState {
                    note_label: "D2",
                    target_label: "D2",
                    string_label: Some("6D"),
                    cents_off: animated_cents(elapsed, -20.0, -6.0, Duration::from_secs(2)),
                    phase: SignalPhase::TooLow,
                    frequency_hz: Some(73.18),
                    target_hz: Some(73.42),
                    confidence: Some(0.91),
                    clarity: Some(0.88),
                    noise_percent: 14,
                },
                history,
                animation_tick,
            ),
            4 | 5 => tuner_state(
                DemoTunerState {
                    note_label: "D2",
                    target_label: "D2",
                    string_label: Some("6D"),
                    cents_off: animated_cents(elapsed, -1.5, 1.5, Duration::from_secs(2)),
                    phase: SignalPhase::InTune,
                    frequency_hz: Some(73.42),
                    target_hz: Some(73.42),
                    confidence: Some(0.96),
                    clarity: Some(0.95),
                    noise_percent: 5,
                },
                history,
                animation_tick,
            ),
            _ => tuner_state(
                DemoTunerState {
                    note_label: "D2",
                    target_label: "D2",
                    string_label: Some("6D"),
                    cents_off: animated_cents(elapsed, 16.0, 6.0, Duration::from_secs(2)),
                    phase: SignalPhase::TooHigh,
                    frequency_hz: Some(73.78),
                    target_hz: Some(73.42),
                    confidence: Some(0.89),
                    clarity: Some(0.82),
                    noise_percent: 18,
                },
                history,
                animation_tick,
            ),
        }
    }
}

fn tuner_state(demo_state: DemoTunerState, history: Vec<u64>, animation_tick: u64) -> TunerUiState {
    let DemoTunerState {
        note_label,
        target_label,
        string_label,
        cents_off,
        phase,
        frequency_hz,
        target_hz,
        confidence,
        clarity,
        noise_percent,
    } = demo_state;
    TunerUiState {
        app_label: "ACCORD//R",
        profile_label: "Drop D".to_string(),
        mode_label: "PRESET".to_string(),
        note_label: note_label.to_string(),
        target_label: target_label.to_string(),
        string_label: string_label.map(str::to_string),
        signal_label: match phase {
            SignalPhase::NoSignal => "NO SIGNAL",
            SignalPhase::Searching => "SCANNING",
            SignalPhase::Unstable => "UNSTABLE",
            SignalPhase::TooLow | SignalPhase::TooHigh | SignalPhase::InTune => "LOCKED",
        },
        status_line: match phase {
            SignalPhase::NoSignal => "NO INPUT SIGNAL",
            SignalPhase::Searching => "SCANNING INPUT",
            SignalPhase::Unstable => "NOISY // HOLD STRING",
            SignalPhase::TooLow => "TENSION TOO LOW",
            SignalPhase::TooHigh => "TENSION TOO HIGH",
            SignalPhase::InTune => "LOCKED // IN TUNE",
        },
        cents_off,
        phase,
        frequency_hz,
        target_hz,
        confidence,
        clarity,
        noise_percent: Some(noise_percent),
        stability_percent: match phase {
            SignalPhase::NoSignal => 0,
            SignalPhase::Searching => 41,
            SignalPhase::Unstable => 64,
            SignalPhase::TooLow | SignalPhase::TooHigh => 86,
            SignalPhase::InTune => 97,
        },
        sample_rate_hz: 48_000,
        history,
        animation_tick,
        overlay_notice: None,
        help_visible: false,
        preset_picker_visible: false,
        preset_picker_items: Vec::new(),
        preset_picker_selected: 0,
    }
}

fn animated_cents(
    elapsed: Duration,
    start_cents: f32,
    end_cents: f32,
    phase_duration: Duration,
) -> f32 {
    let cycle_ms = phase_duration.as_millis().max(1) as f32;
    let elapsed_ms = (elapsed.as_millis() % phase_duration.as_millis().max(1)) as f32;
    let progress = elapsed_ms / cycle_ms;

    start_cents + (end_cents - start_cents) * progress
}

fn demo_history(elapsed: Duration) -> Vec<u64> {
    (0..18)
        .map(|index| {
            let wave = ((elapsed.as_millis() as f32 / 180.0) + index as f32 * 0.45).sin();
            ((wave + 1.0) * 4.0).round() as u64
        })
        .collect()
}
