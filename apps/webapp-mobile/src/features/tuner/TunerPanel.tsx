import type { PresetFamilyFilter, TuningPresetProfile, TuningPresetString } from '../../tuner'
import { ChromaticGauge } from './components/ChromaticGauge'
import { NoteDisplay } from './components/NoteDisplay'
import { SignalInfo } from './components/SignalInfo'
import { SpectralGraph } from './components/SpectralGraph'
import { TuningPresetPicker } from './components/TuningPresetPicker'
import type { PermissionState, TuningMode } from './useTunerController'

interface TunerPanelProps {
  active: boolean
  status: {
    running: boolean
    state: string
    permission: PermissionState
    error: string | null
  }
  display: {
    tuningMode: TuningMode
    selectedPresetLabel: string | null
    noteName: string
    centsOff: number | null
    frequencyHz: number | null
    confidence: number | null
    clarity: number | null
    strings: readonly TuningPresetString[]
    activeStringIndex: number | null
  }
  presets: {
    catalog: readonly TuningPresetProfile[]
    familyFilter: PresetFamilyFilter
    selectedPreset: string | null
    onFamilyFilterChange: (value: PresetFamilyFilter) => void
    onModeChange: (mode: TuningMode) => void
    onSelectPreset: (preset: string) => void
  }
  onToggleRunning: () => void
}

export function TunerPanel({
  active,
  status,
  display,
  presets,
  onToggleRunning,
}: TunerPanelProps) {
  const { running, state, permission, error } = status
  const {
    tuningMode,
    selectedPresetLabel,
    noteName,
    centsOff,
    frequencyHz,
    confidence,
    clarity,
    strings,
    activeStringIndex,
  } = display

  return (
    <section className="panel tuner-panel" hidden={!active}>
      <p className="status-row">
        <span>{running ? 'LISTENING' : 'IDLE'}</span>
        <span>{state.toUpperCase()}</span>
        <span>{permission.toUpperCase()}</span>
      </p>

      <ChromaticGauge centsOff={centsOff} />
      <div className="mode-badge-row">
        <span className={tuningMode === 'preset' ? 'mode-badge preset' : 'mode-badge chromatic'}>
          {tuningMode === 'preset' ? 'Preset mode' : 'Chromatic mode'}
        </span>
        {tuningMode === 'preset' ? (
          <span className="mode-badge-sub">{selectedPresetLabel ?? 'No preset'}</span>
        ) : null}
      </div>
      <NoteDisplay
        noteName={noteName}
        centsOff={centsOff}
        strings={strings}
        stringIndex={activeStringIndex}
        tuningMode={tuningMode}
      />
      <SignalInfo frequencyHz={frequencyHz} centsOff={centsOff} />
      <TuningPresetPicker
        presets={presets.catalog}
        familyFilter={presets.familyFilter}
        onFamilyFilterChange={presets.onFamilyFilterChange}
        tuningMode={tuningMode}
        onModeChange={presets.onModeChange}
        selectedPreset={presets.selectedPreset}
        onSelect={presets.onSelectPreset}
      />

      <button
        className={running ? 'primary primary-running' : 'primary'}
        type="button"
        disabled={tuningMode === 'preset' && !presets.selectedPreset}
        onClick={onToggleRunning}
      >
        {running ? 'Stop microphone' : 'Start microphone'}
      </button>

      <SpectralGraph
        frequencyHz={frequencyHz}
        confidence={confidence}
        clarity={clarity}
      />

      {error ? <p className="error">Error: {error}</p> : null}
    </section>
  )
}
