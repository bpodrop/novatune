use crate::{Note, PitchDetectionResult, PresetId, UiState, match_frequency_to_preset, preset_by_id};

#[derive(Debug, Clone, PartialEq)]
pub struct MappedDetection {
    pub frequency_hz: f32,
    pub confidence: f32,
    pub clarity: f32,
    pub rms: f32,
    pub cents_off: f32,
    pub note_name: String,
    pub ui_state: UiState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningSession {
    preset_id: PresetId,
    calibration_hz: f32,
    preset_match_window_cents: f32,
}

impl Default for TuningSession {
    fn default() -> Self {
        Self {
            preset_id: PresetId::EStandard,
            calibration_hz: 440.0,
            preset_match_window_cents: 300.0,
        }
    }
}

impl TuningSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn preset_id(&self) -> PresetId {
        self.preset_id
    }

    pub fn calibration_hz(&self) -> f32 {
        self.calibration_hz
    }

    pub fn set_preset(&mut self, preset_id: &str) -> bool {
        let Some(parsed) = PresetId::parse(preset_id) else {
            return false;
        };

        self.preset_id = parsed;
        true
    }

    pub fn set_calibration_hz(&mut self, calibration_hz: f32) -> bool {
        if !calibration_hz.is_finite() || calibration_hz <= 0.0 {
            return false;
        }

        self.calibration_hz = calibration_hz;
        true
    }

    pub fn map_detection(&self, detection: PitchDetectionResult) -> MappedDetection {
        let normalized_frequency_hz = detection.frequency_hz * (440.0 / self.calibration_hz);
        let preset = preset_by_id(self.preset_id);
        let preset_match = match_frequency_to_preset(
            normalized_frequency_hz,
            preset,
            self.preset_match_window_cents,
        );

        let (cents_off, note_name) = match preset_match {
            Some(matched) => (
                matched.cents_from_target,
                matched.matched_string.label.to_string(),
            ),
            None => {
                let note_estimate = Note::estimate(normalized_frequency_hz);
                let fallback_cents = note_estimate
                    .as_ref()
                    .map(|note| note.cents_offset)
                    .unwrap_or(0.0);
                let fallback_name = note_estimate
                    .as_ref()
                    .map(|note| note.note_name.clone())
                    .unwrap_or_else(|| "--".to_string());
                (fallback_cents, fallback_name)
            }
        };

        MappedDetection {
            frequency_hz: detection.frequency_hz,
            confidence: detection.confidence,
            clarity: detection.clarity,
            rms: detection.rms,
            cents_off,
            note_name,
            ui_state: resolve_ui_state(detection.confidence, detection.clarity, cents_off),
        }
    }
}

pub fn resolve_ui_state(confidence: f32, clarity: f32, cents_off: f32) -> UiState {
    if confidence < 0.60 {
        return UiState::Searching;
    }
    if clarity < 0.70 {
        return UiState::Unstable;
    }
    if cents_off < -3.0 {
        UiState::TooLow
    } else if cents_off > 3.0 {
        UiState::TooHigh
    } else {
        UiState::InTune
    }
}

#[cfg(test)]
mod tests {
    use super::TuningSession;
    use crate::MeasuredPitch;

    fn measured_pitch(frequency_hz: f32) -> MeasuredPitch {
        MeasuredPitch {
            frequency_hz,
            confidence: 0.95,
            clarity: 0.95,
            rms: 0.2,
        }
    }

    #[test]
    fn supports_preset_and_calibration_updates() {
        let mut session = TuningSession::new();

        assert!(session.set_preset("drop-d"));
        assert_eq!(session.preset_id().as_str(), "drop-d");
        assert!(session.set_calibration_hz(432.0));
        assert_eq!(session.calibration_hz(), 432.0);

        assert!(!session.set_preset("invalid"));
        assert!(!session.set_calibration_hz(0.0));
    }

    #[test]
    fn maps_with_selected_preset() {
        let mut session = TuningSession::new();
        assert!(session.set_preset("drop-d"));

        let mapped = session.map_detection(measured_pitch(82.41));
        assert_eq!(mapped.note_name, "D2");
        assert!(mapped.cents_off > 100.0);
    }
}
