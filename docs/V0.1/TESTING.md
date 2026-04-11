# Testing

## Priorités

1. logique musicale
2. détection DSP sur signaux synthétiques
3. rejet des cas invalides
4. stabilité du pipeline lors des itérations suivantes
5. stabilité visuelle et lisibilité de la TUI

## Types de tests

- unitaires sur `core`
- unitaires sur `dsp`
- tests synthétiques sur sinusoïdes simples
- tests de robustesse sur silence, bruit, transitions de note et amplitude faible
- tests de séquences côté moteur d'application
- tests unitaires sur le mapping UI `cents -> statut -> position`
- tests unitaires sur les phases UI `NoSignal / Searching / Unstable / InTune`

## Ce qu’il faut valider

- `440.0 Hz -> A4`
- `82.41 Hz -> E2`
- `110.0 Hz -> A2`
- `329.63 Hz -> E4`
- silence: aucun faux positif
- bruit non périodique: aucun faux positif crédible
- signal faible: rejet
- fréquence détectée proche de l’attendu sur une sinusoïde propre
- transition stable entre deux notes réelles, par exemple `E2 -> D3`
- perte de signal: la stabilité doit retomber puis se reconstruire
- pitch détecté mais note non publiable: l'UI doit passer en `Searching`
- pitch/note présents mais stabilité insuffisante: l'UI doit passer en `Unstable`
- le marqueur de jauge doit rester clampé entre `-50` et `+50` cents
- l'état `InTune` doit rendre un marqueur fort au centre
- l'état `NoSignal` doit afficher `--`, `NO SIGNAL` et des métriques neutres
- le mode démo doit couvrir `NoSignal`, `Searching`, `TooLow`, `InTune`, `TooHigh`

## Critères de traitement du signal

- plage de recherche utile: `70 Hz` à `400 Hz`
- seuil RMS minimal configurable
- seuil de clarté minimal configurable
- interpolation parabolique appliquée au pic retenu
- pas de note inventée si le buffer est invalide
- transition vers une nouvelle note seulement après confirmation sur plusieurs frames

## Commandes utiles

```bash
cargo test
cargo test -p tuner-core
cargo test -p tuner-dsp-algo
cargo run -p tuner-cli -- strings
cargo run -p tuner-cli -- demo
cargo run -p tuner-cli -- analyze
cargo run -p tuner-cli -- tune A2
```

## Attentes qualitatives V1

- l’erreur cible reste faible sur signal propre
- la note affichée ne doit pas osciller excessivement
- les faux positifs doivent rester rares sur silence
- une nouvelle corde jouée doit remplacer l'ancienne après quelques frames cohérentes
- la TUI doit rester lisible sans scintillement ni saut visuel de la note
- la jauge doit garder un centre fixe et une largeur stable
- les micro-animations éventuelles doivent rester rares et ne jamais perturber la lecture des données
- la CLI/TUI doit rester exploitable si elle lit le micro réel ou un fallback mock

