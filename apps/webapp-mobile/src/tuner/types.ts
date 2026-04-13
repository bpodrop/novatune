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
  stringName: string | null
  mode: 'preset' | 'chromatic'
  uiState: Extract<TunerState, 'no_signal' | 'searching' | 'unstable' | 'too_low' | 'in_tune' | 'too_high'>
}

export interface TunerConfig {
  sampleRate: number
  frameSize: number
  hopSize: number
}
