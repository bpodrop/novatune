# NovaTune Documentation

Cette documentation est la source de vérité du repo pour l'état actuel de l'implémentation.
Les documents historiques/specs de planification ont été retirés.

Documents principaux :

- [ARCHITECTURE_ANALYSIS.md](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\docs\ARCHITECTURE_ANALYSIS.md)
- [TARGET_ARCHITECTURE.md](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\docs\TARGET_ARCHITECTURE.md)
- [REFACTOR_PLAN.md](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\docs\REFACTOR_PLAN.md)

## Monorepo actuel

Workspace Rust (déclaré dans `Cargo.toml`) :

- `crates/tuner-core`
- `crates/tuner-dsp-algo`
- `crates/tuner-engine`
- `crates/tuner-dsp-native`
- `crates/tuner-dsp-web`
- `crates/tuner-dsp-embedded`
- `apps/tuner-cli`

Package web partagé :

- `packages/tuner-web-core`

Applications présentes :

- `apps/webapp-mobile` (client web principal, PWA mobile-first)
- `apps/tuner-cli` (client terminal live)

## Architecture fonctionnelle

- `tuner-core` : notes, cents, presets, mapping tuning.
- `tuner-dsp-algo` : détection de pitch (NSDF/MPM), filtrage et stabilité.
- `tuner-engine` : moteur applicatif partagé du tuner.
- `tuner-dsp-web` : bridge WASM pour le web, branché sur `tuner-engine`.
- `tuner-dsp-native` : intégration audio native (CLI).
- `tuner-dsp-embedded` : intégration embarquée conservée au niveau crate, sans app active dédiée.
- `tuner-cli` : interface terminale live + commandes utilitaires.
- `tuner-web-core` : wrappers TS partagés du runtime web.
- `webapp-mobile` : UI React/Vite branchée sur le bridge WASM via `tuner-web-core`.

## Modes de fonctionnement tuner

Le comportement courant supporte deux modes :

1. `preset`
- mapping vers le preset actif (ex: `e-standard`, `drop-d`, `eb-standard`)
- affichage note + cible de corde (`string`) quand disponible

2. `chromatic`
- mapping chromatique pur (sans cible de preset)
- affichage note/cents sans corde cible

## Commandes clés

Depuis la racine :

- tests Rust :
  - `cargo test`

Dans `apps/webapp-mobile` :

- install : `npm ci`
- build wasm bridge : `npm run wasm:build`
- tests : `npm run test`
- build prod : `npm run build`
- validation PWA : `npm run pwa:validate`

Dans `apps/tuner-cli` :

- exécution : `cargo run -p tuner-cli -- <commande>`

## CI / CD

Workflow principal webapp + wasm :

- `.github/workflows/wasm-ci.yml`

Il vérifie au minimum :

- génération WASM
- cohérence des artefacts commités `src/tuner/wasm/pkg`
- lint, tests, build, validation PWA

## Déploiement Cloudflare Pages (`apps/webapp-mobile`)

Paramètres recommandés :

- Framework preset : `Vite`
- Root directory : `apps/webapp-mobile`
- Build command : `npm ci && npm run build`
- Output directory : `dist`
- Node.js : `20`

## Références par application

- `apps/webapp-mobile/README.md`
- `apps/tuner-cli/README.md`
