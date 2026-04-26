interface SettingsViewProps {
  onClose: () => void
  themeControls: import('./useThemeMode').ThemeControls
  tunerSettings: import('../tuner/useTunerController').TunerSettingsControls
}

export function SettingsView({
  onClose,
  themeControls,
  tunerSettings,
}: SettingsViewProps) {
  const { theme, setTheme } = themeControls
  const {
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
  } = tunerSettings

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
        <input type="checkbox" checked={haptics} onChange={(event) => setHaptics(event.target.checked)} />
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
          onChange={(event) => setKeepAwake(event.target.checked)}
        />
      </label>
      <section className="tuning-control">
        <div className="tuning-control-header">
          <span>Preset Match Window</span>
          <strong>{presetMatchWindow} ct</strong>
        </div>
        <input
          className="calibration-slider"
          type="range"
          min={60}
          max={150}
          step={5}
          value={presetMatchWindow}
          onChange={(event) => setPresetMatchWindow(Number(event.target.value))}
        />
        <p className="tuning-control-copy">
          Smaller values are stricter; larger values match strings more easily.
        </p>
      </section>
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
