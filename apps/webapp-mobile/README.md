# NovaTuner

NovaTuner web tuner client built with React, TypeScript, and Vite.

Mobile-first Precision Tuner implementation generated from NovaTuner component specs.

## WASM Build

WASM bridge artifacts live in `src/tuner/wasm/pkg` and are generated from:
- crate: `crates/tuner-dsp-web`
- build command: `npm run wasm:build`
- build script: `scripts/build-wasm-from-workspace.mjs`

For short-term CI/CD stability (Cloudflare Pages), these generated artifacts are committed.
When DSP code changes, regenerate and commit `src/tuner/wasm/pkg/*`.

## Cloudflare Pages (Short Term)

Use these settings for `apps/webapp-mobile`:
- Framework preset: `Vite`
- Root directory: `apps/webapp-mobile`
- Build command: `npm ci && npm run build`
- Build output directory: `dist`
- Node.js version: `20`

Current build no longer runs `wasm:build` automatically, so Pages does not need Rust/wasm-pack.

## Tuning Config Contract

The app configures the active wasm detector using:

- `set_preset(detector_id, preset_id)`
- `set_calibration_hz(detector_id, calibration_hz)`

This keeps displayed note/cents consistent with CLI/native/embedded mapping.
