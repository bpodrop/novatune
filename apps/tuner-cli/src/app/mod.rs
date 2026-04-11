pub mod tuner_engine;

use tuner_core::{Note, PresetId, TunerMode};

pub use tuner_engine::{TunerConfig, TunerEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    Analyze(TunerMode),
    Target(Note),
}

impl SessionMode {
    pub fn chromatic() -> Self {
        Self::Analyze(TunerMode::Chromatic)
    }

    pub fn preset(preset_id: PresetId) -> Self {
        Self::Analyze(TunerMode::Preset(preset_id))
    }

    pub fn target(note: Note) -> Self {
        Self::Target(note)
    }

    pub fn current_preset(self) -> Option<PresetId> {
        match self {
            Self::Analyze(TunerMode::Preset(preset_id)) => Some(preset_id),
            Self::Analyze(TunerMode::Chromatic) | Self::Target(_) => None,
        }
    }

    pub fn is_analyze(self) -> bool {
        matches!(self, Self::Analyze(_))
    }
}
