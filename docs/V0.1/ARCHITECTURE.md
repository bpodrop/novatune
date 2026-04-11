# Architecture

## Objectif technique

Construire un accordeur guitare simple en Rust, centré sur une détection de pitch monophonique stable dans la plage 70 Hz à 400 Hz, avec capture micro temps réel et TUI terminal modulaire.

## Pipeline

1. acquisition audio mono
2. prétraitement léger
3. calcul NSDF / MPM
4. détection de pics
5. filtrage des faux pics
6. interpolation parabolique
7. conversion fréquence -> note -> cents
8. lissage temporel
9. orchestration applicative
10. exposition CLI/TUI

## Modules Rust

- `core`
  - types métier: fréquence, cents, note, accordage standard
  - structures de sortie partagées entre couches
- `dsp`
  - prétraitement, NSDF, sélection des pics, interpolation, smoothing
  - orchestration du détecteur de pitch
- `cli`
  - commandes `strings`, `analyze`, `tune`, `demo`
  - `app::tuner_engine` pour l'orchestration lecture -> note publiée
  - TUI `ratatui/crossterm` modulaire avec `theme`, `layout` et widgets dédiés

## Flux de données

Le flux audio brut est capturé par `dsp::audio`, converti en frames glissantes `f32`, puis passé à `PitchDetector`. `cli::app::TunerEngine` applique ensuite les heuristiques de publication, expose un `TunerReading`, et la TUI projette ce résultat dans un état d'affichage séparé.

## Dépendances actuelles

- `cpal` pour la capture micro
- `crossterm` pour le terminal interactif
- `ratatui` pour le rendu TUI
- aucune dépendance DSP externe au-delà de `cpal`

## Conventions

- garder les responsabilités séparées par crate
- préserver les types musicaux dans `core`
- garder `dsp` sans I/O
- documenter les fonctions critiques de détection
- privilégier des fonctions pures testables
- regrouper les paramètres contextuels dans de petites structs quand une fonction commence à porter trop d'arguments

## Hypothèses V1

- une seule corde jouée à la fois
- entrée micro monophonique
- sample rate cible `44_100 Hz`
- fenêtre par défaut `4096`
- confiance et stabilité encore heuristiques
- la TUI live reste dépendante du comportement réel du périphérique micro et du terminal

