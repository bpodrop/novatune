# Refactor Plan — D-tunner / Shared Tuning Architecture

## Goal

Reduce duplicated logic, tighten platform contracts, and simplify web/CLI integration without changing product behavior.

The top-level architecture remains valid:

- `tuner-core`: musical/domain model
- `tuner-dsp-algo`: pitch detection
- platform crates: `web`, `native`, `embedded`
- apps: CLI + web clients

Current issues are in integration and contract boundaries:

- duplicated tuning mapping logic in platform crates
- string-based identity in output contracts
- duplicated preset catalog in web UI
- oversized D-tunner `App.tsx` mixing lifecycle and presentation
- wasm buffering lifecycle heavier than needed

---

## Current-State Findings (Validated In Code)

### 1. Duplicate tuning mapping logic

Logic is duplicated in:

- `crates/tuner-dsp-web/src/lib.rs`
- `crates/tuner-dsp-native/src/tuning.rs`
- `crates/tuner-dsp-embedded/src/tuning.rs`

Risk:

- bug fixes must be applied in 3 places
- behavior drift across platforms
- test coverage split across copies

### 2. Weak preset output contract at integration boundary

`MappedDetection` currently leans on display fields (`note_name`, `cents_off`, `ui_state`), and CLI preset mode still re-derives a target via note label matching.

Risk:

- label changes can break matching silently
- enharmonic policy changes become risky
- integration is harder to reason about than identity-based matching

### 3. Web app owns canonical preset metadata

`apps/tuner-web-d-tunner/src/App.tsx` hardcodes preset IDs, labels, and target frequencies while canonical tuning data already exists in `tuner-core`.

Risk:

- web/CLI/native preset drift
- new preset rollout requires multi-layer edits
- UI cannot trust shared model as source of truth

### 4. D-tunner `App.tsx` mixes too many responsibilities

Current responsibilities include:

- view routing
- microphone lifecycle
- wasm session lifecycle
- tuning config sync
- view state and presentation components

Risk:

- difficult unit testing
- low reusability between baseline web and D-tunner
- change concentration in one file

### 5. Web bridge buffering/lifecycle is workable but not ideal

`tuner-dsp-web` currently uses a global detector registry, mutable pending sample buffers, and repeated draining.

Risk:

- unnecessary lifecycle complexity
- scaling pain when detector/session features grow

---

## Preflight (Required Before Refactor)

Purpose: lock baseline behavior before code movement.

1. Add/confirm baseline tests in current architecture:
- mapping parity tests for native/web/embedded outputs on same inputs
- CLI preset-mode integration tests
- wasm bridge API contract tests (shape + semantics)

2. Capture baseline contract snapshots:
- sample `DetectionOutput` payloads for key scenarios (in tune, off, unstable, no signal)
- preset list snapshots currently used by web clients

3. Define compatibility policy:
- additive fields in wasm output are allowed
- field rename/removal requires explicit app migration in same change set

Exit gate:

- baseline tests pass and snapshots are committed

---

## Refactor Strategy (Phased, No Big-Bang)

### Phase 1. Centralize mapping implementation in shared Rust

Target:

- exactly one implementation of tuning mapping/session logic

Action:

- move `TuningSession`, mapping logic, and `resolve_ui_state` into shared location
- preferred: `tuner-core` (or dedicated crate like `crates/tuner-mapping` if dependency pressure appears)
- keep thin adapters in `web`, `native`, `embedded`

Success criteria:

- duplicated mapping code removed from platform crates
- platform crates call shared implementation only
- mapping parity tests still pass

### Phase 2. Strengthen output identity contract

Target:

- remove string-label dependence from consumers

Action:

- evolve mapped output to include explicit target identity:
  - `preset_id: Option<PresetId>`
  - `string_index: Option<u8>` or `string: Option<TargetString>`
  - `target_note_name: Option<String>`
  - `target_frequency_hz: Option<f32>`
- align with existing `tuner-core` model (`TuningTarget` already carries `preset_id` and optional `string`)

Success criteria:

- CLI preset mode no longer relies on `target.label == mapped.note_name`
- matching becomes identity-based
- existing display behavior unchanged

### Phase 3. Expose preset catalog via shared bridge API

Target:

- remove canonical preset duplication from TypeScript

Action:

- add bridge API to list presets from shared Rust in both web apps

Suggested wasm API:

```rust
pub fn list_presets() -> Vec<PresetDescriptor>;
```

`PresetDescriptor`:

- `id`
- `display_name`
- `strings` (label + nominal frequency)

Success criteria:

- no hardcoded canonical preset frequency table in web app code
- web preset views render from Rust-provided metadata

### Phase 4. Split D-tunner controller from presentation

Target:

- make `App.tsx` composition + routing only

Suggested extraction:

- `src/features/tuner/useTunerSession.ts`
- `src/features/tuner/TunerScreen.tsx`
- `src/features/presets/PresetsScreen.tsx`
- `src/features/calibration/CalibrationScreen.tsx`
- `src/features/settings/SettingsScreen.tsx`
- `src/components/*` for shared UI blocks

Success criteria:

- audio/wasm lifecycle isolated in hook/controller module
- screen components are presentational and testable
- `App.tsx` no longer owns integration-heavy effects

### Phase 5. Align baseline web and D-tunner logic stack

Target:

- avoid duplicate control logic across web apps

Action:

- extract shared web tuner session logic into common modules
- keep visual/polish layers app-specific

Success criteria:

- differences between apps are primarily UI, not tuner control behavior
- shared tests cover common session logic

### Phase 6. Revisit wasm bridge lifecycle internals (lower priority)

Target:

- simplify/optimize session buffering internals after centralization

Options:

- keep public API, replace internals with ring-buffer-like structure
- or introduce cleaner session wrapper if wasm constraints allow

Success criteria:

- reduced complexity and stable performance
- no API regression for web clients

---

## Recommended Execution Order

1. Complete preflight tests/snapshots and lock baseline.
2. Move mapping/session logic to one shared implementation.
3. Switch native/embedded/web adapters to shared mapping.
4. Upgrade mapped output contract to explicit identity.
5. Remove CLI label-based target lookup.
6. Add wasm `list_presets()` and migrate both web apps.
7. Remove hardcoded preset metadata from D-tunner app.
8. Split D-tunner `App.tsx` into controller + screens.
9. Extract shared web logic between `tuner-web` and `tuner-web-d-tunner`.
10. Optionally optimize wasm lifecycle internals.

---

## Compatibility Rules

- Do not break user-visible tuner behavior during Phases 1-4.
- Keep wasm API backward compatible while web clients are migrating.
- If breaking API is unavoidable, migrate both web apps in the same PR.
- Preserve current calibration semantics (`normalized_frequency_hz = measured * 440 / calibration`).

---

## Anti-Goals

Do not do these unless explicitly required:

- replace pitch detection algorithm
- redesign all UI screens
- merge both web apps into one app
- introduce large workspace/package-management migration
- rewrite CLI/TUI interaction model

---

## Definition Of Done

Refactor is complete when all are true:

- one shared mapping/session implementation exists in Rust
- platform crates are adapters only, not logic forks
- preset matching is identity-based, not label-based
- canonical preset frequencies are no longer hardcoded in web app code
- D-tunner `App.tsx` is reduced to composition/routing responsibilities
- cross-platform tests pass (native, embedded, wasm/web, CLI integration)

---

## Suggested First Implementation Change

Start with centralizing `TuningSession` and mapped output logic into shared Rust (`tuner-core` preferred), while keeping existing outputs stable.

Reason:

- highest duplication reduction
- lowest product-risk change
- unlocks contract and UI cleanup phases cleanly
