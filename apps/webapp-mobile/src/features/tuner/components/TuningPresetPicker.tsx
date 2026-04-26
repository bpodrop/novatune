import { useMemo, useState } from 'react'
import type { PresetFamilyFilter, TuningPresetProfile } from '../../../tuner'
import { filterPresetsByFamily } from '../../../tuner/presets'
import type { TuningMode } from '../useTunerController'

function presetById(presets: readonly TuningPresetProfile[], presetId: string | null) {
  if (!presetId) {
    return null
  }

  return presets.find((preset) => preset.id === presetId) ?? null
}

interface TuningPresetPickerProps {
  presets: readonly TuningPresetProfile[]
  familyFilter: PresetFamilyFilter
  onFamilyFilterChange: (value: PresetFamilyFilter) => void
  tuningMode: TuningMode
  onModeChange: (mode: TuningMode) => void
  selectedPreset: string | null
  onSelect: (preset: string) => void
}

export function TuningPresetPicker({
  presets,
  familyFilter,
  onFamilyFilterChange,
  tuningMode,
  onModeChange,
  selectedPreset,
  onSelect,
}: TuningPresetPickerProps) {
  const [open, setOpen] = useState(false)
  const filteredPresets = useMemo(
    () => filterPresetsByFamily(presets, familyFilter),
    [familyFilter, presets],
  )
  const selectedPresetMeta = presetById(presets, selectedPreset)
  const activeLabel =
    tuningMode === 'chromatic'
      ? 'CHROMATIC'
      : selectedPresetMeta?.label ?? 'SELECT PRESET'

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
            <div className="preset-family-filter" role="group" aria-label="String family">
              {(['all', 6, 7, 8, 9] as const).map((value) => (
                <button
                  key={String(value)}
                  type="button"
                  className={
                    familyFilter === value ? 'family-filter-button active' : 'family-filter-button'
                  }
                  onClick={() => onFamilyFilterChange(value)}
                >
                  {value === 'all' ? 'All' : `${value} strings`}
                </button>
              ))}
            </div>
            <div className="preset-sheet-list">
              {filteredPresets.map((preset) => (
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
                  {`${preset.label} (${preset.stringCount})`}
                </button>
              ))}
              {filteredPresets.length === 0 ? (
                <p className="preset-empty-state">No preset available for this family.</p>
              ) : null}
            </div>
          </section>
        </div>
      ) : null}
    </section>
  )
}
