# SPEC — Accordeur guitare simple en Rust — V0.1 livrée

## 1. Contexte

L’objectif du projet est de développer une application simple d’accordeur de guitare en Rust.

Cette itération livre déjà une chaîne complète utilisable :

- capture micro temps réel
- détection de pitch monophonique pour guitare
- conversion fréquence -> note -> cents
- publication stabilisée via `TunerEngine`
- TUI terminal cyberpunk modulaire pour le mode live et un mode démo

Le projet cible une guitare standard 6 cordes en accordage standard :

- E2 = 82.41 Hz
- A2 = 110.00 Hz
- D3 = 146.83 Hz
- G3 = 196.00 Hz
- B3 = 246.94 Hz
- E4 = 329.63 Hz

---

## 2. Objectif fonctionnel V0.1

Implémenter un pipeline temps réel capable de :

- capturer un signal audio mono depuis le micro
- détecter la fréquence fondamentale d’une corde de guitare jouée seule
- convertir cette fréquence en :
  - note la plus proche
  - fréquence cible
  - écart en cents
  - indicateur de confiance
- fournir un résultat suffisamment stable pour alimenter une UI d’accordeur
- exposer ce résultat via une CLI simple et une TUI terminal lisible

---

## 3. Choix technique validé

La méthode de détection retenue est :

- **analyse de périodicité temporelle**
- basée sur **autocorrélation normalisée de type MPM / NSDF**
- avec :
  - **filtrage des faux pics**
  - **interpolation parabolique**
  - **stabilisation temporelle**

Ce choix remplace une FFT simple comme méthode principale.

---

## 4. Périmètre V0.1

### Inclus

- capture audio micro temps réel
- traitement mono
- bufferisation
- prétraitement léger
- détection de pitch par MPM/NSDF
- conversion pitch -> note musicale
- sortie exploitable par UI/CLI
- commande `strings`
- commande `analyze` live chromatique
- commande `tune <NOTE>` live mono-cible
- commande `demo`
- TUI terminal modulaire avec états explicites `NoSignal`, `Searching`, `Unstable`, `TooLow`, `TooHigh`, `InTune`
- tests unitaires sur la couche musicale et DSP de base

### Exclus

- reconnaissance polyphonique
- gestion de plusieurs instruments
- accordages alternatifs côté CLI produit
- visualisation avancée
- calibration utilisateur de A4
- suppression de bruit avancée / ML
- pitch shifting
- enregistrement audio
- support mobile

---

## 5. Contraintes produit

- application simple
- faible latence perçue
- comportement stable
- fonctionnement correct sur une guitare seule, une corde à la fois
- robustesse suffisante pour les fréquences entre **70 Hz et 400 Hz**
- code modulaire, lisible, testable

---

## 6. Exigences de précision

Le système doit viser :

- détection correcte de la bonne note sur les 6 cordes standard
- précision d’affichage suffisamment fine pour un accordeur
- erreur cible :
  - idéalement < ±3 cents sur signal propre stabilisé
  - tolérance acceptable V1 : < ±5 cents sur cas simples
- faible oscillation visuelle de la note affichée

Ce sont des objectifs de qualité, pas des garanties absolues sur tout environnement micro.

---

## 7. Hypothèses d’entrée

- source : microphone
- signal : monophonique
- une seule corde jouée à la fois
- environnement raisonnablement calme
- fréquence d’échantillonnage :
  - préférée : `44_100 Hz`
  - acceptable : `48_000 Hz`
- format interne : `f32`

---

## 8. Pipeline DSP V0.1

### 8.1 Capture audio

Le module audio doit :

- capturer le flux micro
- convertir vers mono si nécessaire
- pousser les échantillons dans un buffer circulaire
- ne pas faire de calcul DSP lourd dans le callback audio

### 8.2 Fenêtre d’analyse

Utiliser une fenêtre glissante de taille :

- valeur par défaut : `4096` samples
- option future : `2048` pour réduire la latence

Justification :

- 4096 améliore la robustesse sur la corde grave E2

À `44_100 Hz`, 4096 samples ≈ 92.9 ms.

### 8.3 Pas d’analyse

Le recalcul ne doit pas attendre un buffer entièrement nouveau.

Utiliser un hop size :

- recommandé : `512` ou `1024` samples

Objectif :

- mise à jour plus fluide sans coût excessif

### 8.4 Prétraitement

À appliquer avant la détection :

1. suppression DC offset
2. normalisation légère ou contrôle d’amplitude
3. rejet des buffers trop faibles
4. optionnel si simple : fenêtre Hann avant analyse

Ne pas sur-complexifier le filtrage en V0.1.

### 8.5 Plage de recherche

Limiter la recherche de période à la plage :

- `f_min = 70.0 Hz`
- `f_max = 400.0 Hz`

En nombre de samples :

```text
τ_min = floor(fs / f_max)
τ_max = ceil(fs / f_min)
```

Exemple à 44.1 kHz :

- tau_min ≈ 110
- tau_max ≈ 630

---

## 9. Algorithme de détection

### 9.1 Méthode retenue

Implémenter une version simple et lisible de :

- **MPM / McLeod Pitch Method**
- utilisant **NSDF** (Normalized Square Difference Function)

Formule attendue au niveau conceptuel :

Pour chaque décalage `tau`, calculer une mesure de similarité normalisée entre le signal et sa version retardée.

L’agent n’a pas besoin de reproduire une publication académique mot pour mot, mais doit respecter l’idée suivante :

- score élevé quand le signal est périodique avec période `tau`
- normalisation pour rendre les pics plus fiables qu’en autocorrélation brute

### 9.2 Étapes algorithmiques

Pour chaque fenêtre :

1. calculer la fonction NSDF pour `tau in [tau_min, tau_max]`
2. détecter les maxima locaux positifs
3. rejeter les pics faibles ou non plausibles
4. sélectionner le pic fondamental le plus crédible
5. raffiner sa position par interpolation parabolique
6. convertir `tau` raffiné en fréquence
7. calculer un score de confiance
8. lisser le résultat dans le temps

### 9.3 Filtrage des faux pics

Le filtrage doit au minimum inclure :

- ne considérer que les pics locaux positifs
- seuil minimum sur la hauteur du pic NSDF
- exclusion des pics hors plage fréquentielle
- rejet si le buffer a une énergie trop faible
- rejet si aucun pic crédible n’est trouvé

Règles initiales recommandées :

- `min_rms` configurable
- `min_peak_clarity` configurable, ex. `0.6` à ajuster expérimentalement

### 9.4 Sélection du pic

Stratégie retenue :

- prendre le meilleur pic crédible dans la plage utile
- ou le premier maximum significatif après passage positif robuste, selon la stabilité observée

L’agent doit privilégier :

- lisibilité
- facilité de réglage
- robustesse sur guitare réelle

### 9.5 Interpolation parabolique

Autour de l’indice du maximum `i`, utiliser `i-1`, `i`, `i+1` pour estimer un maximum sous-échantillon.

Objectif :

- obtenir `tau_refined` non entier
- améliorer la précision du calcul de fréquence
- réduire les sauts en cents

Formule attendue : interpolation quadratique/parabolique standard autour du pic local.

### 9.6 Conversion période -> fréquence

```text
f0 = fs / tau_refined
```

---

## 10. Conversion fréquence -> note

### 10.1 Note MIDI

```text
midi = 69 + 12 * log2(f / 440.0)
```

### 10.2 Note la plus proche

- `nearest_midi = round(midi)`

### 10.3 Fréquence théorique de la note cible

```text
f_target = 440.0 * 2^((nearest_midi - 69) / 12)
```

### 10.4 Écart en cents

```text
cents = 1200 * log2(f / f_target)
```

### 10.5 Nom de note

Mapper le numéro MIDI vers :

- C
- C#
- D
- D#
- E
- F
- F#
- G
- G#
- A
- A#
- B

avec octave.

Exemple :

- MIDI 40 -> E2
- MIDI 45 -> A2
- MIDI 50 -> D3
- MIDI 55 -> G3
- MIDI 59 -> B3
- MIDI 64 -> E4

---

## 11. Stabilisation temporelle

Une fréquence brute varie trop pour une UI.

La version codée repose sur :

- smoothing fréquentiel dans `dsp`
- seuil de publication basé sur la confiance
- hystérésis de note dans `cli::app::TunerEngine`
- stabilité multi-frames publiée via `TunerReading::is_stable`

Paramètres actuellement utilisés :

- `smoothing_window_size = 3`
- minimum de `3` frames cohérentes pour publier un état stable
- hystérésis de note `35 cents`
- spread maximal de stabilité `10 cents`

Exigence :

- éviter que l’affichage alterne rapidement entre deux notes voisines

---

## 12. Format de sortie attendu

Le moteur de détection doit fournir une structure de sortie du type :

```rust
pub struct PitchDetectionResult {
    pub frequency_hz: f32,
    pub confidence: f32,
    pub clarity: f32,
    pub rms: f32,
}

pub struct NoteEstimate {
    pub note: Note,
    pub note_name: String,
    pub midi: i32,
    pub target_frequency_hz: f32,
    pub cents_offset: f32,
}

pub struct TunerReading {
    pub detected_pitch: Option<PitchDetectionResult>,
    pub note: Option<NoteEstimate>,
    pub is_stable: bool,
}
```

L’agent peut ajuster les noms, mais l’intention doit rester identique.

---

## 13. Architecture logicielle codée

Structure actuelle :

```text
core/
  src/
    cents.rs
    frequency.rs
    note.rs
    tuning.rs
dsp/
  src/
    audio.rs
    preprocess.rs
    nsdf.rs
    peak_detection.rs
    interpolate.rs
    pitch_detector.rs
    smoothing.rs
cli/
  src/
    main.rs
    app/
      mod.rs
      tuner_engine.rs
    tui/
      mod.rs
      demo.rs
      gauge.rs
      layout.rs
      render.rs
      state.rs
      theme.rs
      widgets/
        header.rs
        history.rs
        pitch_hero.rs
        telemetry.rs
        tune_gauge.rs
```

---

## 14. Responsabilités des modules

### `dsp::audio`

Responsable de :

- initialiser le périphérique d’entrée
- lire les buffers audio
- convertir vers `f32`
- alimenter des frames glissantes vers le thread de traitement

Crate attendue :

- `cpal`

### `dsp::preprocess`

Responsable de :

- DC offset removal
- calcul RMS
- éventuelle normalisation légère
- éventuelle fenêtre Hann

### `dsp::nsdf`

Responsable de :

- calcul de la fonction NSDF sur la fenêtre fournie

### `dsp::peak_detection`

Responsable de :

- détection des maxima locaux
- application des seuils
- sélection du meilleur pic candidat

### `dsp::interpolate`

Responsable de :

- interpolation parabolique du pic retenu

### `dsp::pitch_detector`

Responsable de :

- orchestration complète du pipeline DSP
- retour d’un `PitchDetectionResult`

### `dsp::smoothing`

Responsable de :

- stabilisation inter-frames
- rejet des outliers simples

### `core::note`

Responsable de :

- conversion fréquence <-> midi <-> note textuelle

### `core::tuning`

Responsable de :

- définition des notes de référence guitare standard

### `cli::app::tuner_engine`

Responsable de :

- coordonner pitch détecté, note estimée et publication stabilisée du résultat courant

### `cli::tui`

Responsable de :

- projeter `TunerReading` vers un état UI dédié
- afficher la TUI live et le mode démo
- séparer thème, layout et widgets

---

## 15. API interne exposée

Le dépôt expose une API simple de ce type :

```rust
pub struct PitchDetectorConfig {
    pub sample_rate: u32,
    pub frame_size: usize,
    pub hop_size: usize,
    pub min_frequency_hz: f32,
    pub max_frequency_hz: f32,
    pub min_rms: f32,
    pub min_clarity: f32,
}

pub struct PitchDetector {
    config: PitchDetectorConfig,
}

impl PitchDetector {
    pub fn new(config: PitchDetectorConfig) -> Self;
    pub fn detect_pitch(&self, frame: &[f32]) -> Option<PitchDetectionResult>;
}
```

Et une estimation musicale :

```rust
pub fn frequency_to_note(frequency_hz: f32) -> NoteEstimate;
```

Puis une orchestration applicative :

```rust
pub struct TunerEngine { /* ... */ }

impl TunerEngine {
    pub fn process_frame(&mut self, frame: &[f32]) -> TunerReading;
}
```

---

## 16. Dépendances retenues

- `cpal` pour l’audio
- `crossterm` pour le terminal interactif
- `ratatui` pour la TUI

La logique de pitch reste implémentée dans le projet.

---

## 17. Interface utilisateur V0.1

La V0.1 livrée fournit une TUI terminal. Elle doit afficher :

- fréquence détectée
- note détectée
- écart en cents
- cible active
- confiance / stabilité
- bruit / phase
- indicateur `too low / in tune / too high`
- états `NoSignal`, `Searching`, `Unstable`

Structure attendue :

```text
HEADER TECHNIQUE
PITCH//CORE
TELEMETRY
```

---

## 18. Gestion des cas invalides

Le système doit retourner `None` ou état invalide si :

- buffer trop faible
- aucun pic fiable
- fréquence hors plage
- résultat incohérent

Le système ne doit pas inventer une note sur du bruit faible.

---

## 19. Critères d’acceptation

L’implémentation sera considérée acceptable si :

1. le projet compile proprement
2. l’entrée micro fonctionne sur machine de dev standard si un périphérique d’entrée est disponible
3. une note jouée seule sur guitare renvoie une fréquence plausible
4. l’écart en cents évolue dans le bon sens si la corde est plus basse ou plus haute
5. la note détectée correspond aux cordes standard dans des cas simples
6. le résultat ne clignote pas excessivement à signal stable
7. le code est modulaire et documenté
8. les paramètres principaux sont configurables
9. la TUI reste lisible et exprime clairement la direction d’accordage

---

## 20. Tests attendus

### 20.1 Tests unitaires musique

Tester :

- `440.0 Hz -> A4`
- `82.41 Hz -> E2`
- `110.0 Hz -> A2`
- calcul cents autour des notes cibles
- mapping MIDI -> nom de note

### 20.2 Tests DSP de base

Créer des signaux synthétiques simples :

- sinus 110 Hz
- sinus 82.41 Hz
- sinus 329.63 Hz

Vérifier :

- fréquence détectée proche de l’attendu
- pic plausible détecté
- interpolation améliore ou maintient la précision

### 20.3 Tests de robustesse simples

Tester au moins :

- buffer silence
- buffer bruit faible
- amplitude très basse
- signal hors plage

Attendu :

- pas de faux positif systématique

---

## 21. Paramètres configurables

La V0.1 expose au minimum :

- `sample_rate`
- `frame_size`
- `hop_size`
- `min_frequency_hz`
- `max_frequency_hz`
- `min_rms`
- `min_clarity`
- taille de fenêtre de smoothing

---

## 22. Non-objectifs V0.1

La V0.1 ne couvre pas :

- détection polyphonique
- autocorrection musicale
- apprentissage machine
- analyse spectrale avancée
- calibration automatique des microphones
- persistance
- profils d’instrument

---

## 23. Historique d’implémentation V0.1

La V0.1 a été construite dans l’ordre logique suivant :

1. couche musicale `core`
2. détecteur `dsp::pitch_detector`
3. smoothing et heuristiques de stabilité
4. capture micro via `cpal`
5. orchestration `cli::app::tuner_engine`
6. TUI live et mode démo
7. refactor de la TUI en architecture modulaire cyberpunk

---

## 24. Exigences de qualité de code

Le code produit doit :

- être idiomatique Rust
- éviter les `unwrap()` non justifiés dans le cœur runtime
- séparer clairement audio, DSP, musique et orchestration
- documenter les fonctions critiques
- rester lisible avant d’être micro-optimisé

L’optimisation prématurée n’est pas prioritaire.

---

## 25. Livrables V0.1

La V0.1 livrée comprend :

1. code source compilable
2. structure modulaire conforme à l’esprit de cette spec
3. tests unitaires minimaux
4. README expliquant :
   - comment lancer
   - paramètres importants
   - limites de V0.1
5. exemple de sortie terminal

---

## Décision technique figée pour V0.1

La V0.1 repose sur :

- **détection de pitch par NSDF / MPM**
- **filtrage des faux pics**
- **interpolation parabolique**
- **stabilisation temporelle**
- **conversion fréquence -> note -> cents**
