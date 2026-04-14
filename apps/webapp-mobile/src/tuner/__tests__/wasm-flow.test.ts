import { describe, expect, it } from 'vitest'

import {
  type BridgeApi,
  createWasmTunerSession,
  type DetectionResult,
} from '../wasmTuner'
import { createSineWave } from '../../audio/sineWave'
import {
  initSync,
  list_presets,
  new_detector,
  next_output,
  push_samples,
  reset,
  set_calibration_hz,
  set_min_clarity,
  set_min_rms,
  set_mode,
  set_preset,
  shutdown,
} from '../wasm/pkg/tuner_web_bridge.js'
import { readFileSync } from 'node:fs'

function createBridgeStub(output: BridgeApi['next_output'] extends (id: number) => infer R ? R : never): BridgeApi {
  return {
    list_presets: () => [],
    new_detector: () => 7,
    push_samples: () => (output ? 1 : 0),
    next_output: () => output,
    set_preset: () => true,
    set_mode: () => true,
    set_calibration_hz: () => true,
    set_min_rms: () => true,
    set_min_clarity: () => true,
    reset: () => true,
    shutdown: () => true,
  }
}

describe('wasm tuner integration flow', () => {
  it('ingests samples and emits structured output with state transition', () => {
    const bridgeOutput: DetectionResult = {
      frequencyHz: 440,
      confidence: 0.92,
      clarity: 0.86,
      rms: 0.19,
      centsOff: 0.4,
      noteName: 'A4',
      tuningProfileId: 'e-standard',
      stringIndex: 1,
      stringCount: 6,
      stringName: 'A4',
      mode: 'preset',
      uiState: 'in_tune',
    }

    const session = createWasmTunerSession({
      bridge: createBridgeStub({
        frequency_hz: bridgeOutput.frequencyHz,
        confidence: bridgeOutput.confidence,
        clarity: bridgeOutput.clarity,
        rms: bridgeOutput.rms,
        cents_off: bridgeOutput.centsOff,
        note_name: bridgeOutput.noteName,
        tuning_profile_id: bridgeOutput.tuningProfileId ?? undefined,
        string_index: bridgeOutput.stringIndex ?? undefined,
        string_count: bridgeOutput.stringCount ?? undefined,
        string_name: bridgeOutput.stringName ?? undefined,
        mode: bridgeOutput.mode,
        ui_state: bridgeOutput.uiState,
      }),
      sampleRate: 48_000,
      frameSize: 2048,
      hopSize: 512,
    })

    expect(session.getState()).toBe('ready')

    const results = session.ingestSamples(Float32Array.from({ length: 2048 }, () => 0.1))

    expect(results).toHaveLength(1)
    expect(results[0].frequencyHz).toBeCloseTo(bridgeOutput.frequencyHz, 5)
    expect(results[0].confidence).toBeCloseTo(bridgeOutput.confidence, 5)
    expect(results[0].clarity).toBeCloseTo(bridgeOutput.clarity, 5)
    expect(results[0].rms).toBeCloseTo(bridgeOutput.rms, 5)
    expect(results[0].noteName).toBe('A4')
    expect(results[0].tuningProfileId).toBe('e-standard')
    expect(results[0].stringIndex).toBe(1)
    expect(results[0].stringCount).toBe(6)
    expect(results[0].uiState).toBe('in_tune')
    expect(session.getState()).toBe('in_tune')

    session.reset()
    expect(session.getState()).toBe('ready')
  })

  it('runs real wasm bridge end-to-end on synthetic A2 signal', () => {
    const wasmBytes = readFileSync(new URL('../wasm/pkg/tuner_web_bridge_bg.wasm', import.meta.url))
    initSync({ module: wasmBytes })
    const bridge: BridgeApi = {
      list_presets,
      new_detector,
      next_output,
      push_samples,
      set_preset,
      set_mode,
      set_calibration_hz,
      set_min_rms,
      set_min_clarity,
      reset,
      shutdown,
    }
    const session = createWasmTunerSession({
      bridge,
      sampleRate: 48_000,
      frameSize: 2048,
      hopSize: 512,
    })

    const samples = createSineWave(110, 48_000, 8192)
    const results = session.ingestSamples(samples)

    expect(results.length).toBeGreaterThan(0)
    const last = results[results.length - 1]
    expect(last.frequencyHz).toBeGreaterThan(90)
    expect(last.frequencyHz).toBeLessThan(130)
    expect(last.noteName).toBe('A2')
    expect(['searching', 'unstable', 'too_low', 'in_tune', 'too_high']).toContain(last.uiState)
    expect(session.getState()).toBe(last.uiState)

    session.close()
    expect(session.getState()).toBe('closed')
  })
})
