import { useEffect, useMemo, useState } from 'react'

interface BeforeInstallPromptEvent extends Event {
  readonly platforms: string[]
  prompt: () => Promise<void>
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed'; platform: string }>
}

interface PwaInstallState {
  canInstall: boolean
  isStandalone: boolean
  installApp: () => Promise<void>
}

function isStandaloneDisplayMode(): boolean {
  if (typeof window === 'undefined') {
    return false
  }

  const mediaStandalone = window.matchMedia?.('(display-mode: standalone)').matches ?? false
  const navigatorStandalone =
    'standalone' in window.navigator &&
    typeof window.navigator.standalone === 'boolean' &&
    window.navigator.standalone

  return mediaStandalone || navigatorStandalone
}

export function usePwaInstall(): PwaInstallState {
  const [deferredPrompt, setDeferredPrompt] = useState<BeforeInstallPromptEvent | null>(null)
  const [isStandalone, setIsStandalone] = useState(() => isStandaloneDisplayMode())

  useEffect(() => {
    if (typeof window === 'undefined') {
      return undefined
    }

    const mediaQuery = window.matchMedia?.('(display-mode: standalone)')

    const updateStandalone = () => {
      setIsStandalone(isStandaloneDisplayMode())
    }

    const onBeforeInstallPrompt = (event: Event) => {
      event.preventDefault()
      setDeferredPrompt(event as BeforeInstallPromptEvent)
      updateStandalone()
    }

    const onAppInstalled = () => {
      setDeferredPrompt(null)
      updateStandalone()
    }

    window.addEventListener('beforeinstallprompt', onBeforeInstallPrompt)
    window.addEventListener('appinstalled', onAppInstalled)
    mediaQuery?.addEventListener?.('change', updateStandalone)
    updateStandalone()

    return () => {
      window.removeEventListener('beforeinstallprompt', onBeforeInstallPrompt)
      window.removeEventListener('appinstalled', onAppInstalled)
      mediaQuery?.removeEventListener?.('change', updateStandalone)
    }
  }, [])

  const canInstall = useMemo(
    () => deferredPrompt !== null && !isStandalone,
    [deferredPrompt, isStandalone],
  )

  async function installApp() {
    if (!deferredPrompt) {
      return
    }

    await deferredPrompt.prompt()
    const choice = await deferredPrompt.userChoice
    if (choice.outcome === 'accepted') {
      setDeferredPrompt(null)
      setIsStandalone(true)
      return
    }

    setDeferredPrompt(null)
    setIsStandalone(isStandaloneDisplayMode())
  }

  return {
    canInstall,
    isStandalone,
    installApp,
  }
}
