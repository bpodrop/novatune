import init, {
  new_detector,
  next_output,
  push_samples,
  set_calibration_hz,
  set_min_clarity,
  set_min_rms,
  set_preset,
  reset,
  shutdown,
} from './wasm/pkg/tuner_web_bridge.js'

import type { BridgeApi } from './wasmTuner'

let didInit = false

export async function loadBridge(): Promise<BridgeApi> {
  if (!didInit) {
    await init()
    didInit = true
  }

  return {
    new_detector,
    next_output,
    push_samples,
    set_calibration_hz,
    set_min_clarity,
    set_min_rms,
    set_preset,
    reset,
    shutdown,
  }
}
