import React from 'react'
import { describe, expect, it } from 'vitest'
import { renderToStaticMarkup } from 'react-dom/server'
import { AppFooter } from '../AppFooter'
import { AppHeader } from '../AppHeader'

describe('AppHeader', () => {
  it('renders the brand and settings control', () => {
    const markup = renderToStaticMarkup(React.createElement(AppHeader, { onOpenSettings: () => {} }))

    expect(markup).toContain('NovaTuner')
    expect(markup).toContain('aria-label="Settings"')
    expect(markup).toContain('gear-icon')
  })

  it('renders install control when prompt is available', () => {
    const markup = renderToStaticMarkup(
      React.createElement(AppHeader, {
        onOpenSettings: () => {},
        installPromptAvailable: true,
        onInstallApp: async () => {},
      }),
    )

    expect(markup).toContain('Install')
    expect(markup).toContain('aria-label="Install app"')
  })
})

describe('AppFooter', () => {
  it('renders version and repository link', () => {
    const markup = renderToStaticMarkup(React.createElement(AppFooter))

    expect(markup).toContain('V0.1.0')
    expect(markup).toContain('https://github.com/bpodrop/novatune')
    expect(markup).toContain('NovaTuner GitHub repository')
  })
})
