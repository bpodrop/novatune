import { useEffect, useState } from 'react'

export type ThemeMode = 'dark' | 'light'
export interface ThemeControls {
  theme: ThemeMode
  setTheme: (value: ThemeMode) => void
}

export function useThemeMode(): ThemeControls {
  const [theme, setTheme] = useState<ThemeMode>(() => {
    if (typeof window === 'undefined') {
      return 'dark'
    }
    const stored = window.localStorage.getItem('novatuner.theme')
    return stored === 'light' ? 'light' : 'dark'
  })

  useEffect(() => {
    if (typeof window === 'undefined') {
      return
    }
    window.localStorage.setItem('novatuner.theme', theme)
  }, [theme])

  return { theme, setTheme }
}
