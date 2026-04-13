import init, {
  new_detector,
  next_output,
  push_samples,
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
    reset,
    shutdown,
  }
}
