import { useEffect, useMemo, useState } from 'react'
import './App.css'
import { startMicrophoneStream } from './audio/microphone'
import {
  createWasmTunerSession,
  type DetectionResult,
  loadBridge,
  type TunerState,
} from './tuner'

type PermissionState = 'idle' | 'granted' | 'denied' | 'unsupported'

function App() {
  const [state, setState] = useState<TunerState>('no_signal')
  const [detection, setDetection] = useState<DetectionResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [permission, setPermission] = useState<PermissionState>('idle')
  const [running, setRunning] = useState(false)

  const frameSize = useMemo(() => 2048, [])
  const hopSize = useMemo(() => 512, [])

  useEffect(() => {
    if (!running) {
      return undefined
    }

    let stopAudio: (() => void) | undefined
    let closeSession: (() => void) | undefined
    let active = true
    let session: ReturnType<typeof createWasmTunerSession> | undefined

    async function start() {
      try {
        const bridge = await loadBridge()
        if (!active) {
          return
        }

        const audio = await startMicrophoneStream(frameSize, (samples) => {
          if (!active || !session) {
            return
          }
          const results = session.ingestSamples(samples)
          if (results.length === 0) {
            setState('searching')
            return
          }
          const last = results[results.length - 1]
          setDetection(last)
          setState(last.uiState)
        })

        if (!active) {
          audio.stop()
          return
        }

        setPermission('granted')
        session = createWasmTunerSession({
          bridge,
          sampleRate: audio.sampleRate,
          frameSize,
          hopSize,
        })

        closeSession = () => session?.close()
        stopAudio = () => audio.stop()
        setState('searching')
      } catch (startError) {
        const message = startError instanceof Error ? startError.message : String(startError)
        if (message.toLowerCase().includes('denied') || message.toLowerCase().includes('permission')) {
          setPermission('denied')
        } else if (message.toLowerCase().includes('unavailable')) {
          setPermission('unsupported')
        }
        setRunning(false)
        setError(message)
      }
    }

    void start()

    return () => {
      active = false
      stopAudio?.()
      closeSession?.()
    }
  }, [frameSize, hopSize, running])

  return (
    <main className="app-shell">
      <h1>Tuner Web</h1>
      <p>UI state: {state}</p>
      <p>Microphone: {permission}</p>
      <p>
        <button type="button" onClick={() => {
          setError(null)
          setRunning((value) => !value)
          setState('no_signal')
          setDetection(null)
        }}>
          {running ? 'Stop microphone' : 'Start microphone'}
        </button>
      </p>
      {detection ? (
        <dl>
          <dt>Note</dt>
          <dd>{detection.noteName}</dd>
          <dt>Frequency</dt>
          <dd>{detection.frequencyHz.toFixed(2)} Hz</dd>
          <dt>Cents</dt>
          <dd>{detection.centsOff.toFixed(1)}</dd>
          <dt>State</dt>
          <dd>{detection.uiState}</dd>
          <dt>Confidence</dt>
          <dd>{detection.confidence.toFixed(2)}</dd>
          <dt>Clarity</dt>
          <dd>{detection.clarity.toFixed(2)}</dd>
          <dt>RMS</dt>
          <dd>{detection.rms.toFixed(2)}</dd>
        </dl>
      ) : (
        <p>No output yet.</p>
      )}
      {error ? <p className="error">Error: {error}</p> : null}
    </main>
  )
}

export default App
