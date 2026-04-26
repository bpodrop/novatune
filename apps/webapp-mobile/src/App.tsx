import { useState } from 'react'
import './App.css'
import { AppFooter } from './features/layout/AppFooter'
import { AppHeader } from './features/layout/AppHeader'
import { SettingsView } from './features/settings/SettingsView'
import { useThemeMode } from './features/settings/useThemeMode'
import { TunerPanel } from './features/tuner/TunerPanel'
import { useTunerController } from './features/tuner/useTunerController'

type ViewId = 'tuner' | 'settings'

function App() {
  const themeControls = useThemeMode()
  const [activeView, setActiveView] = useState<ViewId>('tuner')
  const tuner = useTunerController()
  const { presets, selection, settings, runtime } = tuner

  return (
    <main className="app-shell" data-theme={themeControls.theme}>
      <AppHeader
        onOpenSettings={() => setActiveView((view) => (view === 'settings' ? 'tuner' : 'settings'))}
      />

      <TunerPanel
        active={activeView === 'tuner'}
        status={runtime}
        display={{
          tuningMode: selection.tuningMode,
          selectedPresetLabel: selection.selectedPresetMeta?.label ?? null,
          noteName: runtime.detection?.noteName ?? '--',
          centsOff: runtime.detection?.centsOff ?? null,
          frequencyHz: runtime.detection?.frequencyHz ?? null,
          confidence: runtime.detection?.confidence ?? null,
          clarity: runtime.detection?.clarity ?? null,
          strings: selection.selectedPresetMeta?.strings ?? [],
          activeStringIndex: selection.activeStringIndex,
        }}
        presets={{
          catalog: presets,
          familyFilter: selection.presetFamilyFilter,
          selectedPreset: selection.selectedPreset,
          onFamilyFilterChange: selection.setPresetFamilyFilter,
          onModeChange: selection.setTuningMode,
          onSelectPreset: (preset) => selection.setSelectedPreset(preset),
        }}
        onToggleRunning={runtime.toggleRunning}
      />

      {activeView === 'settings' ? (
        <SettingsView
          onClose={() => setActiveView('tuner')}
          themeControls={themeControls}
          tunerSettings={settings}
        />
      ) : null}

      <AppFooter />
    </main>
  )
}

export default App
