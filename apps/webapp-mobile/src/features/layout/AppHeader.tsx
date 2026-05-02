interface AppHeaderProps {
  onOpenSettings: () => void
  installPromptAvailable?: boolean
  onInstallApp?: () => void | Promise<void>
}

export function AppHeader({
  onOpenSettings,
  installPromptAvailable = false,
  onInstallApp,
}: AppHeaderProps) {
  return (
    <header className="top-app-bar">
      <span className="brand">NovaTuner</span>
      <div className="top-app-bar-actions">
        {installPromptAvailable ? (
          <button
            className="header-pill-button"
            type="button"
            aria-label="Install app"
            onClick={onInstallApp}
          >
            Install
          </button>
        ) : null}
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
      </div>
    </header>
  )
}
