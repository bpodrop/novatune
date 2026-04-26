# webapp-mobile

Client web principal NovaTuner (PWA mobile-first) basé sur React + TypeScript + Vite.

Version courante : `0.1.0`.

## Fonctionnalités implémentées

- accordeur temps réel avec bridge WASM (`tuner-dsp-web`)
- mode `preset` et mode `chromatic`
- sélection de preset in-app via bottom sheet
- calibration A4 et réglages DSP (`min_rms`, `min_clarity`)
- thème `dark/light` avec toggle icônes
- interface mobile-first, navigation basse supprimée

## Commandes

Depuis `apps/webapp-mobile` :

- `npm ci`
- `npm run dev`
- `npm run test`
- `npm run wasm:build`
- `npm run build`
- `npm run pwa:validate`

## Bridge WASM

Artefacts commités dans :

- `src/tuner/wasm/pkg`

Source :

- crate Rust : `crates/tuner-dsp-web`
- moteur partagé : `crates/tuner-engine`
- runtime web partagé : `packages/tuner-web-core`
- script : `scripts/build-wasm-from-workspace.mjs`

Quand le DSP/bridge change, exécuter `npm run wasm:build` et committer `src/tuner/wasm/pkg/*`.

## Contrat de configuration runtime

Le frontend configure le detector via :

- `set_mode(detector_id, "preset" | "chromatic")`
- `set_preset(detector_id, preset_id)`
- `set_calibration_hz(detector_id, calibration_hz)`
- `set_min_rms(detector_id, value)`
- `set_min_clarity(detector_id, value)`

## Déploiement Cloudflare Pages

Configuration recommandée :

- Framework preset : `Vite`
- Root directory : `apps/webapp-mobile`
- Build command : `npm ci && npm run build`
- Build output directory : `dist`
- Node.js : `20`
