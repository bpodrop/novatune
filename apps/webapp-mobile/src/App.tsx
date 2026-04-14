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
type ViewId = 'tuner' | 'settings'
type ThemeMode = 'dark' | 'light'
type TuningMode = 'preset' | 'chromatic'

const DEFAULT_MIN_RMS = 0.01
const DEFAULT_MIN_CLARITY = 0.6
const APP_VERSION = 'V0.1.0'
const GITHUB_URL = 'https://github.com/bpodrop/novatune'

const PRESETS = [
  {
    id: 'standard_e',
    label: 'E STANDARD (6)',
    bridgeId: 'e-standard',
    strings: ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
    stringCount: 6,
  },
  {
    id: 'drop_d',
    label: 'DROP D (6)',
    bridgeId: 'drop-d',
    strings: ['D2', 'A2', 'D3', 'G3', 'B3', 'E4'],
    stringCount: 6,
  },
  {
    id: 'half_step',
    label: 'EB STANDARD (6)',
    bridgeId: 'eb-standard',
    strings: ['Eb2', 'Ab2', 'Db3', 'Gb3', 'Bb3', 'Eb4'],
    stringCount: 6,
  },
  {
    id: 'd_standard',
    label: 'D STANDARD (6)',
    bridgeId: 'd-standard',
    strings: ['D2', 'G2', 'C3', 'F3', 'A3', 'D4'],
    stringCount: 6,
  },
  {
    id: 'drop_c',
    label: 'DROP C (6)',
    bridgeId: 'drop-c',
    strings: ['C2', 'G2', 'C3', 'F3', 'A3', 'D4'],
    stringCount: 6,
  },
  {
    id: 'open_g',
    label: 'OPEN G (6)',
    bridgeId: 'open-g',
    strings: ['D2', 'G2', 'D3', 'G3', 'B3', 'D4'],
    stringCount: 6,
  },
  {
    id: 'open_d',
    label: 'OPEN D (6)',
    bridgeId: 'open-d',
    strings: ['D2', 'A2', 'D3', 'F#3', 'A3', 'D4'],
    stringCount: 6,
  },
  {
    id: 'dadgad',
    label: 'DADGAD (6)',
    bridgeId: 'dadgad',
    strings: ['D2', 'A2', 'D3', 'G3', 'A3', 'D4'],
    stringCount: 6,
  },
  {
    id: 'b_standard_7',
    label: 'B STANDARD (7)',
    bridgeId: 'b-standard-7',
    strings: ['B1', 'E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
    stringCount: 7,
  },
  {
    id: 'drop_a_7',
    label: 'DROP A (7)',
    bridgeId: 'drop-a-7',
    strings: ['A1', 'E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
    stringCount: 7,
  },
  {
    id: 'a_standard_7',
    label: 'A STANDARD (7)',
    bridgeId: 'a-standard-7',
    strings: ['A1', 'D2', 'G2', 'C3', 'F3', 'A3', 'D4'],
    stringCount: 7,
  },
  {
    id: 'fsharp_standard_8',
    label: 'F# STANDARD (8)',
    bridgeId: 'fsharp-standard-8',
    strings: ['F#1', 'B1', 'E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
    stringCount: 8,
  },
  {
    id: 'e_standard_8',
    label: 'E STANDARD (8)',
    bridgeId: 'e-standard-8',
    strings: ['E1', 'A1', 'D2', 'G2', 'C3', 'F3', 'A3', 'D4'],
    stringCount: 8,
  },
  {
    id: 'csharp_standard_9',
    label: 'C# STANDARD (9)',
    bridgeId: 'csharp-standard-9',
    strings: ['C#1', 'F#1', 'B1', 'E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
    stringCount: 9,
  },
  {
    id: 'drop_b_9',
    label: 'DROP B (9)',
    bridgeId: 'drop-b-9',
    strings: ['B0', 'F#1', 'B1', 'E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
    stringCount: 9,
  },
] as const

type PresetId = (typeof PRESETS)[number]['id']


function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}

function presetById(presetId: PresetId) {
  return PRESETS.find((preset) => preset.id === presetId) ?? PRESETS[0]
}

function TopAppBar({ onOpenSettings }: { onOpenSettings: () => void }) {
  return (
    <header className="top-app-bar">
      <span className="brand">NovaTuner</span>
      <button
        className="ghost-icon-button"
        type="button"
        aria-label="Settings"
        onClick={onOpenSettings}
      >
        <svg
          className="gear-icon"
          viewBox="0 0 24 24"
          width="18"
          height="18"
          aria-hidden="true"
          focusable="false"
        >
          <path
            d="M19.14 12.94c.04-.31.06-.63.06-.94s-.02-.63-.06-.94l2.03-1.58a.5.5 0 0 0 .12-.64l-1.92-3.32a.5.5 0 0 0-.6-.22l-2.39.96a7.28 7.28 0 0 0-1.63-.94l-.36-2.54a.5.5 0 0 0-.5-.42h-3.84a.5.5 0 0 0-.5.42l-.36 2.54c-.58.22-1.13.54-1.63.94l-2.39-.96a.5.5 0 0 0-.6.22L2.71 8.84a.5.5 0 0 0 .12.64l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58a.5.5 0 0 0-.12.64l1.92 3.32a.5.5 0 0 0 .6.22l2.39-.96c.5.4 1.05.72 1.63.94l.36 2.54a.5.5 0 0 0 .5.42h3.84a.5.5 0 0 0 .5-.42l.36-2.54c.58-.22 1.13-.54 1.63-.94l2.39.96a.5.5 0 0 0 .6-.22l1.92-3.32a.5.5 0 0 0-.12-.64l-2.03-1.58ZM12 15.5A3.5 3.5 0 1 1 12 8.5a3.5 3.5 0 0 1 0 7Z"
            fill="currentColor"
          />
        </svg>
      </button>
    </header>
  )
}

function ChromaticGauge({ centsOff }: { centsOff: number | null }) {
  const gaugeMin = -10
  const gaugeMax = 10
  const gaugeSpan = gaugeMax - gaugeMin
  const hasSignal = centsOff !== null
  const cents = clamp(centsOff ?? 0, gaugeMin, gaugeMax)
  const absolute = Math.abs(cents)
  const inTune = hasSignal && absolute <= 3
  const spread = hasSignal ? (absolute / (gaugeSpan / 2)) * 44 : 0
  const leftMarkerPos = 50 - spread
  const rightMarkerPos = 50 + spread

  const stateClass = !hasSignal ? 'no-signal' : inTune ? 'in-tune' : cents < 0 ? 'too-low' : 'too-high'
  const guidanceArrow = !hasSignal ? '•' : inTune ? '✓' : cents < 0 ? '↑' : '↓'
  const guidanceLabel = !hasSignal
    ? 'Waiting signal'
    : inTune
      ? 'In tune'
      : cents < 0
        ? 'Too low · tighten'
        : 'Too high · loosen'

  return (
    <section className="chromatic-gauge" aria-label="Chromatic tuning gauge">
      <div className="gauge-markers">
        <span>-10</span>
        <span>-5</span>
        <span>0</span>
        <span>+5</span>
        <span>+10</span>
      </div>
      <div className="gauge-rail">
        <span className="target-line" />
        <span
          className={`gauge-indicator gauge-indicator-left ${stateClass}`}
          style={{ left: `${leftMarkerPos}%` }}
        />
        <span
          className={`gauge-indicator gauge-indicator-right ${stateClass}`}
          style={{ left: `${rightMarkerPos}%` }}
        />
      </div>
      <div className={`gauge-guidance ${stateClass}`} aria-live="polite">
        <span className="gauge-guidance-arrow" aria-hidden="true">
          {guidanceArrow}
        </span>
        <span>{guidanceLabel}</span>
      </div>
    </section>
  )
}

function NoteDisplay({
  noteName,
  centsOff,
  strings,
  stringIndex,
  tuningMode,
}: {
  noteName: string
  centsOff: number | null
  strings: readonly string[]
  stringIndex: number | null
  tuningMode: TuningMode
}) {
  const hasActiveString = stringIndex !== null && stringIndex >= 0 && stringIndex < strings.length

  return (
    <section className="note-display">
      <p className="note-display-main">{noteName}</p>
      <p className="note-display-offset">
        {centsOff === null ? '-- ct' : `${centsOff >= 0 ? '+' : ''}${centsOff.toFixed(1)} ct`}
      </p>
      {tuningMode === 'preset' ? (
        <div
          className={hasActiveString ? 'note-display-strings' : 'note-display-strings unmatched'}
          role="list"
          aria-label="Preset strings"
        >
          {strings.map((stringLabel, index) => (
            <span
              key={`${stringLabel}-${index}`}
              role="listitem"
              className={
                index === stringIndex ? 'note-display-string-chip active' : 'note-display-string-chip'
              }
            >
              {stringLabel}
            </span>
          ))}
        </div>
      ) : (
        <p className="note-display-string">Chromatic</p>
      )}
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
    <section className="signal-summary">
      <article className="signal-metric signal-metric-frequency">
        <p className="signal-label">Frequency</p>
        <p className="signal-value">{frequencyHz === null ? '--' : `${frequencyHz.toFixed(2)} Hz`}</p>
      </article>
      <article className="signal-metric signal-metric-tolerance">
        <p className="signal-label">Tolerance</p>
        <p className="signal-value">{centsOff === null ? '±-- ct' : `±${Math.abs(centsOff).toFixed(2)} ct`}</p>
      </article>
    </section>
  )
}

function TuningPresetPicker({
  tuningMode,
  onModeChange,
  selectedPreset,
  onSelect,
}: {
  tuningMode: TuningMode
  onModeChange: (mode: TuningMode) => void
  selectedPreset: PresetId
  onSelect: (preset: PresetId) => void
}) {
  const [open, setOpen] = useState(false)
  const activeLabel =
    tuningMode === 'chromatic'
      ? 'CHROMATIC'
      : presetById(selectedPreset).label

  return (
    <section className="preset-picker">
      <button
        type="button"
        className="preset-picker-trigger"
        aria-label="Choose tuning preset"
        onClick={() => setOpen(true)}
      >
        <span>{activeLabel}</span>
        <span aria-hidden="true">▾</span>
      </button>
      {open ? (
        <div className="preset-sheet-backdrop" role="dialog" aria-modal="true" aria-label="Tuning presets">
          <section className="preset-sheet">
            <header className="preset-sheet-header">
              <h3 className="panel-title">Tuning Presets</h3>
              <button
                type="button"
                className="settings-close-button"
                aria-label="Close presets"
                onClick={() => setOpen(false)}
              >
                <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false">
                  <path
                    d="M6.23 6.23a.75.75 0 0 1 1.06 0L12 10.94l4.71-4.71a.75.75 0 1 1 1.06 1.06L13.06 12l4.71 4.71a.75.75 0 0 1-1.06 1.06L12 13.06l-4.71 4.71a.75.75 0 0 1-1.06-1.06L10.94 12 6.23 7.29a.75.75 0 0 1 0-1.06Z"
                    fill="currentColor"
                  />
                </svg>
              </button>
            </header>
            <div className="mode-toggle-row" role="group" aria-label="Tuning mode">
              <button
                type="button"
                className={tuningMode === 'preset' ? 'mode-toggle-button active' : 'mode-toggle-button'}
                onClick={() => onModeChange('preset')}
              >
                Preset
              </button>
              <button
                type="button"
                className={tuningMode === 'chromatic' ? 'mode-toggle-button active' : 'mode-toggle-button'}
                onClick={() => onModeChange('chromatic')}
              >
                Chromatic
              </button>
            </div>
            <div className="preset-sheet-list">
              {PRESETS.map((preset) => (
                <button
                  key={preset.id}
                  className={selectedPreset === preset.id ? 'preset-button active' : 'preset-button'}
                  type="button"
                  onClick={() => {
                    onModeChange('preset')
                    onSelect(preset.id)
                    setOpen(false)
                  }}
                >
                  {preset.label}
                </button>
              ))}
            </div>
          </section>
        </div>
      ) : null}
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

function SettingsView({
  onClose,
  theme,
  setTheme,
  calibrationHz,
  setCalibrationHz,
  haptics,
  setHaptics,
  keepAwake,
  setKeepAwake,
  minRms,
  setMinRms,
  minClarity,
  setMinClarity,
}: {
  onClose: () => void
  theme: ThemeMode
  setTheme: (value: ThemeMode) => void
  calibrationHz: number
  setCalibrationHz: (value: number) => void
  haptics: boolean
  setHaptics: (value: boolean) => void
  keepAwake: boolean
  setKeepAwake: (value: boolean) => void
  minRms: number
  setMinRms: (value: number) => void
  minClarity: number
  setMinClarity: (value: number) => void
}) {
  return (
    <section className="panel stack">
      <header className="panel-heading">
        <h2 className="panel-title">Settings</h2>
        <button
          type="button"
          className="settings-close-button"
          aria-label="Close settings"
          onClick={onClose}
        >
          <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false">
            <path
              d="M6.23 6.23a.75.75 0 0 1 1.06 0L12 10.94l4.71-4.71a.75.75 0 1 1 1.06 1.06L13.06 12l4.71 4.71a.75.75 0 0 1-1.06 1.06L12 13.06l-4.71 4.71a.75.75 0 0 1-1.06-1.06L10.94 12 6.23 7.29a.75.75 0 0 1 0-1.06Z"
              fill="currentColor"
            />
          </svg>
        </button>
      </header>
      <section className="toggle-row">
        <span>Theme</span>
        <div className="theme-toggle" role="group" aria-label="Theme mode">
          <button
            type="button"
            className={theme === 'dark' ? 'theme-toggle-button active' : 'theme-toggle-button'}
            aria-label="Dark mode"
            aria-pressed={theme === 'dark'}
            onClick={() => setTheme('dark')}
          >
            <svg viewBox="0 0 24 24" width="15" height="15" aria-hidden="true" focusable="false">
              <path
                d="M12 3.5a8.5 8.5 0 1 0 8.3 10.3 7.2 7.2 0 0 1-8.8-8.8A8.4 8.4 0 0 0 12 3.5Z"
                fill="currentColor"
              />
            </svg>
          </button>
          <button
            type="button"
            className={theme === 'light' ? 'theme-toggle-button active' : 'theme-toggle-button'}
            aria-label="Light mode"
            aria-pressed={theme === 'light'}
            onClick={() => setTheme('light')}
          >
            <svg viewBox="0 0 24 24" width="15" height="15" aria-hidden="true" focusable="false">
              <circle cx="12" cy="12" r="4.3" fill="currentColor" />
              <g stroke="currentColor" strokeWidth="1.7" strokeLinecap="round">
                <line x1="12" y1="2.8" x2="12" y2="5.1" />
                <line x1="12" y1="18.9" x2="12" y2="21.2" />
                <line x1="2.8" y1="12" x2="5.1" y2="12" />
                <line x1="18.9" y1="12" x2="21.2" y2="12" />
                <line x1="5.6" y1="5.6" x2="7.3" y2="7.3" />
                <line x1="16.7" y1="16.7" x2="18.4" y2="18.4" />
                <line x1="16.7" y1="7.3" x2="18.4" y2="5.6" />
                <line x1="5.6" y1="18.4" x2="7.3" y2="16.7" />
              </g>
            </svg>
          </button>
        </div>
      </section>
      <label className="toggle-row">
        <span>Haptic feedback</span>
        <input type="checkbox" checked={haptics} onChange={(e) => setHaptics(e.target.checked)} />
      </label>
      <section className="tuning-control">
        <div className="tuning-control-header">
          <span>Calibration</span>
          <strong>{calibrationHz} Hz</strong>
        </div>
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
      <label className="toggle-row">
        <span>Keep screen awake</span>
        <input
          type="checkbox"
          checked={keepAwake}
          onChange={(e) => setKeepAwake(e.target.checked)}
        />
      </label>
      <section className="tuning-control">
        <div className="tuning-control-header">
          <span>Min RMS</span>
          <strong>{minRms.toFixed(3)}</strong>
        </div>
        <input
          className="calibration-slider"
          type="range"
          min={0}
          max={0.03}
          step={0.001}
          value={minRms}
          onChange={(event) => setMinRms(Number(event.target.value))}
        />
        <p className="tuning-control-copy">Lower values increase input sensitivity.</p>
      </section>
      <section className="tuning-control">
        <div className="tuning-control-header">
          <span>Min Clarity</span>
          <strong>{minClarity.toFixed(2)}</strong>
        </div>
        <input
          className="calibration-slider"
          type="range"
          min={0.3}
          max={0.95}
          step={0.01}
          value={minClarity}
          onChange={(event) => setMinClarity(Number(event.target.value))}
        />
        <p className="tuning-control-copy">Lower values accept noisier pitch candidates.</p>
      </section>
    </section>
  )
}

function App() {
  const [theme, setTheme] = useState<ThemeMode>(() => {
    if (typeof window === 'undefined') {
      return 'dark'
    }
    const stored = window.localStorage.getItem('novatuner.theme')
    return stored === 'light' ? 'light' : 'dark'
  })
  const [activeView, setActiveView] = useState<ViewId>('tuner')
  const [selectedPreset, setSelectedPreset] = useState<PresetId>('standard_e')
  const [tuningMode, setTuningMode] = useState<TuningMode>('preset')
  const [calibrationHz, setCalibrationHz] = useState(440)
  const [haptics, setHaptics] = useState(true)
  const [keepAwake, setKeepAwake] = useState(false)
  const [minRms, setMinRms] = useState(DEFAULT_MIN_RMS)
  const [minClarity, setMinClarity] = useState(DEFAULT_MIN_CLARITY)
  const [state, setState] = useState<TunerState>('no_signal')
  const [detection, setDetection] = useState<DetectionResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [permission, setPermission] = useState<PermissionState>('idle')
  const [running, setRunning] = useState(false)

  const frameSize = useMemo(() => 2048, [])
  const hopSize = useMemo(() => 512, [])
  const sessionRef = useRef<ReturnType<typeof createWasmTunerSession> | null>(null)
  const selectedPresetRef = useRef<PresetId>(selectedPreset)
  const tuningModeRef = useRef<TuningMode>(tuningMode)
  const calibrationRef = useRef<number>(calibrationHz)
  const minRmsRef = useRef<number>(minRms)
  const minClarityRef = useRef<number>(minClarity)
  const selectedPresetMeta = useMemo(() => presetById(selectedPreset), [selectedPreset])
  const activeStringIndex =
    tuningMode === 'preset' &&
    detection?.mode === 'preset' &&
    (detection.tuningProfileId === null || detection.tuningProfileId === selectedPresetMeta.bridgeId)
      ? detection?.stringIndex ?? null
      : null

  useEffect(() => {
    if (typeof window === 'undefined') {
      return
    }
    window.localStorage.setItem('novatuner.theme', theme)
  }, [theme])

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
        session.setMode(tuningModeRef.current)
        session.setPreset(presetById(selectedPresetRef.current).bridgeId)
        session.setCalibrationHz(calibrationRef.current)
        session.setMinRms(minRmsRef.current)
        session.setMinClarity(minClarityRef.current)
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
    tuningModeRef.current = tuningMode
    calibrationRef.current = calibrationHz
    minRmsRef.current = minRms
    minClarityRef.current = minClarity
    const session = sessionRef.current
    if (!session) {
      return
    }
    try {
      session.setMode(tuningMode)
      session.setPreset(selectedPresetMeta.bridgeId)
      session.setCalibrationHz(calibrationHz)
      session.setMinRms(minRms)
      session.setMinClarity(minClarity)
    } catch (error) {
      console.error('Failed to sync tuning config to wasm session', error)
    }
  }, [calibrationHz, minClarity, minRms, selectedPresetMeta.bridgeId, selectedPreset, tuningMode])

  return (
    <main className="app-shell" data-theme={theme}>
      <TopAppBar
        onOpenSettings={() => setActiveView((view) => (view === 'settings' ? 'tuner' : 'settings'))}
      />

      <section className="panel tuner-panel" hidden={activeView !== 'tuner'}>
        <p className="status-row">
          <span>{running ? 'LISTENING' : 'IDLE'}</span>
          <span>{state.toUpperCase()}</span>
          <span>{permission.toUpperCase()}</span>
        </p>

        <ChromaticGauge centsOff={detection?.centsOff ?? null} />
        <NoteDisplay
          noteName={detection?.noteName ?? '--'}
          centsOff={detection?.centsOff ?? null}
          strings={selectedPresetMeta.strings}
          stringIndex={activeStringIndex}
          tuningMode={tuningMode}
        />
        <SignalInfo frequencyHz={detection?.frequencyHz ?? null} centsOff={detection?.centsOff ?? null} />
        <TuningPresetPicker
          tuningMode={tuningMode}
          onModeChange={setTuningMode}
          selectedPreset={selectedPreset}
          onSelect={setSelectedPreset}
        />

        <button
          className={running ? 'primary primary-running' : 'primary'}
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

        <SpectralGraph
          frequencyHz={detection?.frequencyHz ?? null}
          confidence={detection?.confidence ?? null}
          clarity={detection?.clarity ?? null}
        />

        {error ? <p className="error">Error: {error}</p> : null}
      </section>

      {activeView === 'settings' ? (
        <SettingsView
          onClose={() => setActiveView('tuner')}
          theme={theme}
          setTheme={setTheme}
          calibrationHz={calibrationHz}
          setCalibrationHz={setCalibrationHz}
          haptics={haptics}
          setHaptics={setHaptics}
          keepAwake={keepAwake}
          setKeepAwake={setKeepAwake}
          minRms={minRms}
          setMinRms={setMinRms}
          minClarity={minClarity}
          setMinClarity={setMinClarity}
        />
      ) : null}

      <footer className="app-footer">
        <span>{APP_VERSION}</span>
        <a href={GITHUB_URL} target="_blank" rel="noreferrer" aria-label="NovaTuner GitHub repository">
          <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false">
            <path
              d="M12 .5a12 12 0 0 0-3.79 23.39c.6.11.82-.26.82-.58v-2.24c-3.34.73-4.04-1.41-4.04-1.41-.55-1.38-1.33-1.75-1.33-1.75-1.09-.74.08-.73.08-.73 1.2.09 1.84 1.21 1.84 1.21 1.07 1.81 2.8 1.29 3.48.99.11-.76.42-1.29.76-1.58-2.66-.3-5.47-1.31-5.47-5.85 0-1.29.47-2.34 1.23-3.16-.13-.3-.53-1.52.11-3.17 0 0 1-.31 3.3 1.2a11.6 11.6 0 0 1 6 0c2.29-1.51 3.29-1.2 3.29-1.2.65 1.65.25 2.87.12 3.17.77.82 1.23 1.87 1.23 3.16 0 4.56-2.81 5.54-5.49 5.84.43.37.82 1.09.82 2.21v3.28c0 .32.22.69.83.57A12 12 0 0 0 12 .5Z"
              fill="currentColor"
            />
          </svg>
        </a>
      </footer>

    </main>
  )
}

export default App
