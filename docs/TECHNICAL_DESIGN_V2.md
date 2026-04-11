# TECHNICAL DESIGN — V0.2

## 1. Objectif

Traduire [SPEC_V2.md](./SPEC_V2.md) en un design technique Rust directement implémentable pour :

- modes `Chromatic` et `Preset`
- sélection CLI du mode et du preset initiaux
- sortie moteur unifiée
- API de presets stable
- splashscreen terminal de démarrage
- séparation nette entre `core`, `dsp` et `cli`

---

## 2. Principes de conception

- `dsp` produit une détection fréquentielle stabilisée, sans logique produit
- `core` porte les types métier, les presets, le matching et les calculs musicaux
- `cli` parse les commandes, construit la config runtime, gère les raccourcis live, affiche le splashscreen et rend la TUI
- `TunerEngine` orchestre la projection `PitchDetection -> TunerOutput`
- la TUI ne recalcule ni note, ni cible, ni corde probable
- le moteur doit utiliser le sample rate réel du périphérique audio actif
- le preset actif et la calibration A4 doivent être configurables via un contrat DSP partagé

---

## 3. Découpage par crate

### `tuner-core`

Responsable de :

- notes, midi, fréquences, cents
- `TunerMode`
- `PresetId`, `TuningPreset`, `TargetString`
- matching fréquence -> corde cible
- `TunerOutput` et états UI partagés

### `tuner-dsp-algo`

Responsable de :

- capture audio
- fenêtrage
- NSDF / MPM
- interpolation parabolique
- estimation amplitude / clarté
- stabilisation de la fréquence publiée
- remontée du sample rate réel via `AudioStart`

### `tuner-cli`

Responsable de :

- parsing CLI manuel
- mapping CLI -> config initiale de session
- chargement du preset demandé
- splashscreen de démarrage
- boucle live
- gestion des changements de mode/preset à chaud
- projection `TunerOutput` -> view model TUI

---

## 4. Types métier Rust

### 4.1 Identifiants et modes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunerMode {
    Chromatic,
    Preset(PresetId),
}

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
}

impl PresetId {
    pub const ALL: [PresetId; 8] = [
        PresetId::EStandard,
        PresetId::DropD,
        PresetId::EbStandard,
        PresetId::DStandard,
        PresetId::DropC,
        PresetId::OpenG,
        PresetId::OpenD,
        PresetId::Dadgad,
    ];

    pub fn as_str(self) -> &'static str;
    pub fn display_name(self) -> &'static str;
    pub fn parse(value: &str) -> Option<Self>;
}
```

### 4.2 Notes et calculs

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasuredPitch {
    pub frequency_hz: f32,
    pub confidence: f32,
    pub clarity: f32,
    pub rms: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteEstimate {
    pub note: Note,
    pub note_name: String,
    pub midi: i32,
    pub target_frequency_hz: f32,
    pub cents_offset: f32,
}
```

### 4.3 Presets et cordes

Convention :

- index interne `0..=5` du grave vers l'aigu
- la corde 6 correspond à `index = 0`
- la corde 1 correspond à `index = 5`

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetString {
    pub index: u8,
    pub display_number: u8,
    pub label: &'static str,
    pub note: Note,
    pub frequency_hz: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningPreset {
    pub id: PresetId,
    pub display_name: &'static str,
    pub strings: [TargetString; 6],
}
```

Presets statiques :

```rust
pub const E_STANDARD: TuningPreset = /* ... */;
pub const DROP_D: TuningPreset = /* ... */;
pub const EB_STANDARD: TuningPreset = /* ... */;
pub const D_STANDARD: TuningPreset = /* ... */;
pub const DROP_C: TuningPreset = /* ... */;
pub const OPEN_G: TuningPreset = /* ... */;
pub const OPEN_D: TuningPreset = /* ... */;
pub const DADGAD: TuningPreset = /* ... */;
```

### 4.4 Matching preset

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresetMatch {
    pub preset_id: PresetId,
    pub matched_string: TargetString,
    pub cents_from_target: f32,
    pub absolute_cents: f32,
}
```

API minimale :

```rust
pub fn all_presets() -> &'static [TuningPreset];
pub fn preset_by_id(id: PresetId) -> &'static TuningPreset;
pub fn default_preset() -> &'static TuningPreset;

pub fn match_frequency_to_preset(
    measured_hz: f32,
    preset: &TuningPreset,
    max_distance_cents: f32,
) -> Option<PresetMatch>;
```

Règles :

- comparer en cents à chaque corde du preset
- retenir la distance absolue minimale
- retourner `None` si la distance dépasse `max_distance_cents`
- ne jamais déduire la corde depuis la seule note arrondie
- couvrir au minimum les presets `E Standard`, `Drop D`, `Eb Standard`, `D Standard`, `Drop C`, `Open G`, `Open D` et `DADGAD`

---

## 5. États UI partagés

Les états de rendu ne doivent pas vivre uniquement dans la TUI.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    NoSignal,
    Searching,
    Unstable,
    TooLow,
    InTune,
    TooHigh,
}
```

Règle de calcul :

- `NoSignal` si RMS trop faible ou aucune fréquence exploitable
- `Searching` si fréquence présente mais cible pas encore stabilisée
- `Unstable` si fréquence ou matching plausibles mais confiance ou stabilité insuffisante
- `TooLow`, `InTune`, `TooHigh` seulement si la cible affichée est fiable

---

## 6. Structure `TunerOutput`

La sortie moteur doit être unique et indépendante du rendu.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TunerOutput {
    pub mode: TunerMode,
    pub measured_frequency_hz: Option<f32>,
    pub confidence: f32,
    pub detected_note: Option<NoteEstimate>,
    pub display_cents: Option<f32>,
    pub target: Option<TuningTarget>,
    pub ui_state: UiState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuningTarget {
    pub note_name: String,
    pub frequency_hz: f32,
    pub preset_id: Option<PresetId>,
    pub string: Option<TargetString>,
}
```

Interprétation :

- en `Chromatic`, `target` correspond à la note tempérée la plus proche
- en `Preset`, `target` correspond à la corde retenue dans le preset actif
- `display_cents` vaut l'écart à la `target` affichée
- `detected_note` reste la meilleure note tempérée liée à la fréquence mesurée, même en mode `Preset`

Ce format évite un `enum TunerOutput` dépendant de la TUI et reste compatible avec :

- rendu TUI
- sortie texte
- tests unitaires
- journalisation

---

## 7. Configuration moteur

```rust
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
```

Valeurs par défaut recommandées :

```rust
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
```

---

## 8. `TunerEngine`

```rust
pub struct TunerEngine {
    detector: PitchDetector,
    config: TunerConfig,
    last_preset_id: PresetId,
    published_frequencies_hz: VecDeque<f32>,
    last_note: Option<NoteEstimate>,
    stable_pitch_count: usize,
    stable_string_count: usize,
    last_matched_string_index: Option<u8>,
}
```

API :

```rust
impl TunerEngine {
    pub fn new(detector_config: PitchDetectorConfig, config: TunerConfig) -> Self;

    pub fn session_mode(&self) -> SessionMode;

    pub fn set_session_mode(&mut self, mode: SessionMode);

    pub fn toggle_analyze_mode(&mut self) -> Option<SessionMode>;

    pub fn select_preset(&mut self, preset_id: PresetId) -> PresetId;

    pub fn process_frame(&mut self, frame: &[f32]) -> TunerOutput;

    pub fn process_detection(&mut self, detection: Option<MeasuredPitch>) -> TunerOutput;
}
```

Algorithme de haut niveau :

1. si `detection` est absente ou sous seuil RMS, publier `NoSignal`
2. si la clarté est trop faible, publier `Searching`
3. convertir la fréquence en `NoteEstimate`
4. selon `config.session_mode` :
   - `Chromatic` : cibler la note tempérée la plus proche
   - `Preset` : charger le preset et calculer le `PresetMatch`
   - `Target` : comparer à la note demandée
5. appliquer hystérésis et stabilité multi-frames
6. calculer `UiState`
7. publier un `TunerOutput` complet

Règles lors d'un changement à chaud :

- un changement de mode ou de preset doit réinitialiser l'historique de stabilité moteur
- si le système passe de `Chromatic` à `Preset`, il réutilise le dernier `PresetId` actif ou le preset par défaut
- si le système passe de `Preset` à `Chromatic`, aucun preset n'est perdu, il reste mémorisé pour un retour ultérieur

---

## 9. CLI concrète

La spec V0.2 impose un choix initial possible au lancement de `analyze`, puis un changement à chaud dans la TUI.

### 9.1 Types de parsing CLI

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalyzeMode {
    Chromatic,
    Preset,
}

#[derive(Debug, Clone, PartialEq)]
enum Command {
    Strings { preset: PresetId, a4_hz: f32 },
    Analyze { mode: AnalyzeMode, preset: Option<PresetId>, a4_hz: f32 },
    Tune { note: Note },
    Demo,
    Help,
}
```

### 9.2 Fonctions de parsing

```rust
fn parse_cli(args: &[String]) -> Result<Command, String>;
fn parse_strings(args: &[String]) -> Result<Command, String>;
fn parse_analyze(args: &[String]) -> Result<Command, String>;
fn parse_tune(args: &[String]) -> Result<Command, String>;
```

### 9.3 Règles CLI

- `analyze --mode chromatic` ignore `preset`
- `analyze --mode preset --preset e-standard` est valide
- `analyze --mode preset` sans `--preset` utilise `E Standard` par défaut
- `analyze --a4 432` applique une calibration runtime A4=432 Hz
- `strings --preset drop-d` affiche les 6 cordes du preset choisi
- `strings --preset drop-d --a4 442` affiche les fréquences des cordes recalibrées
- `tune <NOTE>` reste séparé de `TunerMode`
- en l'absence de commande, le comportement actuel est d'afficher `strings` sur le preset par défaut

La CLI définit seulement l'état initial de la session live. Après démarrage :

- `m` bascule entre `Chromatic` et `Preset`
- `p` ouvre le sélecteur de presets
- `↑` et `↓` naviguent dans la liste
- `Enter` applique le preset sélectionné
- `Esc` ferme le sélecteur ou quitte selon le contexte
- `h` et `?` affichent ou masquent l'aide

---

## 10. API presets

Module recommandé : `tuner_core::tuning`

```rust
pub fn all_presets() -> &'static [TuningPreset];
pub fn preset_by_id(id: PresetId) -> &'static TuningPreset;
pub fn default_preset() -> &'static TuningPreset;
```

Helpers utiles :

```rust
impl TuningPreset {
    pub fn string_by_index(&self, index: u8) -> Option<&TargetString>;
    pub fn lowest_frequency_hz(&self) -> f32;
    pub fn highest_frequency_hz(&self) -> f32;
}
```

Objectifs :

- éviter toute allocation pour les presets intégrés
- garantir un ordre stable des cordes
- simplifier tests et affichage CLI

---

## 10.b Contrat tuning cross-platform

Le mapping `frequency -> note_name/cents/ui_state` doit être cohérent entre `web`, `native` et `embedded`.

### Web (`tuner-dsp-web`)

APIs detector-level exposées via wasm :

```rust
pub fn set_preset(detector_id: u32, preset_id: &str) -> bool;
pub fn set_calibration_hz(detector_id: u32, calibration_hz: f32) -> bool;
```

### Native / Embedded (`tuner-dsp-native`, `tuner-dsp-embedded`)

API session-level :

```rust
pub struct TuningSession { /* ... */ }

impl TuningSession {
    pub fn set_preset(&mut self, preset_id: &str) -> bool;
    pub fn set_calibration_hz(&mut self, calibration_hz: f32) -> bool;
    pub fn map_detection(&self, detection: PitchDetectionResult) -> MappedDetection;
}
```

Règle : pour un même preset et une même calibration, les clients doivent afficher des valeurs de note/cents alignées.

---

## 11. Projection TUI

La TUI doit consommer `TunerOutput` et non la logique moteur brute.

```rust
pub struct TunerUiState {
    pub profile_label: String,
    pub mode_label: String,
    pub note_label: String,
    pub target_label: String,
    pub string_label: Option<String>,
    pub frequency_hz: Option<f32>,
    pub target_hz: Option<f32>,
    pub cents_off: f32,
    pub confidence: Option<f32>,
    pub sample_rate_hz: u32,
    pub overlay_notice: Option<String>,
    pub help_visible: bool,
    pub preset_picker_visible: bool,
    pub preset_picker_items: Vec<(PresetId, String)>,
    pub preset_picker_selected: usize,
}
```

Règles :

- si `ui_state` vaut `NoSignal`, `Searching` ou `Unstable`, la TUI ne doit pas forcer un état `TooLow/TooHigh/InTune`
- `string_label` est dérivé au format compact `6D2`, `5A2`, etc.
- l'overlay doit rendre visibles les changements de mode et de preset sans interrompre la mesure

État de sélection de preset requis :

- `p` ouvre la liste sur le preset courant
- `↑` et `↓` déplacent la sélection
- `Enter` applique et ferme
- `Esc` ferme sans appliquer

---

## 12. Splashscreen

La CLI démarre sur un splashscreen terminal avant d'entrer dans la commande principale.

```rust
pub enum SplashOutcome {
    Finished,
    Aborted,
}

pub struct SplashScreen {
    started_at: Instant,
    finished: bool,
    handoff_started_at: Option<Instant>,
}

impl SplashScreen {
    pub fn new() -> Self;
    pub fn tick(&mut self);
    pub fn is_finished(&self) -> bool;
}
```

Contraintes :

- rendu `boot system`
- checks progressifs `KERNEL CLOCK`, `TERM LINK`, `AUDIO LINK`, `JACK DETECTION`, `DSP ENGINE`, `NOISE FILTER`
- bloc `SIGNAL SNAPSHOT` simulé
- état final `STATUS :: READY`
- handoff court vers la TUI principale
- interruption possible avec `q` ou `Esc`

---

## 13. Sample rate runtime

Le moteur ne doit pas supposer un sample rate fixe de `44_100 Hz`.

Règles :

- `AudioCapture` démarre d'abord le flux `cpal`
- le sample rate réel négocié est remonté via `AudioStart.sample_rate_hz`
- `PitchDetectorConfig.sample_rate` est réécrit avec cette valeur avant la construction de `TunerEngine`
- le même sample rate réel est propagé à la TUI
- la source mock utilise aussi ce sample rate pour sa cadence

Cette contrainte est nécessaire pour éviter une détection de note incorrecte lors d'un changement de carte son ou de périphérique d'entrée.

---

## 14. Tests recommandés

### `core`

- conversion fréquence -> note sur les 6 cordes standard
- calcul des cents
- lookup de preset
- matching de fréquence vers la bonne corde
- rejet hors fenêtre de matching

### `cli/app`

- `Analyze` mappe correctement vers `TunerMode`
- `Preset` charge le bon preset
- parsing de `--a4` sur `strings` et `analyze`
- `m` bascule correctement entre `Chromatic` et `Preset`
- la sélection de preset applique le bon `PresetId`
- `TunerEngine` publie `NoSignal` si entrée absente
- `TunerEngine` publie `InTune` si fréquence cible stable
- `TunerEngine` garde `Unstable` avant validation de corde
- le moteur utilise le sample rate réel du flux audio
- la calibration A4 runtime est propagée à la couche de mapping DSP

---

## 15. Décisions d'implémentation à garder fixes

- `TunerOutput` reste un `struct` unifié, pas un `enum`
- `PresetId` est un enum fermé pour V0.2
- `tune <NOTE>` ne fusionne pas avec `analyze`
- les presets intégrés sont des constantes statiques
- la TUI n'invente pas de logique de matching ou de priorité d'état
- le sample rate réel du périphérique actif est source de vérité pour la détection live
