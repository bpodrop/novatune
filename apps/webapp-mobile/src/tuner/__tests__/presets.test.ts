import { describe, expect, it } from 'vitest'

import {
  filterPresetsByFamily,
  resolveDefaultPresetId,
  resolvePresetIdForFamily,
  shouldShowPresetStrings,
} from '../presets'
import type { TuningPresetProfile } from '../types'

const PRESETS: TuningPresetProfile[] = [
  {
    id: 'drop-d',
    label: 'Drop D',
    stringCount: 6,
    strings: [],
  },
  {
    id: 'e-standard',
    label: 'E Standard',
    stringCount: 6,
    strings: [],
  },
  {
    id: 'drop-a-7',
    label: 'Drop A (7)',
    stringCount: 7,
    strings: [],
  },
  {
    id: 'fsharp-standard-8',
    label: 'F# Standard (8)',
    stringCount: 8,
    strings: [],
  },
]

describe('preset helpers', () => {
  it('filters by instrument family', () => {
    const onlySeven = filterPresetsByFamily(PRESETS, 7)
    expect(onlySeven.map((preset) => preset.id)).toEqual(['drop-a-7'])

    const all = filterPresetsByFamily(PRESETS, 'all')
    expect(all).toHaveLength(4)
  })

  it('resolves default preset with e-standard priority', () => {
    expect(resolveDefaultPresetId(PRESETS)).toBe('e-standard')
    expect(resolveDefaultPresetId([])).toBeNull()
  })

  it('falls back to first matching preset for the selected family', () => {
    expect(resolvePresetIdForFamily(PRESETS, 'drop-d', 8)).toBe('fsharp-standard-8')
    expect(resolvePresetIdForFamily(PRESETS, null, 7)).toBe('drop-a-7')
    expect(resolvePresetIdForFamily(PRESETS, 'drop-d', 'all')).toBe('drop-d')
  })

  it('shows preset strings only in preset mode', () => {
    expect(shouldShowPresetStrings('preset', 'preset')).toBe(true)
    expect(shouldShowPresetStrings('preset', 'chromatic')).toBe(false)
    expect(shouldShowPresetStrings('chromatic', 'preset')).toBe(false)
  })
})
