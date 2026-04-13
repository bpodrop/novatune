# Changelog

## [0.1.0] - 2026-04-13

- Mobile-first PWA UI finalized with responsive tuner layout.
- Added dual tuning modes: `preset` and `chromatic` end-to-end (UI + WASM + Rust core mapping).
- Added live DSP sensitivity controls (`min_rms`, `min_clarity`).
- Migrated microphone processing from deprecated `ScriptProcessorNode` to `AudioWorkletNode`.
- Added in-app preset picker and removed bottom navigation menu.
- Added dark/light theme support with icon toggle.
- Added app footer with version and GitHub icon link.
- Added/updated WASM CI workflow and committed WASM bridge artifacts.
