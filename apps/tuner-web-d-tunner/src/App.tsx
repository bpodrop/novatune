import { useEffect, useMemo, useRef, useState } from 'react'
import './App.css'
import { startMicrophoneStream } from './audio/microphone'
import {
  createWasmTunerSession,
  type DetectionResult,
  loadBridge,
  type TunerState,
} from './tuner'

type PermissionState = 'idle' | 'granted' | 'denied' | 'unsupported'
type ViewId = 'tuner' | 'presets' | 'calibration' | 'settings'
type PresetId = 'standard_e' | 'drop_d' | 'half_step'
type PresetString = { label: string; frequencyHz: number }
type PresetProfile = { id: PresetId; label: string; strings: PresetString[] }

const VIEWS: Array<{ id: ViewId; label: string }> = [
  { id: 'tuner', label: 'Tuner' },
  { id: 'presets', label: 'Presets' },
  { id: 'calibration', label: 'Calibrate' },
  { id: 'settings', label: 'Settings' },
]

const PRESETS: Array<{ id: PresetId; label: string }> = [
  { id: 'standard_e', label: 'STANDARD E' },
  { id: 'drop_d', label: 'DROP D' },
  { id: 'half_step', label: 'HALF STEP' },
]

const BRIDGE_PRESET_IDS: Record<PresetId, string> = {
  standard_e: 'e-standard',
  drop_d: 'drop-d',
  half_step: 'eb-standard',
}

const PRESET_PROFILES: Record<PresetId, PresetProfile> = {
  standard_e: {
    id: 'standard_e',
    label: 'STANDARD E',
    strings: [
      { label: 'E2', frequencyHz: 82.41 },
      { label: 'A2', frequencyHz: 110.0 },
      { label: 'D3', frequencyHz: 146.83 },
      { label: 'G3', frequencyHz: 196.0 },
      { label: 'B3', frequencyHz: 246.94 },
      { label: 'E4', frequencyHz: 329.63 },
    ],
  },
  drop_d: {
    id: 'drop_d',
    label: 'DROP D',
    strings: [
      { label: 'D2', frequencyHz: 73.42 },
      { label: 'A2', frequencyHz: 110.0 },
      { label: 'D3', frequencyHz: 146.83 },
      { label: 'G3', frequencyHz: 196.0 },
      { label: 'B3', frequencyHz: 246.94 },
      { label: 'E4', frequencyHz: 329.63 },
    ],
  },
  half_step: {
    id: 'half_step',
    label: 'HALF STEP',
    strings: [
      { label: 'Eb2', frequencyHz: 77.78 },
      { label: 'Ab2', frequencyHz: 103.83 },
      { label: 'Db3', frequencyHz: 138.59 },
      { label: 'Gb3', frequencyHz: 185.0 },
      { label: 'Bb3', frequencyHz: 233.08 },
      { label: 'Eb4', frequencyHz: 311.13 },
    ],
  },
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}

function TopAppBar({ onOpenSettings }: { onOpenSettings: () => void }) {
  return (
    <header className="top-app-bar">
      <span className="brand">SONIC EDGE</span>
      <button
        className="ghost-icon-button"
        type="button"
        aria-label="Settings"
        onClick={onOpenSettings}
      >
        SETTINGS
      </button>
    </header>
  )
}

function ChromaticGauge({ centsOff }: { centsOff: number | null }) {
  const cents = centsOff ?? 0
  const markerPos = ((clamp(cents, -50, 50) + 50) / 100) * 100
  const inTune = Math.abs(cents) <= 3

  return (
    <section className="chromatic-gauge" aria-label="Chromatic tuning gauge">
      <div className="gauge-markers">
        <span>-50</span>
        <span>-25</span>
        <span>0</span>
        <span>+25</span>
        <span>+50</span>
      </div>
      <div className="gauge-rail">
        <span className="target-line" />
        <span
          className={inTune ? 'pitch-indicator in-tune' : 'pitch-indicator off-pitch'}
          style={{ left: `${markerPos}%` }}
        />
      </div>
    </section>
  )
}

function NoteDisplay({ noteName, centsOff }: { noteName: string; centsOff: number | null }) {
  return (
    <section className="note-display">
      <p className="note-display-main">{noteName}</p>
      <p className="note-display-offset">{centsOff === null ? '--' : `${centsOff >= 0 ? '+' : ''}${centsOff.toFixed(1)}`}</p>
    </section>
  )
}

function SignalInfo({
  frequencyHz,
  centsOff,
}: {
  frequencyHz: number | null
  centsOff: number | null
}) {
  return (
    <section className="signal-info">
      <article className="signal-card signal-card-green">
        <p className="signal-label">Frequency</p>
        <p className="signal-value">{frequencyHz === null ? '--' : `${frequencyHz.toFixed(2)} Hz`}</p>
      </article>
      <article className="signal-card signal-card-cyan">
        <p className="signal-label">Tolerance</p>
        <p className="signal-value">{centsOff === null ? '±-- ct' : `±${Math.abs(centsOff).toFixed(2)} ct`}</p>
      </article>
    </section>
  )
}

function TuningPresetStrip({
  selectedPreset,
  onSelect,
}: {
  selectedPreset: PresetId
  onSelect: (preset: PresetId) => void
}) {
  return (
    <section className="preset-strip" aria-label="Tuning presets">
      {PRESETS.map((preset) => (
        <button
          key={preset.id}
          className={selectedPreset === preset.id ? 'preset-button active' : 'preset-button'}
          type="button"
          onClick={() => onSelect(preset.id)}
        >
          {preset.label}
        </button>
      ))}
    </section>
  )
}

function SpectralGraph({
  frequencyHz,
  confidence,
  clarity,
}: {
  frequencyHz: number | null
  confidence: number | null
  clarity: number | null
}) {
  const bars = useMemo(() => {
    const safeConfidence = confidence ?? 0
    const safeClarity = clarity ?? 0
    const center = frequencyHz === null ? 11 : clamp(((frequencyHz % 120) / 120) * 23, 0, 23)
    const phase = frequencyHz === null ? 0 : frequencyHz / 13

    return Array.from({ length: 24 }, (_, index) => {
      const spread = Math.exp(-Math.pow(index - center, 2) / 18)
      const wobble = (Math.sin(index * 0.9 + phase) + 1) * 0.5
      const height = 0.15 + spread * (0.7 + safeConfidence * 0.5) + wobble * safeClarity * 0.25
      return clamp(height, 0.08, 1)
    })
  }, [clarity, confidence, frequencyHz])

  return (
    <section className="spectral-graph">
      <header className="spectral-title">
        <span className="live-dot" />
        LIVE INPUT • SPECTRAL ANALYSIS
      </header>
      <div className="spectral-bars" aria-hidden="true">
        {bars.map((height, index) => (
          <span key={index} className="spectral-bar" style={{ height: `${Math.round(height * 100)}%` }} />
        ))}
      </div>
    </section>
  )
}

function PresetsView({
  selectedPreset,
  onSelect,
  calibrationHz,
}: {
  selectedPreset: PresetId
  onSelect: (preset: PresetId) => void
  calibrationHz: number
}) {
  const ratio = calibrationHz / 440
  return (
    <section className="panel stack">
      <h2 className="panel-title">Tuning Presets</h2>
      <p className="panel-copy">Choose a base profile for fast stage switching.</p>
      <div className="preset-grid">
        {PRESETS.map((preset) => (
          <button
            key={preset.id}
            className={selectedPreset === preset.id ? 'preset-tile active' : 'preset-tile'}
            type="button"
            onClick={() => onSelect(preset.id)}
          >
            <span>{preset.label}</span>
            <small>
              {PRESET_PROFILES[preset.id].strings.map((item) => `${item.label} ${(
                item.frequencyHz * ratio
              ).toFixed(1)}Hz`).join(' • ')}
            </small>
          </button>
        ))}
      </div>
    </section>
  )
}

function CalibrationView({
  calibrationHz,
  setCalibrationHz,
}: {
  calibrationHz: number
  setCalibrationHz: (value: number) => void
}) {
  return (
    <section className="panel stack">
      <h2 className="panel-title">Calibration</h2>
      <p className="panel-copy">Reference pitch for A4.</p>
      <div className="calibration-readout">{calibrationHz} Hz</div>
      <input
        className="calibration-slider"
        type="range"
        min={432}
        max={448}
        step={1}
        value={calibrationHz}
        onChange={(event) => setCalibrationHz(Number(event.target.value))}
      />
      <div className="calibration-steps">
        {[432, 436, 440, 444, 448].map((value) => (
          <button key={value} type="button" className="chip" onClick={() => setCalibrationHz(value)}>
            {value}
          </button>
        ))}
      </div>
    </section>
  )
}

function SettingsView({
  haptics,
  setHaptics,
  keepAwake,
  setKeepAwake,
}: {
  haptics: boolean
  setHaptics: (value: boolean) => void
  keepAwake: boolean
  setKeepAwake: (value: boolean) => void
}) {
  return (
    <section className="panel stack">
      <h2 className="panel-title">Settings</h2>
      <label className="toggle-row">
        <span>Haptic feedback</span>
        <input type="checkbox" checked={haptics} onChange={(e) => setHaptics(e.target.checked)} />
      </label>
      <label className="toggle-row">
        <span>Keep screen awake</span>
        <input
          type="checkbox"
          checked={keepAwake}
          onChange={(e) => setKeepAwake(e.target.checked)}
        />
      </label>
    </section>
  )
}

function App() {
  const [activeView, setActiveView] = useState<ViewId>('tuner')
  const [selectedPreset, setSelectedPreset] = useState<PresetId>('standard_e')
  const [calibrationHz, setCalibrationHz] = useState(440)
  const [haptics, setHaptics] = useState(true)
  const [keepAwake, setKeepAwake] = useState(false)
  const [state, setState] = useState<TunerState>('no_signal')
  const [detection, setDetection] = useState<DetectionResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [permission, setPermission] = useState<PermissionState>('idle')
  const [running, setRunning] = useState(false)

  const frameSize = useMemo(() => 2048, [])
  const hopSize = useMemo(() => 512, [])
  const sessionRef = useRef<ReturnType<typeof createWasmTunerSession> | null>(null)
  const selectedPresetRef = useRef<PresetId>(selectedPreset)
  const calibrationRef = useRef<number>(calibrationHz)

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
        session.setPreset(BRIDGE_PRESET_IDS[selectedPresetRef.current])
        session.setCalibrationHz(calibrationRef.current)
        sessionRef.current = session

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
      sessionRef.current = null
    }
  }, [frameSize, hopSize, running])

  useEffect(() => {
    selectedPresetRef.current = selectedPreset
    calibrationRef.current = calibrationHz
    const session = sessionRef.current
    if (!session) {
      return
    }
    try {
      session.setPreset(BRIDGE_PRESET_IDS[selectedPreset])
      session.setCalibrationHz(calibrationHz)
    } catch (error) {
      console.error('Failed to sync tuning config to wasm session', error)
    }
  }, [calibrationHz, selectedPreset])

  return (
    <main className="app-shell">
      <TopAppBar onOpenSettings={() => setActiveView('settings')} />

      <section className="panel" hidden={activeView !== 'tuner'}>
        <p className="status-row">
          <span>{running ? 'LISTENING' : 'IDLE'}</span>
          <span>{state.toUpperCase()}</span>
          <span>{permission.toUpperCase()}</span>
        </p>

        <ChromaticGauge centsOff={detection?.centsOff ?? null} />
        <NoteDisplay noteName={detection?.noteName ?? '--'} centsOff={detection?.centsOff ?? null} />
        <SignalInfo frequencyHz={detection?.frequencyHz ?? null} centsOff={detection?.centsOff ?? null} />
        <TuningPresetStrip selectedPreset={selectedPreset} onSelect={setSelectedPreset} />
        <SpectralGraph
          frequencyHz={detection?.frequencyHz ?? null}
          confidence={detection?.confidence ?? null}
          clarity={detection?.clarity ?? null}
        />

        <button
          className="primary"
          type="button"
          onClick={() => {
            setError(null)
            setRunning((value) => !value)
            setState('no_signal')
            setDetection(null)
          }}
        >
          {running ? 'Stop microphone' : 'Start microphone'}
        </button>

        {error ? <p className="error">Error: {error}</p> : null}
      </section>

      {activeView === 'presets' ? (
        <PresetsView
          selectedPreset={selectedPreset}
          onSelect={setSelectedPreset}
          calibrationHz={calibrationHz}
        />
      ) : null}

      {activeView === 'calibration' ? (
        <CalibrationView calibrationHz={calibrationHz} setCalibrationHz={setCalibrationHz} />
      ) : null}

      {activeView === 'settings' ? (
        <SettingsView
          haptics={haptics}
          setHaptics={setHaptics}
          keepAwake={keepAwake}
          setKeepAwake={setKeepAwake}
        />
      ) : null}

      <nav className="bottom-nav" aria-label="Primary">
        {VIEWS.map((view) => (
          <button
            key={view.id}
            type="button"
            className={activeView === view.id ? 'nav-item active' : 'nav-item'}
            onClick={() => setActiveView(view.id)}
          >
            {view.label}
          </button>
        ))}
      </nav>
    </main>
  )
}

export default App
