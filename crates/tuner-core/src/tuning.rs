use crate::{Cents, Note};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresetId {
    EStandard,
    DropD,
    EbStandard,
    DStandard,
    DropC,
    OpenG,
    OpenD,
    Dadgad,
    BStandard7,
    DropA7,
    AStandard7,
    FSharpStandard8,
    EStandard8,
    CSharpStandard9,
    DropB9,
}

impl PresetId {
    pub const ALL: [PresetId; 15] = [
        PresetId::EStandard,
        PresetId::DropD,
        PresetId::EbStandard,
        PresetId::DStandard,
        PresetId::DropC,
        PresetId::OpenG,
        PresetId::OpenD,
        PresetId::Dadgad,
        PresetId::BStandard7,
        PresetId::DropA7,
        PresetId::AStandard7,
        PresetId::FSharpStandard8,
        PresetId::EStandard8,
        PresetId::CSharpStandard9,
        PresetId::DropB9,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::EStandard => "e-standard",
            Self::DropD => "drop-d",
            Self::EbStandard => "eb-standard",
            Self::DStandard => "d-standard",
            Self::DropC => "drop-c",
            Self::OpenG => "open-g",
            Self::OpenD => "open-d",
            Self::Dadgad => "dadgad",
            Self::BStandard7 => "b-standard-7",
            Self::DropA7 => "drop-a-7",
            Self::AStandard7 => "a-standard-7",
            Self::FSharpStandard8 => "fsharp-standard-8",
            Self::EStandard8 => "e-standard-8",
            Self::CSharpStandard9 => "csharp-standard-9",
            Self::DropB9 => "drop-b-9",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::EStandard => "E Standard",
            Self::DropD => "Drop D",
            Self::EbStandard => "Eb Standard",
            Self::DStandard => "D Standard",
            Self::DropC => "Drop C",
            Self::OpenG => "Open G",
            Self::OpenD => "Open D",
            Self::Dadgad => "DADGAD",
            Self::BStandard7 => "B Standard (7)",
            Self::DropA7 => "Drop A (7)",
            Self::AStandard7 => "A Standard (7)",
            Self::FSharpStandard8 => "F# Standard (8)",
            Self::EStandard8 => "E Standard (8)",
            Self::CSharpStandard9 => "C# Standard (9)",
            Self::DropB9 => "Drop B (9)",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "e-standard" | "estandard" | "standard" | "standard-e" => Some(Self::EStandard),
            "drop-d" | "dropd" => Some(Self::DropD),
            "eb-standard" | "ebstandard" | "dsharp-standard" | "d#-standard" => {
                Some(Self::EbStandard)
            }
            "d-standard" | "dstandard" => Some(Self::DStandard),
            "drop-c" | "dropc" => Some(Self::DropC),
            "open-g" | "openg" => Some(Self::OpenG),
            "open-d" | "opend" => Some(Self::OpenD),
            "dadgad" => Some(Self::Dadgad),
            "b-standard" | "bstandard" | "b-standard-7" => Some(Self::BStandard7),
            "drop-a" | "dropa" | "drop-a-7" => Some(Self::DropA7),
            "a-standard" | "astandard" | "a-standard-7" => Some(Self::AStandard7),
            "fsharp-standard" | "f#-standard" | "fsharp-standard-8" => Some(Self::FSharpStandard8),
            "e-standard-8" => Some(Self::EStandard8),
            "csharp-standard" | "c#-standard" | "csharp-standard-9" => Some(Self::CSharpStandard9),
            "drop-b" | "dropb" | "drop-b-9" => Some(Self::DropB9),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetString {
    pub index: u8,
    pub display_number: u8,
    pub label: &'static str,
    pub note: Note,
    pub frequency_hz: f32,
}

impl TargetString {
    pub fn note_name(self) -> &'static str {
        self.label
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningPreset {
    pub id: PresetId,
    pub display_name: &'static str,
    pub strings: &'static [TargetString],
}

impl TuningPreset {
    pub fn string_by_index(&self, index: u8) -> Option<&TargetString> {
        self.strings.iter().find(|string| string.index == index)
    }

    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    pub fn lowest_frequency_hz(&self) -> f32 {
        self.strings
            .first()
            .map_or(0.0, |string| string.frequency_hz)
    }

    pub fn highest_frequency_hz(&self) -> f32 {
        self.strings
            .last()
            .map_or(0.0, |string| string.frequency_hz)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresetMatch {
    pub preset_id: PresetId,
    pub matched_string: TargetString,
    pub cents_from_target: f32,
    pub absolute_cents: f32,
}

const fn string(
    index: u8,
    display_number: u8,
    label: &'static str,
    midi: i32,
    frequency_hz: f32,
) -> TargetString {
    TargetString {
        index,
        display_number,
        label,
        note: Note::from_midi(midi),
        frequency_hz,
    }
}

const E_STANDARD_STRINGS: [TargetString; 6] = [
    string(0, 6, "E2", 40, 82.41),
    string(1, 5, "A2", 45, 110.00),
    string(2, 4, "D3", 50, 146.83),
    string(3, 3, "G3", 55, 196.00),
    string(4, 2, "B3", 59, 246.94),
    string(5, 1, "E4", 64, 329.63),
];

const DROP_D_STRINGS: [TargetString; 6] = [
    string(0, 6, "D2", 38, 73.42),
    string(1, 5, "A2", 45, 110.00),
    string(2, 4, "D3", 50, 146.83),
    string(3, 3, "G3", 55, 196.00),
    string(4, 2, "B3", 59, 246.94),
    string(5, 1, "E4", 64, 329.63),
];

const EB_STANDARD_STRINGS: [TargetString; 6] = [
    string(0, 6, "Eb2", 39, 77.78),
    string(1, 5, "Ab2", 44, 103.83),
    string(2, 4, "Db3", 49, 138.59),
    string(3, 3, "Gb3", 54, 185.00),
    string(4, 2, "Bb3", 58, 233.08),
    string(5, 1, "Eb4", 63, 311.13),
];

const D_STANDARD_STRINGS: [TargetString; 6] = [
    string(0, 6, "D2", 38, 73.42),
    string(1, 5, "G2", 43, 98.00),
    string(2, 4, "C3", 48, 130.81),
    string(3, 3, "F3", 53, 174.61),
    string(4, 2, "A3", 57, 220.00),
    string(5, 1, "D4", 62, 293.66),
];

const DROP_C_STRINGS: [TargetString; 6] = [
    string(0, 6, "C2", 36, 65.41),
    string(1, 5, "G2", 43, 98.00),
    string(2, 4, "C3", 48, 130.81),
    string(3, 3, "F3", 53, 174.61),
    string(4, 2, "A3", 57, 220.00),
    string(5, 1, "D4", 62, 293.66),
];

const OPEN_G_STRINGS: [TargetString; 6] = [
    string(0, 6, "D2", 38, 73.42),
    string(1, 5, "G2", 43, 98.00),
    string(2, 4, "D3", 50, 146.83),
    string(3, 3, "G3", 55, 196.00),
    string(4, 2, "B3", 59, 246.94),
    string(5, 1, "D4", 62, 293.66),
];

const OPEN_D_STRINGS: [TargetString; 6] = [
    string(0, 6, "D2", 38, 73.42),
    string(1, 5, "A2", 45, 110.00),
    string(2, 4, "D3", 50, 146.83),
    string(3, 3, "F#3", 54, 185.00),
    string(4, 2, "A3", 57, 220.00),
    string(5, 1, "D4", 62, 293.66),
];

const DADGAD_STRINGS: [TargetString; 6] = [
    string(0, 6, "D2", 38, 73.42),
    string(1, 5, "A2", 45, 110.00),
    string(2, 4, "D3", 50, 146.83),
    string(3, 3, "G3", 55, 196.00),
    string(4, 2, "A3", 57, 220.00),
    string(5, 1, "D4", 62, 293.66),
];

const B_STANDARD_7_STRINGS: [TargetString; 7] = [
    string(0, 7, "B1", 35, 61.74),
    string(1, 6, "E2", 40, 82.41),
    string(2, 5, "A2", 45, 110.00),
    string(3, 4, "D3", 50, 146.83),
    string(4, 3, "G3", 55, 196.00),
    string(5, 2, "B3", 59, 246.94),
    string(6, 1, "E4", 64, 329.63),
];

const DROP_A_7_STRINGS: [TargetString; 7] = [
    string(0, 7, "A1", 33, 55.00),
    string(1, 6, "E2", 40, 82.41),
    string(2, 5, "A2", 45, 110.00),
    string(3, 4, "D3", 50, 146.83),
    string(4, 3, "G3", 55, 196.00),
    string(5, 2, "B3", 59, 246.94),
    string(6, 1, "E4", 64, 329.63),
];

const A_STANDARD_7_STRINGS: [TargetString; 7] = [
    string(0, 7, "A1", 33, 55.00),
    string(1, 6, "D2", 38, 73.42),
    string(2, 5, "G2", 43, 98.00),
    string(3, 4, "C3", 48, 130.81),
    string(4, 3, "F3", 53, 174.61),
    string(5, 2, "A3", 57, 220.00),
    string(6, 1, "D4", 62, 293.66),
];

const FSHARP_STANDARD_8_STRINGS: [TargetString; 8] = [
    string(0, 8, "F#1", 30, 46.25),
    string(1, 7, "B1", 35, 61.74),
    string(2, 6, "E2", 40, 82.41),
    string(3, 5, "A2", 45, 110.00),
    string(4, 4, "D3", 50, 146.83),
    string(5, 3, "G3", 55, 196.00),
    string(6, 2, "B3", 59, 246.94),
    string(7, 1, "E4", 64, 329.63),
];

const E_STANDARD_8_STRINGS: [TargetString; 8] = [
    string(0, 8, "E1", 28, 41.20),
    string(1, 7, "A1", 33, 55.00),
    string(2, 6, "D2", 38, 73.42),
    string(3, 5, "G2", 43, 98.00),
    string(4, 4, "C3", 48, 130.81),
    string(5, 3, "F3", 53, 174.61),
    string(6, 2, "A3", 57, 220.00),
    string(7, 1, "D4", 62, 293.66),
];

const CSHARP_STANDARD_9_STRINGS: [TargetString; 9] = [
    string(0, 9, "C#1", 25, 34.65),
    string(1, 8, "F#1", 30, 46.25),
    string(2, 7, "B1", 35, 61.74),
    string(3, 6, "E2", 40, 82.41),
    string(4, 5, "A2", 45, 110.00),
    string(5, 4, "D3", 50, 146.83),
    string(6, 3, "G3", 55, 196.00),
    string(7, 2, "B3", 59, 246.94),
    string(8, 1, "E4", 64, 329.63),
];

const DROP_B_9_STRINGS: [TargetString; 9] = [
    string(0, 9, "B0", 23, 30.87),
    string(1, 8, "F#1", 30, 46.25),
    string(2, 7, "B1", 35, 61.74),
    string(3, 6, "E2", 40, 82.41),
    string(4, 5, "A2", 45, 110.00),
    string(5, 4, "D3", 50, 146.83),
    string(6, 3, "G3", 55, 196.00),
    string(7, 2, "B3", 59, 246.94),
    string(8, 1, "E4", 64, 329.63),
];

pub const E_STANDARD: TuningPreset = TuningPreset {
    id: PresetId::EStandard,
    display_name: "E Standard",
    strings: &E_STANDARD_STRINGS,
};

pub const DROP_D: TuningPreset = TuningPreset {
    id: PresetId::DropD,
    display_name: "Drop D",
    strings: &DROP_D_STRINGS,
};

pub const EB_STANDARD: TuningPreset = TuningPreset {
    id: PresetId::EbStandard,
    display_name: "Eb Standard",
    strings: &EB_STANDARD_STRINGS,
};

pub const D_STANDARD: TuningPreset = TuningPreset {
    id: PresetId::DStandard,
    display_name: "D Standard",
    strings: &D_STANDARD_STRINGS,
};

pub const DROP_C: TuningPreset = TuningPreset {
    id: PresetId::DropC,
    display_name: "Drop C",
    strings: &DROP_C_STRINGS,
};

pub const OPEN_G: TuningPreset = TuningPreset {
    id: PresetId::OpenG,
    display_name: "Open G",
    strings: &OPEN_G_STRINGS,
};

pub const OPEN_D: TuningPreset = TuningPreset {
    id: PresetId::OpenD,
    display_name: "Open D",
    strings: &OPEN_D_STRINGS,
};

pub const DADGAD: TuningPreset = TuningPreset {
    id: PresetId::Dadgad,
    display_name: "DADGAD",
    strings: &DADGAD_STRINGS,
};

pub const B_STANDARD_7: TuningPreset = TuningPreset {
    id: PresetId::BStandard7,
    display_name: "B Standard (7)",
    strings: &B_STANDARD_7_STRINGS,
};

pub const DROP_A_7: TuningPreset = TuningPreset {
    id: PresetId::DropA7,
    display_name: "Drop A (7)",
    strings: &DROP_A_7_STRINGS,
};

pub const A_STANDARD_7: TuningPreset = TuningPreset {
    id: PresetId::AStandard7,
    display_name: "A Standard (7)",
    strings: &A_STANDARD_7_STRINGS,
};

pub const FSHARP_STANDARD_8: TuningPreset = TuningPreset {
    id: PresetId::FSharpStandard8,
    display_name: "F# Standard (8)",
    strings: &FSHARP_STANDARD_8_STRINGS,
};

pub const E_STANDARD_8: TuningPreset = TuningPreset {
    id: PresetId::EStandard8,
    display_name: "E Standard (8)",
    strings: &E_STANDARD_8_STRINGS,
};

pub const CSHARP_STANDARD_9: TuningPreset = TuningPreset {
    id: PresetId::CSharpStandard9,
    display_name: "C# Standard (9)",
    strings: &CSHARP_STANDARD_9_STRINGS,
};

pub const DROP_B_9: TuningPreset = TuningPreset {
    id: PresetId::DropB9,
    display_name: "Drop B (9)",
    strings: &DROP_B_9_STRINGS,
};

pub const STANDARD_TUNING: [TargetString; 6] = E_STANDARD_STRINGS;

const ALL_PRESETS: [TuningPreset; 15] = [
    E_STANDARD,
    DROP_D,
    EB_STANDARD,
    D_STANDARD,
    DROP_C,
    OPEN_G,
    OPEN_D,
    DADGAD,
    B_STANDARD_7,
    DROP_A_7,
    A_STANDARD_7,
    FSHARP_STANDARD_8,
    E_STANDARD_8,
    CSHARP_STANDARD_9,
    DROP_B_9,
];

pub fn all_presets() -> &'static [TuningPreset] {
    &ALL_PRESETS
}

pub fn default_preset() -> &'static TuningPreset {
    &E_STANDARD
}

pub fn preset_by_id(id: PresetId) -> &'static TuningPreset {
    match id {
        PresetId::EStandard => &E_STANDARD,
        PresetId::DropD => &DROP_D,
        PresetId::EbStandard => &EB_STANDARD,
        PresetId::DStandard => &D_STANDARD,
        PresetId::DropC => &DROP_C,
        PresetId::OpenG => &OPEN_G,
        PresetId::OpenD => &OPEN_D,
        PresetId::Dadgad => &DADGAD,
        PresetId::BStandard7 => &B_STANDARD_7,
        PresetId::DropA7 => &DROP_A_7,
        PresetId::AStandard7 => &A_STANDARD_7,
        PresetId::FSharpStandard8 => &FSHARP_STANDARD_8,
        PresetId::EStandard8 => &E_STANDARD_8,
        PresetId::CSharpStandard9 => &CSHARP_STANDARD_9,
        PresetId::DropB9 => &DROP_B_9,
    }
}

pub fn match_frequency_to_preset(
    measured_hz: f32,
    preset: &TuningPreset,
    max_distance_cents: f32,
) -> Option<PresetMatch> {
    let mut best_match: Option<PresetMatch> = None;

    for &target_string in preset.strings {
        let cents_from_target = Cents::between(measured_hz, target_string.frequency_hz)?.value();
        let absolute_cents = cents_from_target.abs();

        let candidate = PresetMatch {
            preset_id: preset.id,
            matched_string: target_string,
            cents_from_target,
            absolute_cents,
        };

        let is_better = best_match
            .as_ref()
            .map(|current| candidate.absolute_cents < current.absolute_cents)
            .unwrap_or(true);

        if is_better {
            best_match = Some(candidate);
        }
    }

    match best_match {
        Some(candidate) if candidate.absolute_cents <= max_distance_cents => Some(candidate),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{PresetId, all_presets, default_preset, match_frequency_to_preset, preset_by_id};

    #[test]
    fn parses_known_preset_ids() {
        assert_eq!(PresetId::parse("e-standard"), Some(PresetId::EStandard));
        assert_eq!(PresetId::parse("drop-d"), Some(PresetId::DropD));
        assert_eq!(PresetId::parse("eb-standard"), Some(PresetId::EbStandard));
        assert_eq!(PresetId::parse("d-standard"), Some(PresetId::DStandard));
        assert_eq!(PresetId::parse("drop-c"), Some(PresetId::DropC));
        assert_eq!(PresetId::parse("open-g"), Some(PresetId::OpenG));
        assert_eq!(PresetId::parse("open-d"), Some(PresetId::OpenD));
        assert_eq!(PresetId::parse("dadgad"), Some(PresetId::Dadgad));
        assert_eq!(PresetId::parse("b-standard"), Some(PresetId::BStandard7));
        assert_eq!(PresetId::parse("drop-a"), Some(PresetId::DropA7));
        assert_eq!(PresetId::parse("a-standard"), Some(PresetId::AStandard7));
        assert_eq!(
            PresetId::parse("fsharp-standard"),
            Some(PresetId::FSharpStandard8)
        );
        assert_eq!(PresetId::parse("e-standard-8"), Some(PresetId::EStandard8));
        assert_eq!(
            PresetId::parse("csharp-standard-9"),
            Some(PresetId::CSharpStandard9)
        );
        assert_eq!(PresetId::parse("drop-b-9"), Some(PresetId::DropB9));
        assert_eq!(PresetId::parse("unknown"), None);
    }

    #[test]
    fn returns_default_preset() {
        assert_eq!(default_preset().id, PresetId::EStandard);
        assert_eq!(preset_by_id(PresetId::DropD).display_name, "Drop D");
        assert_eq!(preset_by_id(PresetId::DropC).display_name, "Drop C");
    }

    #[test]
    fn exposes_all_presets() {
        assert_eq!(all_presets().len(), 15);
    }

    #[test]
    fn matches_frequency_to_closest_string() {
        let preset = preset_by_id(PresetId::EStandard);
        let matched = match_frequency_to_preset(109.8, preset, 100.0).unwrap();

        assert_eq!(matched.matched_string.label, "A2");
        assert!(matched.absolute_cents < 10.0);
    }

    #[test]
    fn matches_drop_c_low_string() {
        let preset = preset_by_id(PresetId::DropC);
        let matched = match_frequency_to_preset(65.5, preset, 100.0).unwrap();

        assert_eq!(matched.matched_string.label, "C2");
    }

    #[test]
    fn matches_low_b_for_7_string_preset() {
        let preset = preset_by_id(PresetId::BStandard7);
        let matched = match_frequency_to_preset(61.7, preset, 100.0).unwrap();

        assert_eq!(matched.matched_string.label, "B1");
    }

    #[test]
    fn rejects_frequency_outside_match_window() {
        let preset = preset_by_id(PresetId::EStandard);
        assert!(match_frequency_to_preset(98.0, preset, 20.0).is_none());
    }
}
