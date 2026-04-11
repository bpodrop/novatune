# Monorepo Structure

## Workspace Layout

- `crates/tuner-core`
- `crates/tuner-dsp-algo`
- `crates/tuner-dsp-native`
- `crates/tuner-dsp-web`
- `crates/tuner-dsp-embedded`
- `apps/tuner-cli`
- `apps/tuner-embedded`
- `apps/tuner-web` (frontend app baseline)
- `apps/tuner-web-d-tunner` (frontend app, mobile-first D tunner variant)

## Dependency Rules

Allowed:
- `apps/*` -> `crates/*`
- `tuner-dsp-native` -> `tuner-dsp-algo`, `tuner-core`
- `tuner-dsp-web` -> `tuner-dsp-algo`, `tuner-core`
- `tuner-dsp-embedded` -> `tuner-dsp-algo`, `tuner-core`
- `tuner-dsp-algo` -> `tuner-core`

Disallowed:
- `tuner-core` depending on platform crates
- `tuner-dsp-algo` depending on platform crates
- cross-app dependencies (`apps/tuner-cli` -> `apps/tuner-embedded`, etc.)

## Platform Split Intent

- `tuner-core`: domain model, note/preset/tuning logic
- `tuner-dsp-algo`: pitch detection algorithm and smoothing
- `tuner-dsp-native`: native audio capture integration + shared `TuningSession` mapping API
- `tuner-dsp-web`: wasm-facing bridge layer + detector tuning config (`set_preset`, `set_calibration_hz`)
- `tuner-dsp-embedded`: embedded-facing integration layer + shared `TuningSession` mapping API
