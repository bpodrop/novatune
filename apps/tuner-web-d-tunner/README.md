# tuner-web-d-tunner

D tunner web tuner client built with React, TypeScript, and Vite.

Mobile-first Precision Tuner implementation generated from D tunner component specs.

## WASM Build

WASM bridge artifacts in `src/tuner/wasm/pkg` are generated directly from monorepo crate:
- crate: `crates/tuner-dsp-web`
- build command: `npm run wasm:build`
- build script: `scripts/build-wasm-from-workspace.mjs`

This keeps CLI, web, and embedded code paths aligned on the same commit.

## Tuning Config Contract

The app configures the active wasm detector using:

- `set_preset(detector_id, preset_id)`
- `set_calibration_hz(detector_id, calibration_hz)`

This keeps displayed note/cents consistent with CLI/native/embedded mapping.
