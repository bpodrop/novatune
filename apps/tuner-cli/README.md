# tuner-cli

Terminal tuner app (live TUI + demo) built on shared Rust crates.

## Commands

- `tuner-cli strings [--preset <id>] [--a4 <hz>]`
- `tuner-cli analyze [--mode chromatic|preset] [--preset <id>] [--a4 <hz>]`
- `tuner-cli tune <NOTE>`
- `tuner-cli demo`

## Examples

- `tuner-cli strings --preset drop-d --a4 432`
- `tuner-cli analyze --mode preset --preset e-standard --a4 442`
- `tuner-cli tune E2`

## Notes

- `--a4` sets calibration reference (Hz) for preset/chromatic target mapping in live analysis.
- Preset/note mapping uses shared `tuner-dsp-native::TuningSession`.
