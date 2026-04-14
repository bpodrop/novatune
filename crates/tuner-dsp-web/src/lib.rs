use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use tuner_core::{
    Note, PitchDetectionResult, TuningSession, UiState, all_presets, resolve_ui_state,
};
use tuner_dsp_algo::{PitchDetector, PitchDetectorConfig};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectionOutput {
    pub frequency_hz: f32,
    pub confidence: f32,
    pub clarity: f32,
    pub rms: f32,
    pub cents_off: f32,
    pub note_name: String,
    pub tuning_profile_id: Option<String>,
    pub string_index: Option<u8>,
    pub string_count: Option<u8>,
    pub string_name: Option<String>,
    pub mode: String,
    pub ui_state: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresetStringOutput {
    pub index: u8,
    pub display_number: u8,
    pub label: String,
    pub frequency_hz: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresetOutput {
    pub id: String,
    pub label: String,
    pub string_count: u8,
    pub strings: Vec<PresetStringOutput>,
}

impl From<PitchDetectionResult> for DetectionOutput {
    fn from(value: PitchDetectionResult) -> Self {
        let note_estimate = Note::estimate(value.frequency_hz);
        let cents_off = note_estimate
            .as_ref()
            .map(|note| note.cents_offset)
            .unwrap_or(0.0);
        let note_name = note_estimate
            .as_ref()
            .map(|note| note.note_name.clone())
            .unwrap_or_else(|| "--".to_string());
        let ui_state = resolve_ui_state(value.confidence, value.clarity, cents_off);

        Self {
            frequency_hz: value.frequency_hz,
            confidence: value.confidence,
            clarity: value.clarity,
            rms: value.rms,
            cents_off,
            note_name,
            tuning_profile_id: None,
            string_index: None,
            string_count: None,
            string_name: None,
            mode: "chromatic".to_string(),
            ui_state: ui_state_label(ui_state).to_string(),
        }
    }
}

fn map_detection_for_tuning(
    value: PitchDetectionResult,
    tuning_session: &TuningSession,
) -> DetectionOutput {
    let mapped = tuning_session.map_detection(value);

    DetectionOutput {
        frequency_hz: mapped.frequency_hz,
        confidence: mapped.confidence,
        clarity: mapped.clarity,
        rms: mapped.rms,
        cents_off: mapped.cents_off,
        note_name: mapped.note_name,
        tuning_profile_id: mapped.tuning_profile_id,
        string_index: mapped.string_index,
        string_count: mapped.string_count,
        string_name: mapped.string_name,
        mode: mapped.mode.as_str().to_string(),
        ui_state: ui_state_label(mapped.ui_state).to_string(),
    }
}

fn ui_state_label(ui_state: UiState) -> &'static str {
    match ui_state {
        UiState::NoSignal => "no_signal",
        UiState::Searching => "searching",
        UiState::Unstable => "unstable",
        UiState::TooLow => "too_low",
        UiState::InTune => "in_tune",
        UiState::TooHigh => "too_high",
    }
}

#[derive(Debug)]
struct DetectorHandle {
    detector: PitchDetector,
    config: PitchDetectorConfig,
    tuning_session: TuningSession,
    pending_samples: Vec<f32>,
    outputs: VecDeque<DetectionOutput>,
}

#[derive(Debug, Default)]
struct DetectorRegistry {
    next_id: u32,
    detectors: HashMap<u32, DetectorHandle>,
}

impl DetectorRegistry {
    fn allocate_id(&mut self) -> u32 {
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == 0 {
            self.next_id = 1;
        }

        while self.detectors.contains_key(&self.next_id) {
            self.next_id = self.next_id.wrapping_add(1);
            if self.next_id == 0 {
                self.next_id = 1;
            }
        }

        self.next_id
    }
}

fn registry() -> &'static Mutex<DetectorRegistry> {
    static REGISTRY: OnceLock<Mutex<DetectorRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(DetectorRegistry::default()))
}

fn is_valid_config(sample_rate: u32, frame_size: usize, hop_size: usize) -> bool {
    sample_rate > 0 && frame_size > 0 && hop_size > 0 && hop_size <= frame_size
}

pub fn new_detector(sample_rate: u32, frame_size: usize, hop_size: usize) -> u32 {
    if !is_valid_config(sample_rate, frame_size, hop_size) {
        return 0;
    }

    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");

    let id = lock.allocate_id();
    let config = PitchDetectorConfig {
        sample_rate,
        frame_size,
        hop_size,
        ..PitchDetectorConfig::default()
    };

    lock.detectors.insert(
        id,
        DetectorHandle {
            detector: PitchDetector::new(config),
            config,
            tuning_session: TuningSession::new(),
            pending_samples: Vec::new(),
            outputs: VecDeque::new(),
        },
    );

    id
}

pub fn push_samples(detector_id: u32, samples: &[f32]) -> usize {
    if detector_id == 0 || samples.is_empty() {
        return 0;
    }

    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return 0;
    };

    if !is_valid_config(
        handle.config.sample_rate,
        handle.config.frame_size,
        handle.config.hop_size,
    ) {
        return 0;
    }

    handle.pending_samples.extend_from_slice(samples);

    let mut produced = 0;
    while handle.pending_samples.len() >= handle.config.frame_size {
        let frame = &handle.pending_samples[..handle.config.frame_size];
        if let Some(output) = handle.detector.detect_pitch(frame) {
            handle
                .outputs
                .push_back(map_detection_for_tuning(output, &handle.tuning_session));
            produced += 1;
        }

        let advance = handle.config.hop_size.min(handle.config.frame_size).max(1);
        handle.pending_samples.drain(..advance);
    }

    produced
}

pub fn next_output(detector_id: u32) -> Option<DetectionOutput> {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let handle = lock.detectors.get_mut(&detector_id)?;
    handle.outputs.pop_front()
}

pub fn reset(detector_id: u32) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return false;
    };

    handle.detector = PitchDetector::new(handle.config);
    handle.pending_samples.clear();
    handle.outputs.clear();

    true
}

pub fn set_preset(detector_id: u32, preset_id: &str) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return false;
    };

    handle.tuning_session.set_preset(preset_id)
}

pub fn set_calibration_hz(detector_id: u32, calibration_hz: f32) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return false;
    };

    handle.tuning_session.set_calibration_hz(calibration_hz)
}

pub fn set_mode(detector_id: u32, mode: &str) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return false;
    };

    handle.tuning_session.set_mode(mode)
}

pub fn set_preset_match_window_cents(detector_id: u32, window_cents: f32) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return false;
    };

    handle
        .tuning_session
        .set_preset_match_window_cents(window_cents)
}

pub fn set_min_rms(detector_id: u32, min_rms: f32) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return false;
    };

    handle.detector.set_min_rms(min_rms)
}

pub fn set_min_clarity(detector_id: u32, min_clarity: f32) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return false;
    };

    handle.detector.set_min_clarity(min_clarity)
}

pub fn shutdown(detector_id: u32) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    lock.detectors.remove(&detector_id).is_some()
}

pub fn list_presets() -> Vec<PresetOutput> {
    all_presets()
        .iter()
        .map(|preset| PresetOutput {
            id: preset.id.as_str().to_string(),
            label: preset.display_name.to_string(),
            string_count: preset.string_count() as u8,
            strings: preset
                .strings
                .iter()
                .map(|target| PresetStringOutput {
                    index: target.index,
                    display_number: target.display_number,
                    label: target.label.to_string(),
                    frequency_hz: target.frequency_hz,
                })
                .collect(),
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use serde_wasm_bindgen::to_value;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(js_name = new_detector)]
    pub fn wasm_new_detector(sample_rate: u32, frame_size: usize, hop_size: usize) -> u32 {
        super::new_detector(sample_rate, frame_size, hop_size)
    }

    #[wasm_bindgen(js_name = push_samples)]
    pub fn wasm_push_samples(detector_id: u32, samples: Vec<f32>) -> usize {
        super::push_samples(detector_id, &samples)
    }

    #[wasm_bindgen(js_name = next_output)]
    pub fn wasm_next_output(detector_id: u32) -> Option<JsValue> {
        super::next_output(detector_id).map(|output| {
            to_value(&output).expect("detection output serialization should not fail")
        })
    }

    #[wasm_bindgen(js_name = reset)]
    pub fn wasm_reset(detector_id: u32) -> bool {
        super::reset(detector_id)
    }

    #[wasm_bindgen(js_name = shutdown)]
    pub fn wasm_shutdown(detector_id: u32) -> bool {
        super::shutdown(detector_id)
    }

    #[wasm_bindgen(js_name = set_preset)]
    pub fn wasm_set_preset(detector_id: u32, preset_id: String) -> bool {
        super::set_preset(detector_id, &preset_id)
    }

    #[wasm_bindgen(js_name = set_calibration_hz)]
    pub fn wasm_set_calibration_hz(detector_id: u32, calibration_hz: f32) -> bool {
        super::set_calibration_hz(detector_id, calibration_hz)
    }

    #[wasm_bindgen(js_name = set_mode)]
    pub fn wasm_set_mode(detector_id: u32, mode: String) -> bool {
        super::set_mode(detector_id, &mode)
    }

    #[wasm_bindgen(js_name = set_preset_match_window_cents)]
    pub fn wasm_set_preset_match_window_cents(detector_id: u32, window_cents: f32) -> bool {
        super::set_preset_match_window_cents(detector_id, window_cents)
    }

    #[wasm_bindgen(js_name = set_min_rms)]
    pub fn wasm_set_min_rms(detector_id: u32, min_rms: f32) -> bool {
        super::set_min_rms(detector_id, min_rms)
    }

    #[wasm_bindgen(js_name = set_min_clarity)]
    pub fn wasm_set_min_clarity(detector_id: u32, min_clarity: f32) -> bool {
        super::set_min_clarity(detector_id, min_clarity)
    }

    #[wasm_bindgen(js_name = list_presets)]
    pub fn wasm_list_presets() -> JsValue {
        to_value(&super::list_presets()).expect("preset list serialization should not fail")
    }
}
