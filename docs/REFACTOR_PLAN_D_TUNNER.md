# Refactor Plan — D tunner / Shared Tuning Architecture

## Goal

Reduce duplicated logic, tighten platform contracts, and simplify the web/CLI integration path without changing the product surface.

The current top-level architecture is viable:

- `tuner-core`: musical/domain model
- `tuner-dsp-algo`: pitch detection
- platform crates: `web`, `native`, `embedded`
- apps: CLI + web clients

The main issues are lower-level:

- tuning-mapping logic duplicated in multiple crates
- string-based contracts where explicit target identity should be used
- duplicated preset catalogs in web UI
- overly large app/controller files in the D tunner web app

---

## Findings To Address

### 1. Duplicate tuning mapping logic

The same preset/calibration mapping logic exists in:

- `crates/tuner-dsp-web/src/lib.rs`
- `crates/tuner-dsp-native/src/tuning.rs`
- `crates/tuner-dsp-embedded/src/tuning.rs`

Risk:

- bug fixes must be applied three times
- behavior drift across platforms is likely
- tests are split across copies instead of protecting one implementation

### 2. Weak preset output contract

`MappedDetection` currently exposes display-oriented fields:

- `note_name`
- `cents_off`
- `ui_state`

Then `tuner-cli` re-derives the target string by matching `note_name` against preset string labels.

Risk:

- label format changes can silently break matching
- enharmonic policy changes become dangerous
- the contract is harder to reason about than necessary

### 3. Web app owns preset catalog

`apps/tuner-web-d-tunner/src/App.tsx` hardcodes:

- preset IDs
- labels
- target string frequencies

while `tuner-core` already owns the canonical preset definitions.

Risk:

- web preset list can diverge from CLI/native/core
- adding new presets requires touching multiple layers
- the UI cannot trust the shared model as source of truth

### 4. D tunner app file is doing too much

`apps/tuner-web-d-tunner/src/App.tsx` currently owns:

- page routing
- audio session lifecycle
- wasm session lifecycle
- tuning config synchronization
- view-level state
- large inline presentational components

Risk:

- hard to test
- hard to reuse logic between baseline web and D tunner variant
- future changes will accumulate in one file

### 5. Web bridge buffering model is heavier than necessary

`tuner-dsp-web` uses a global detector registry with mutable buffers and repeated draining.

This is acceptable short-term, but it is not the cleanest long-term shape.

Risk:

- more complexity than needed for wasm-facing session management
- harder to evolve when more detector/session config is added

---

## Refactor Strategy

Do this in small steps. Do not attempt a big-bang rewrite.

### Phase 1. Centralize tuning mapping

Target:

- one shared Rust implementation for preset/calibration mapping

Action:

- move `TuningSession`, `MappedDetection`, and `resolve_ui_state` into a shared non-platform location
- preferred location: `tuner-core`
- acceptable alternative: new crate such as `crates/tuner-mapping`

Success criteria:

- `native`, `embedded`, and `web` stop owning separate copies
- all tests use the same implementation

### Phase 2. Strengthen output contract

Target:

- eliminate string-based target recovery

Change `MappedDetection` to include explicit target identity:

- `preset_id: Option<PresetId>`
- `string_index: Option<u8>`
- `target_note_name: Option<String>`
- `target_frequency_hz: Option<f32>`

Optional:

- return `TargetString` directly where ownership/serialization permits

Success criteria:

- `tuner-cli` preset mode no longer does `find(|target| target.label == mapped.note_name)`
- matching becomes identity-based, not display-label-based

### Phase 3. Expose preset catalog to web clients

Target:

- remove hardcoded preset metadata from `apps/tuner-web-d-tunner`

Action:

- add bridge API returning preset metadata from shared Rust
- use that in both web apps

Suggested wasm API:

```rust
pub fn list_presets() -> Vec<PresetDescriptor>;
```

Where `PresetDescriptor` contains:

- `id`
- `display_name`
- `strings`

Success criteria:

- no duplicated preset frequency table in TypeScript
- web app renders from shared Rust data

### Phase 4. Split D tunner controller from presentation

Target:

- make `App.tsx` small and testable

Suggested extraction:

- `src/features/tuner/useTunerSession.ts`
- `src/features/tuner/TunerScreen.tsx`
- `src/features/presets/PresetsScreen.tsx`
- `src/features/calibration/CalibrationScreen.tsx`
- `src/features/settings/SettingsScreen.tsx`
- `src/components/...` for shared visual pieces

Success criteria:

- `App.tsx` becomes composition and routing only
- audio/wasm lifecycle lives in a hook/controller module

### Phase 5. Align baseline web and D tunner app

Target:

- avoid maintaining two separate app logic stacks unnecessarily

Action:

- extract shared web tuner session code into a common module under each app or a shared app package if introduced later
- keep visual layers separate

Success criteria:

- baseline web and D tunner differ mostly in UI, not in tuner control logic

### Phase 6. Revisit wasm bridge lifecycle shape

Target:

- simplify detector/session lifecycle internals once logic is centralized

Options:

- keep current API but replace internal buffering with a more efficient ring-buffer-like structure
- or move toward a cleaner session wrapper design if wasm constraints allow

This phase is lower priority than Phases 1 to 4.

---

## Recommended Execution Order

1. Move `TuningSession` and `MappedDetection` into shared Rust code.
2. Update `native`, `embedded`, and `web` to call the shared implementation.
3. Strengthen `MappedDetection` so target identity is explicit.
4. Simplify `tuner-cli` preset path to consume the stronger contract directly.
5. Expose shared preset catalog to wasm.
6. Remove hardcoded preset metadata from `apps/tuner-web-d-tunner/src/App.tsx`.
7. Split `App.tsx` into controller hook + screen components.
8. Optionally extract shared web client logic between `tuner-web` and `tuner-web-d-tunner`.

---

## Anti-Goals

Do not do these during the refactor unless necessary:

- replace the pitch detection algorithm
- redesign all UI screens
- merge both web apps into one app
- introduce a large workspace/package-management migration
- rewrite the CLI/TUI interaction model

---

## Acceptance Criteria

The refactor is successful when:

- there is exactly one tuning-mapping implementation in shared Rust
- platform crates use adapters, not forks of the same logic
- preset matching is based on explicit identity, not note-label string comparison
- web apps no longer hardcode canonical preset frequencies
- `apps/tuner-web-d-tunner/src/App.tsx` is substantially reduced in responsibility
- cross-platform tests still pass

---

## Suggested First Change

The best first implementation step is:

- move `TuningSession` + `MappedDetection` into `tuner-core`

Reason:

- highest reduction in duplication
- lowest product risk
- unlocks the later cleanup steps without changing UI behavior
