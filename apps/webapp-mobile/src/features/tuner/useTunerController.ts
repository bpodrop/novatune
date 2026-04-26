import { useEffect, useMemo, useRef, useState } from 'react'
import { startMicrophoneStream } from '../../audio/microphone'
import {
  createWasmTunerSession,
  type DetectionResult,
  listAvailablePresets,
  loadBridge,
  type PresetFamilyFilter,
  type TuningPresetProfile,
  type TunerState,
} from '../../tuner'
import {
  resolvePresetIdForFamily,
  shouldShowPresetStrings,
} from '../../tuner/presets'

export type PermissionState = 'idle' | 'granted' | 'denied' | 'unsupported'
export type TuningMode = 'preset' | 'chromatic'
export interface TunerSelectionControls {
  selectedPreset: string | null
  setSelectedPreset: (value: string | null) => void
  presetFamilyFilter: PresetFamilyFilter
  setPresetFamilyFilter: (value: PresetFamilyFilter) => void
  tuningMode: TuningMode
  setTuningMode: (value: TuningMode) => void
  selectedPresetMeta: TuningPresetProfile | null
  activeStringIndex: number | null
}
export interface TunerSettingsControls {
  calibrationHz: number
  setCalibrationHz: (value: number) => void
  haptics: boolean
  setHaptics: (value: boolean) => void
  keepAwake: boolean
  setKeepAwake: (value: boolean) => void
  presetMatchWindow: number
  setPresetMatchWindow: (value: number) => void
  minRms: number
  setMinRms: (value: number) => void
  minClarity: number
  setMinClarity: (value: number) => void
}
export interface TunerRuntimeState {
  state: TunerState
  detection: DetectionResult | null
  error: string | null
  permission: PermissionState
  running: boolean
  toggleRunning: () => void
}
export interface TunerController {
  presets: readonly TuningPresetProfile[]
  selection: TunerSelectionControls
  settings: TunerSettingsControls
  runtime: TunerRuntimeState
}

const DEFAULT_MIN_RMS = 0.01
const DEFAULT_MIN_CLARITY = 0.6
const DEFAULT_PRESET_MATCH_WINDOW_CENTS = 100
const FRAME_SIZE = 2048
const HOP_SIZE = 512

function presetById(presets: readonly TuningPresetProfile[], presetId: string | null) {
  if (!presetId) {
    return null
  }
  return presets.find((preset) => preset.id === presetId) ?? null
}

export function useTunerController(): TunerController {
  const [presets, setPresets] = useState<TuningPresetProfile[]>([])
  const [selectedPreset, setSelectedPreset] = useState<string | null>(null)
  const [presetFamilyFilter, setPresetFamilyFilter] = useState<PresetFamilyFilter>('all')
  const [tuningMode, setTuningMode] = useState<TuningMode>('chromatic')
  const [calibrationHz, setCalibrationHz] = useState(440)
  const [haptics, setHaptics] = useState(true)
  const [keepAwake, setKeepAwake] = useState(false)
  const [presetMatchWindow, setPresetMatchWindow] = useState(DEFAULT_PRESET_MATCH_WINDOW_CENTS)
  const [minRms, setMinRms] = useState(DEFAULT_MIN_RMS)
  const [minClarity, setMinClarity] = useState(DEFAULT_MIN_CLARITY)
  const [state, setState] = useState<TunerState>('no_signal')
  const [detection, setDetection] = useState<DetectionResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [permission, setPermission] = useState<PermissionState>('idle')
  const [running, setRunning] = useState(false)

  const sessionRef = useRef<ReturnType<typeof createWasmTunerSession> | null>(null)
  const selectedPresetRef = useRef<string | null>(selectedPreset)
  const presetFamilyFilterRef = useRef<PresetFamilyFilter>(presetFamilyFilter)
  const presetsRef = useRef<TuningPresetProfile[]>(presets)
  const tuningModeRef = useRef<TuningMode>(tuningMode)
  const calibrationRef = useRef<number>(calibrationHz)
  const presetMatchWindowRef = useRef<number>(presetMatchWindow)
  const minRmsRef = useRef<number>(minRms)
  const minClarityRef = useRef<number>(minClarity)

  const selectedPresetMeta = useMemo(
    () => presetById(presets, selectedPreset),
    [presets, selectedPreset],
  )

  const activeStringIndex = useMemo(() => {
    if (
      detection &&
      shouldShowPresetStrings(tuningMode, detection.mode) &&
      (detection.tuningProfileId === null || detection.tuningProfileId === selectedPresetMeta?.id)
    ) {
      return detection.stringIndex
    }
    return null
  }, [detection, selectedPresetMeta?.id, tuningMode])

  useEffect(() => {
    let active = true

    async function syncPresets() {
      try {
        const bridge = await loadBridge()
        if (!active) {
          return
        }
        const bridgePresets = listAvailablePresets(bridge)
        if (bridgePresets.length === 0) {
          setError('No tuning presets returned by WASM bridge')
          return
        }
        setPresets(bridgePresets)
        setSelectedPreset((current) =>
          resolvePresetIdForFamily(bridgePresets, current, presetFamilyFilterRef.current),
        )
      } catch (syncError) {
        const message = syncError instanceof Error ? syncError.message : String(syncError)
        if (active) {
          setError(message)
        }
      }
    }

    void syncPresets()

    return () => {
      active = false
    }
  }, [])

  useEffect(() => {
    presetsRef.current = presets
  }, [presets])

  useEffect(() => {
    presetFamilyFilterRef.current = presetFamilyFilter
    setSelectedPreset((current) => resolvePresetIdForFamily(presets, current, presetFamilyFilter))
  }, [presetFamilyFilter, presets])

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

        const audio = await startMicrophoneStream(FRAME_SIZE, (samples) => {
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
        stopAudio = () => audio.stop()

        setPermission('granted')
        session = createWasmTunerSession({
          bridge,
          sampleRate: audio.sampleRate,
          frameSize: FRAME_SIZE,
          hopSize: HOP_SIZE,
        })
        const bridgePresets = listAvailablePresets(bridge)
        if (bridgePresets.length > 0) {
          setPresets(bridgePresets)
        }
        const presetId = resolvePresetIdForFamily(
          bridgePresets.length > 0 ? bridgePresets : presetsRef.current,
          selectedPresetRef.current,
          presetFamilyFilterRef.current,
        )
        if (!presetId) {
          throw new Error('No preset available to start tuner session')
        }
        selectedPresetRef.current = presetId
        setSelectedPreset(presetId)
        session.setMode(tuningModeRef.current)
        session.setPreset(presetId)
        session.setPresetMatchWindowCents(presetMatchWindowRef.current)
        session.setCalibrationHz(calibrationRef.current)
        session.setMinRms(minRmsRef.current)
        session.setMinClarity(minClarityRef.current)
        sessionRef.current = session

        closeSession = () => session?.close()
        setState('searching')
      } catch (startError) {
        stopAudio?.()
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
  }, [running])

  useEffect(() => {
    selectedPresetRef.current = selectedPreset
    tuningModeRef.current = tuningMode
    calibrationRef.current = calibrationHz
    presetMatchWindowRef.current = presetMatchWindow
    minRmsRef.current = minRms
    minClarityRef.current = minClarity
    const session = sessionRef.current
    if (!session || !selectedPreset) {
      return
    }
    try {
      session.setMode(tuningMode)
      session.setPreset(selectedPreset)
      session.setPresetMatchWindowCents(presetMatchWindow)
      session.setCalibrationHz(calibrationHz)
      session.setMinRms(minRms)
      session.setMinClarity(minClarity)
    } catch (syncError) {
      console.error('Failed to sync tuning config to wasm session', syncError)
    }
  }, [calibrationHz, minClarity, minRms, presetMatchWindow, selectedPreset, tuningMode])

  function toggleRunning() {
    setError(null)
    setRunning((value) => !value)
    setState('no_signal')
    setDetection(null)
  }

  return {
    presets,
    selection: {
      selectedPreset,
      setSelectedPreset,
      presetFamilyFilter,
      setPresetFamilyFilter,
      tuningMode,
      setTuningMode,
      selectedPresetMeta,
      activeStringIndex,
    },
    settings: {
      calibrationHz,
      setCalibrationHz,
      haptics,
      setHaptics,
      keepAwake,
      setKeepAwake,
      presetMatchWindow,
      setPresetMatchWindow,
      minRms,
      setMinRms,
      minClarity,
      setMinClarity,
    },
    runtime: {
      state,
      detection,
      error,
      permission,
      running,
      toggleRunning,
    },
  }
}
