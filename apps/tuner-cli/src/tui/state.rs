use crate::app::SessionMode;
use std::collections::VecDeque;
use tuner_core::{PresetId, TunerMode, TunerOutput, UiState};

pub const DEFAULT_IN_TUNE_THRESHOLD_CENTS: f32 = 3.0;
pub const GAUGE_CLAMP_CENTS: f32 = 50.0;
pub const HISTORY_CAPACITY: usize = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalPhase {
    NoSignal,
    Searching,
    Unstable,
    TooLow,
    TooHigh,
    InTune,
}

impl From<UiState> for SignalPhase {
    fn from(value: UiState) -> Self {
        match value {
            UiState::NoSignal => Self::NoSignal,
            UiState::Searching => Self::Searching,
            UiState::Unstable => Self::Unstable,
            UiState::TooLow => Self::TooLow,
            UiState::TooHigh => Self::TooHigh,
            UiState::InTune => Self::InTune,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TunerUiState {
    pub app_label: &'static str,
    pub profile_label: String,
    pub mode_label: String,
    pub note_label: String,
    pub target_label: String,
    pub string_label: Option<String>,
    pub signal_label: &'static str,
    pub status_line: &'static str,
    pub cents_off: f32,
    pub phase: SignalPhase,
    pub frequency_hz: Option<f32>,
    pub target_hz: Option<f32>,
    pub confidence: Option<f32>,
    pub clarity: Option<f32>,
    pub noise_percent: Option<u8>,
    pub stability_percent: u8,
    pub sample_rate_hz: u32,
    pub history: Vec<u64>,
    pub animation_tick: u64,
    pub overlay_notice: Option<String>,
    pub help_visible: bool,
    pub preset_picker_visible: bool,
    pub preset_picker_items: Vec<(PresetId, String)>,
    pub preset_picker_selected: usize,
}

impl TunerUiState {
    pub fn no_signal(
        mode: SessionMode,
        sample_rate_hz: u32,
        history: Vec<u64>,
        animation_tick: u64,
    ) -> Self {
        Self {
            app_label: "ACCORD//R",
            profile_label: profile_label(mode),
            mode_label: mode_label(mode),
            note_label: "--".to_string(),
            target_label: "--".to_string(),
            string_label: None,
            signal_label: "NO SIGNAL",
            status_line: "NO INPUT SIGNAL",
            cents_off: 0.0,
            phase: SignalPhase::NoSignal,
            frequency_hz: None,
            target_hz: None,
            confidence: None,
            clarity: None,
            noise_percent: None,
            stability_percent: 0,
            sample_rate_hz,
            history,
            animation_tick,
            overlay_notice: None,
            help_visible: false,
            preset_picker_visible: false,
            preset_picker_items: Vec::new(),
            preset_picker_selected: 0,
        }
    }

    pub fn direction_label(&self) -> &'static str {
        direction_label(self.phase)
    }

    pub fn badge_label(&self) -> &'static str {
        badge_label(self.phase)
    }
}

pub fn output_to_ui_state(
    output: &TunerOutput,
    mode: SessionMode,
    sample_rate_hz: u32,
    history: Vec<u64>,
    animation_tick: u64,
) -> TunerUiState {
    let phase = SignalPhase::from(output.ui_state);

    let note_label = output
        .detected_note
        .as_ref()
        .map(|note| note.note_name.clone())
        .or_else(|| {
            output
                .target
                .as_ref()
                .map(|target| target.note_name.clone())
        })
        .unwrap_or_else(|| "--".to_string());

    let target_label = output
        .target
        .as_ref()
        .map(|target| target.note_name.clone())
        .unwrap_or_else(|| "--".to_string());

    let string_label = output.target.as_ref().and_then(|target| {
        target
            .string
            .map(|string| format!("{}{}", string.display_number, string.label))
    });

    let confidence = (output.measured_frequency_hz.is_some()).then_some(output.confidence);
    let clarity = confidence;
    let noise_percent =
        confidence.map(|value| ((1.0 - value).clamp(0.0, 1.0) * 100.0).round() as u8);

    TunerUiState {
        app_label: "ACCORD//R",
        profile_label: profile_label(mode),
        mode_label: mode_label(mode),
        note_label,
        target_label,
        string_label,
        signal_label: signal_label(phase),
        status_line: status_line(phase),
        cents_off: output.display_cents.unwrap_or_default(),
        phase,
        frequency_hz: output.measured_frequency_hz,
        target_hz: output.target.as_ref().map(|target| target.frequency_hz),
        confidence,
        clarity,
        noise_percent,
        stability_percent: derived_stability_percent(output.confidence, phase),
        sample_rate_hz,
        history,
        animation_tick,
        overlay_notice: None,
        help_visible: false,
        preset_picker_visible: false,
        preset_picker_items: Vec::new(),
        preset_picker_selected: 0,
    }
}

pub fn direction_label(phase: SignalPhase) -> &'static str {
    match phase {
        SignalPhase::NoSignal => "EN ATTENTE",
        SignalPhase::Searching => "SCANNER",
        SignalPhase::Unstable => "STABILISER",
        SignalPhase::TooLow => "TENDRE",
        SignalPhase::TooHigh => "DESSERRER",
        SignalPhase::InTune => "MAINTENIR",
    }
}

pub fn badge_label(phase: SignalPhase) -> &'static str {
    match phase {
        SignalPhase::NoSignal => "NO SIGNAL",
        SignalPhase::Searching => "SCANNING",
        SignalPhase::Unstable => "UNSTABLE",
        SignalPhase::TooLow => "TOO LOW",
        SignalPhase::TooHigh => "TOO HIGH",
        SignalPhase::InTune => "IN TUNE",
    }
}

fn mode_label(mode: SessionMode) -> String {
    match mode {
        SessionMode::Analyze(TunerMode::Chromatic) => "CHROMATIC".to_string(),
        SessionMode::Analyze(TunerMode::Preset(_)) => "PRESET".to_string(),
        SessionMode::Target(_) => "TUNE".to_string(),
    }
}

fn profile_label(mode: SessionMode) -> String {
    match mode {
        SessionMode::Analyze(TunerMode::Chromatic) => "CHROMATIC".to_string(),
        SessionMode::Analyze(TunerMode::Preset(preset_id)) => preset_id.display_name().to_string(),
        SessionMode::Target(note) => format!("TARGET {}", note.label()),
    }
}

fn signal_label(phase: SignalPhase) -> &'static str {
    match phase {
        SignalPhase::NoSignal => "NO SIGNAL",
        SignalPhase::Searching => "SCANNING",
        SignalPhase::Unstable => "UNSTABLE",
        SignalPhase::TooLow | SignalPhase::TooHigh | SignalPhase::InTune => "LOCKED",
    }
}

fn status_line(phase: SignalPhase) -> &'static str {
    match phase {
        SignalPhase::NoSignal => "NO INPUT SIGNAL",
        SignalPhase::Searching => "SCANNING INPUT",
        SignalPhase::Unstable => "NOISY // HOLD STRING",
        SignalPhase::TooLow => "TENSION TOO LOW",
        SignalPhase::TooHigh => "TENSION TOO HIGH",
        SignalPhase::InTune => "LOCKED // IN TUNE",
    }
}

fn derived_stability_percent(confidence: f32, phase: SignalPhase) -> u8 {
    let blended = (confidence.clamp(0.0, 1.0) * 100.0).round() as u8;

    match phase {
        SignalPhase::NoSignal => 0,
        SignalPhase::Searching => blended.min(45),
        SignalPhase::Unstable => blended.min(79),
        SignalPhase::TooLow | SignalPhase::TooHigh | SignalPhase::InTune => blended.max(80),
    }
}

#[derive(Debug, Clone)]
pub struct HistoryBuffer {
    values: VecDeque<u64>,
}

impl HistoryBuffer {
    pub fn new() -> Self {
        Self {
            values: VecDeque::with_capacity(HISTORY_CAPACITY),
        }
    }

    pub fn record(&mut self, output: &TunerOutput) {
        let sample = history_sample(output);
        if self.values.len() == HISTORY_CAPACITY {
            self.values.pop_front();
        }
        self.values.push_back(sample);
    }

    pub fn snapshot(&self) -> Vec<u64> {
        self.values.iter().copied().collect()
    }
}

fn history_sample(output: &TunerOutput) -> u64 {
    let Some(cents_off) = output.display_cents else {
        return 0;
    };

    let closeness = 1.0 - (cents_off.abs() / GAUGE_CLAMP_CENTS).clamp(0.0, 1.0);
    let base = (closeness * 8.0).round() as u64;

    match output.ui_state {
        UiState::TooLow | UiState::TooHigh | UiState::InTune => base.max(3),
        UiState::Searching | UiState::Unstable => base.min(5),
        UiState::NoSignal => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{SignalPhase, TunerUiState, badge_label, direction_label, output_to_ui_state};
    use crate::app::SessionMode;
    use tuner_core::{Note, TunerMode, TunerOutput, UiState};

    #[test]
    fn labels_match_expected_copy() {
        assert_eq!(direction_label(SignalPhase::NoSignal), "EN ATTENTE");
        assert_eq!(direction_label(SignalPhase::TooLow), "TENDRE");
        assert_eq!(badge_label(SignalPhase::InTune), "IN TUNE");
    }

    #[test]
    fn chromatic_output_maps_to_unstable_ui_state() {
        let output = TunerOutput {
            mode: TunerMode::Chromatic,
            measured_frequency_hz: Some(109.0),
            confidence: 0.9,
            detected_note: Some(Note::estimate(109.0).unwrap()),
            display_cents: Some(-12.0),
            target: None,
            ui_state: UiState::Unstable,
        };

        let ui_state = output_to_ui_state(&output, SessionMode::chromatic(), 48_000, vec![2, 3], 4);

        assert_eq!(ui_state.note_label, "A2");
        assert_eq!(ui_state.phase, SignalPhase::Unstable);
        assert_eq!(ui_state.frequency_hz, Some(109.0));
    }

    #[test]
    fn no_signal_builder_keeps_neutral_state() {
        let ui_state = TunerUiState::no_signal(SessionMode::chromatic(), 48_000, vec![1, 2], 3);

        assert_eq!(ui_state.phase, SignalPhase::NoSignal);
        assert_eq!(ui_state.mode_label, "CHROMATIC");
    }
}
