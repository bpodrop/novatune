# tuner-cli

Application terminal NovaTuner (TUI live + démo) basée sur les crates Rust partagées.

## Exécution

Depuis la racine du repo :

- `cargo run -p tuner-cli -- <commande>`

## Commandes

- `strings [--preset <id>] [--a4 <hz>]`
- `analyze [--mode chromatic|preset] [--preset <id>] [--a4 <hz>]`
- `tune <NOTE>`
- `demo`
- `help`

## Exemples

- `cargo run -p tuner-cli -- strings --preset drop-d --a4 432`
- `cargo run -p tuner-cli -- analyze --mode chromatic`
- `cargo run -p tuner-cli -- analyze --mode preset --preset e-standard --a4 442`
- `cargo run -p tuner-cli -- tune E2`

## Raccourcis TUI live

- `q` / `Esc` : quitter
- `m` : basculer `Chromatic` / `Preset` (en mode analyze)
- `p` : ouvrir le sélecteur de presets
- `Up` / `Down` : naviguer dans le picker preset
- `Enter` : appliquer le preset sélectionné
- `h` / `?` : afficher/masquer l'aide

## Notes

- splashscreen au démarrage (quittable avec `q` ou `Esc`)
- le mapping preset/chromatic est partagé via `tuner-core` + `tuner-dsp-native`
