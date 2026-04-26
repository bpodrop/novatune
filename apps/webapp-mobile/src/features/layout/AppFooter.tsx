const APP_VERSION = 'V0.1.0'
const GITHUB_URL = 'https://github.com/bpodrop/novatune'

export function AppFooter() {
  return (
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
  )
}
