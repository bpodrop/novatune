export type TunerState =
  | 'idle'
  | 'no_signal'
  | 'searching'
  | 'unstable'
  | 'too_low'
  | 'in_tune'
  | 'too_high'
  | 'ready'
  | 'awaiting-output'
  | 'output-ready'
  | 'closed'

export interface DetectionResult {
  frequencyHz: number
  confidence: number
  clarity: number
  rms: number
  centsOff: number
  noteName: string
  tuningProfileId: string | null
  stringIndex: number | null
  stringCount: number | null
  stringName: string | null
  mode: 'preset' | 'chromatic'
  uiState: Extract<TunerState, 'no_signal' | 'searching' | 'unstable' | 'too_low' | 'in_tune' | 'too_high'>
}

export interface TuningPresetString {
  index: number
  displayNumber: number
  label: string
  frequencyHz: number
}

export interface TuningPresetProfile {
  id: string
  label: string
  stringCount: number
  strings: TuningPresetString[]
}

export type PresetFamilyFilter = 'all' | 6 | 7 | 8 | 9

export interface TunerConfig {
  sampleRate: number
  frameSize: number
  hopSize: number
}
