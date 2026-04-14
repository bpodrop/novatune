import type { PresetFamilyFilter, TuningPresetProfile } from './types'

const DEFAULT_PRESET_ID = 'e-standard'

export function filterPresetsByFamily(
  presets: readonly TuningPresetProfile[],
  family: PresetFamilyFilter,
): TuningPresetProfile[] {
  if (family === 'all') {
    return [...presets]
  }
  return presets.filter((preset) => preset.stringCount === family)
}

export function resolveDefaultPresetId(presets: readonly TuningPresetProfile[]): string | null {
  if (presets.length === 0) {
    return null
  }
  const eStandard = presets.find((preset) => preset.id === DEFAULT_PRESET_ID)
  return eStandard?.id ?? presets[0].id
}

export function resolvePresetIdForFamily(
  presets: readonly TuningPresetProfile[],
  currentPresetId: string | null,
  family: PresetFamilyFilter,
): string | null {
  const filtered = filterPresetsByFamily(presets, family)
  if (filtered.length === 0) {
    return resolveDefaultPresetId(presets)
  }
  if (currentPresetId && filtered.some((preset) => preset.id === currentPresetId)) {
    return currentPresetId
  }
  return filtered[0].id
}

export function shouldShowPresetStrings(
  tuningMode: 'preset' | 'chromatic',
  detectionMode: 'preset' | 'chromatic' | null,
): boolean {
  return tuningMode === 'preset' && detectionMode !== 'chromatic'
}
