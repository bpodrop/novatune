# D tunner Export Intake

## Source Files

- `tmp/components.md`: usable component specification for Precision Tuner mobile screen.
- `tmp/next.md`: generic PRD template output, no screen-specific implementation data.

## Imported Spec (from `tmp/components.md`)

- Background: `#0e0e0e`
- Panel background: `#131313`
- Primary glow: `#00FF41`
- Accent: `#00e3fd`
- Typography: `Space Grotesk`

### Component List

- `TopAppBar`
- `ChromaticGauge`
- `NoteDisplay`
- `SignalInfo`
- `TuningPresetStrip`
- `SpectralGraph`

## Mapping status

- [x] Tuner screen shell implemented in `apps/tuner-web-d-tunner/src/App.tsx`
- [x] Mobile-first style/tokens implemented in `apps/tuner-web-d-tunner/src/App.css`
- [x] Presets screen detailed layout
- [x] Calibration screen detailed layout
- [x] Settings screen detailed layout
- [x] Preset + calibration logic moved to shared Rust/WASM bridge APIs
