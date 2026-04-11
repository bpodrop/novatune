# tuner-web

Web tuner client built with React, TypeScript, and Vite.

This app is the baseline web client. A mobile-first variant is available in
`apps/tuner-web-d-tunner`.

## WASM Build

WASM bridge artifacts in `src/tuner/wasm/pkg` are generated directly from monorepo crate:
- crate: `crates/tuner-dsp-web`
- build command: `npm run wasm:build`
- build script: `scripts/build-wasm-from-workspace.mjs`

This keeps CLI, web, and embedded code paths aligned on the same commit.

## Tuning Config Contract

The wasm bridge exposes detector-level tuning controls:

- `set_preset(detector_id, preset_id)`
- `set_calibration_hz(detector_id, calibration_hz)`
