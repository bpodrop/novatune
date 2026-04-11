# Decisions

## ADR-001 - Workspace multi-crates

- Date: 2026-03-30
- Statut: accepted

Contexte: le projet doit rester simple mais testable, avec séparation claire entre métier, DSP et CLI.

Décision: utiliser un workspace Cargo avec crates internes `core`, `dsp` et `cli`.

Conséquences:
- séparation nette des responsabilités
- tests ciblés par couche
- intégration audio possible sans mélanger le domaine musical et la CLI

## ADR-002 - Détection temporelle basée sur NSDF

- Date: 2026-03-30
- Statut: accepted

Contexte: `SPEC.md` écarte la FFT comme méthode principale et demande une approche MPM / NSDF.

Décision: baser le détecteur de pitch sur NSDF avec sélection de pic, filtrage des faux pics et interpolation parabolique.

Conséquences:
- meilleur alignement avec la spec
- pipeline explicable et ajustable
- implémentation plus lisible pour V1 qu’une chaîne DSP plus agressive

## ADR-003 - Types musicaux dans `core`

- Date: 2026-03-30
- Statut: accepted

Contexte: la conversion fréquence -> note -> cents est une logique métier partagée.

Décision: centraliser `Note`, `FrequencyHz`, `Cents`, `NoteEstimate` et `STANDARD_TUNING` dans `core`.

Conséquences:
- réutilisation simple par `dsp` et `cli`
- tests unitaires concentrés sur la logique musicale
- moins de duplication entre couches

## ADR-004 - Dépendances minimales hors cœur DSP

- Date: 2026-03-30
- Statut: accepted

Contexte: le projet doit rester léger, mais la capture audio et la TUI terminal font partie du périmètre V0.1.

Décision: limiter les dépendances à `cpal` pour l'audio et `crossterm` / `ratatui` pour la TUI, sans bibliothèque DSP externe.

Conséquences:
- surface de maintenance réduite
- pipeline DSP gardé local et explicable
- capture micro et TUI intégrées sans sur-ingénierie

## ADR-005 - TUI modulaire séparée de la logique métier

- Date: 2026-03-31
- Statut: accepted

Contexte: la V0.1 devait livrer une interface terminal plus lisible et expressive qu'une simple sortie texte.

Décision: structurer la TUI en modules `theme`, `layout`, `widgets` et projeter `TunerReading` vers un état UI dédié.

Conséquences:
- séparation nette entre DSP, orchestration et rendu
- évolution visuelle moins coûteuse
- tests et logique de mapping UI plus simples à maintenir

## ADR-006 - Regroupement des paramètres contextuels

- Date: 2026-04-02
- Statut: accepted

Contexte: plusieurs fonctions internes du runtime audio et du mode démo TUI avaient accumulé trop d'arguments, ce qui dégradait la lisibilité et faisait échouer `clippy` avec `-D warnings`.

Décision: regrouper les paramètres liés à un même contexte dans de petites structs dédiées, par exemple `FrameConfig`, `StreamState` et `DemoTunerState`, plutôt que d'étendre les signatures.

Conséquences:
- signatures plus courtes et plus lisibles
- séparation plus explicite entre configuration, état partagé et données de rendu
- conformité `clippy` maintenue sans introduire de `#[allow(...)]`

