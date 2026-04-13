# tuner-web

Client web baseline (legacy) conservé dans le monorepo.

Le client principal pour la roadmap produit est :

- `apps/webapp-mobile`

## Statut

- `tuner-web` reste exécutable et utile pour des tests/comparaisons.
- les évolutions UX récentes sont faites en priorité sur `webapp-mobile`.

## Commandes

Depuis `apps/tuner-web` :

- `npm ci`
- `npm run dev`
- `npm run wasm:build`
- `npm run test`
- `npm run build`

## Bridge WASM

- crate source : `crates/tuner-dsp-web`
- artefacts générés : `src/tuner/wasm/pkg`
- script : `scripts/build-wasm-from-workspace.mjs`
