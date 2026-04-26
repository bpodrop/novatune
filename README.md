# NovaTune

Monorepo d'accordeur guitare (Rust + Web) avec partage du coeur métier, du DSP, d'un moteur applicatif commun et des bridges plateforme.

Documentation active :

- [docs/README.md](./docs/README.md)

Applications principales :

- `apps/webapp-mobile` : client web/PWA principal
- `apps/tuner-cli` : client terminal live (TUI)

Support technique conservé :

- `crates/tuner-dsp-embedded` : couche embarquée encore présente au niveau crate, sans app dédiée active

Socle partagé :

- `crates/tuner-engine` : moteur applicatif partagé entre CLI et bridge WASM
- `packages/tuner-web-core` : runtime web partagé extrait de `webapp-mobile`
