import type { DetectionResult, TunerConfig, TunerState, TuningPresetProfile } from './types'

export interface BridgeApi {
  new_detector(sampleRate: number, frameSize: number, hopSize: number): number
  push_samples(detectorId: number, samples: Float32Array): number
  next_output(detectorId: number): RawBridgeOutput | undefined
  list_presets(): RawBridgePreset[]
  set_preset(detectorId: number, presetId: string): boolean
  set_mode(detectorId: number, mode: 'preset' | 'chromatic'): boolean
  set_preset_match_window_cents(detectorId: number, windowCents: number): boolean
  set_calibration_hz(detectorId: number, calibrationHz: number): boolean
  set_min_rms(detectorId: number, minRms: number): boolean
  set_min_clarity(detectorId: number, minClarity: number): boolean
  reset(detectorId: number): boolean
  shutdown(detectorId: number): boolean
}

interface RawBridgeOutput {
  frequency_hz: number
  confidence: number
  clarity: number
  rms: number
  cents_off: number
  note_name: string
  tuning_profile_id?: string
  string_index?: number
  string_count?: number
  string_name?: string
  mode?: 'preset' | 'chromatic'
  ui_state: DetectionResult['uiState']
}

interface RawBridgePresetString {
  index: number
  display_number: number
  label: string
  frequency_hz: number
}

interface RawBridgePreset {
  id: string
  label: string
  string_count: number
  strings: RawBridgePresetString[]
}

export interface WasmTunerSession {
  ingestSamples(samples: Float32Array): DetectionResult[]
  setMode(mode: 'preset' | 'chromatic'): void
  setPreset(presetId: string): void
  setPresetMatchWindowCents(windowCents: number): void
  setCalibrationHz(calibrationHz: number): void
  setMinRms(minRms: number): void
  setMinClarity(minClarity: number): void
  getState(): TunerState
  reset(): void
  close(): void
}

interface SessionOptions extends TunerConfig {
  bridge: BridgeApi
}

function mapOutput(raw: RawBridgeOutput | undefined): DetectionResult | null {
  if (!raw) {
    return null
  }

  return {
    frequencyHz: raw.frequency_hz,
    confidence: raw.confidence,
    clarity: raw.clarity,
    rms: raw.rms,
    centsOff: raw.cents_off,
    noteName: raw.note_name,
    tuningProfileId: raw.tuning_profile_id ?? null,
    stringIndex: raw.string_index ?? null,
    stringCount: raw.string_count ?? null,
    stringName: raw.string_name ?? null,
    mode: raw.mode ?? 'chromatic',
    uiState: raw.ui_state,
  }
}

function mapPreset(raw: RawBridgePreset): TuningPresetProfile {
  return {
    id: raw.id,
    label: raw.label,
    stringCount: raw.string_count,
    strings: raw.strings.map((rawString) => ({
      index: rawString.index,
      displayNumber: rawString.display_number,
      label: rawString.label,
      frequencyHz: rawString.frequency_hz,
    })),
  }
}

export function listAvailablePresets(bridge: BridgeApi): TuningPresetProfile[] {
  const presets = bridge.list_presets()
  return presets.map(mapPreset)
}

export function createWasmTunerSession(options: SessionOptions): WasmTunerSession {
  const { bridge, sampleRate, frameSize, hopSize } = options
  const detectorId = bridge.new_detector(sampleRate, frameSize, hopSize)

  if (detectorId === 0) {
    throw new Error('Failed to create wasm tuner detector')
  }

  let state: TunerState = 'ready'

  return {
    ingestSamples(samples: Float32Array): DetectionResult[] {
      if (state === 'closed') {
        throw new Error('Cannot ingest samples after close')
      }

      if (samples.length === 0) {
        return []
      }

      const produced = bridge.push_samples(detectorId, samples)
      const results: DetectionResult[] = []
      for (let i = 0; i < produced; i += 1) {
        const raw = bridge.next_output(detectorId)
        const mapped = mapOutput(raw)
        if (mapped) {
          results.push(mapped)
        }
      }

      if (results.length > 0) {
        state = results[results.length - 1].uiState
      } else {
        state = 'awaiting-output'
      }
      return results
    },
    setPreset(presetId: string): void {
      if (state === 'closed') {
        return
      }
      const ok = bridge.set_preset(detectorId, presetId)
      if (!ok) {
        throw new Error(`Failed to set preset: ${presetId}`)
      }
    },
    setMode(mode: 'preset' | 'chromatic'): void {
      if (state === 'closed') {
        return
      }
      const ok = bridge.set_mode(detectorId, mode)
      if (!ok) {
        throw new Error(`Failed to set mode: ${mode}`)
      }
    },
    setPresetMatchWindowCents(windowCents: number): void {
      if (state === 'closed') {
        return
      }
      const ok = bridge.set_preset_match_window_cents(detectorId, windowCents)
      if (!ok) {
        throw new Error(`Failed to set preset match window: ${windowCents}`)
      }
    },
    setCalibrationHz(calibrationHz: number): void {
      if (state === 'closed') {
        return
      }
      const ok = bridge.set_calibration_hz(detectorId, calibrationHz)
      if (!ok) {
        throw new Error(`Failed to set calibration: ${calibrationHz}`)
      }
    },
    setMinRms(minRms: number): void {
      if (state === 'closed') {
        return
      }
      const ok = bridge.set_min_rms(detectorId, minRms)
      if (!ok) {
        throw new Error(`Failed to set min RMS: ${minRms}`)
      }
    },
    setMinClarity(minClarity: number): void {
      if (state === 'closed') {
        return
      }
      const ok = bridge.set_min_clarity(detectorId, minClarity)
      if (!ok) {
        throw new Error(`Failed to set min clarity: ${minClarity}`)
      }
    },
    getState(): TunerState {
      return state
    },
    reset(): void {
      if (state === 'closed') {
        return
      }
      const ok = bridge.reset(detectorId)
      if (!ok) {
        throw new Error('Failed to reset wasm tuner detector')
      }
      state = 'ready'
    },
    close(): void {
      if (state === 'closed') {
        return
      }
      const ok = bridge.shutdown(detectorId)
      if (!ok) {
        throw new Error('Failed to shutdown wasm tuner detector')
      }
      state = 'closed'
    },
  }
}
