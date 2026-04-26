import React from 'react'
import { describe, expect, it } from 'vitest'
import { renderToStaticMarkup } from 'react-dom/server'
import { ChromaticGauge } from '../ChromaticGauge'
import { NoteDisplay } from '../NoteDisplay'
import { SignalInfo } from '../SignalInfo'
import { SpectralGraph } from '../SpectralGraph'

describe('ChromaticGauge', () => {
  it('shows waiting state when there is no signal', () => {
    const markup = renderToStaticMarkup(React.createElement(ChromaticGauge, { centsOff: null }))

    expect(markup).toContain('Waiting signal')
    expect(markup).toContain('gauge-guidance no-signal')
  })

  it('shows in-tune guidance for small offsets', () => {
    const markup = renderToStaticMarkup(React.createElement(ChromaticGauge, { centsOff: -2 }))

    expect(markup).toContain('In tune')
    expect(markup).toContain('gauge-guidance in-tune')
  })

  it('shows tightening guidance when the pitch is too low', () => {
    const markup = renderToStaticMarkup(React.createElement(ChromaticGauge, { centsOff: -8 }))

    expect(markup).toContain('Too low · tighten')
    expect(markup).toContain('gauge-guidance too-low')
  })

  it('shows loosening guidance when the pitch is too high', () => {
    const markup = renderToStaticMarkup(React.createElement(ChromaticGauge, { centsOff: 7 }))

    expect(markup).toContain('Too high · loosen')
    expect(markup).toContain('gauge-guidance too-high')
  })
})

describe('NoteDisplay', () => {
  const strings = [
    { index: 0, label: 'E', frequencyHz: 82.41, displayNumber: 6 },
    { index: 1, label: 'A', frequencyHz: 110, displayNumber: 5 },
  ] as const

  it('renders chromatic mode label', () => {
    const markup = renderToStaticMarkup(
      React.createElement(NoteDisplay, {
        noteName: 'A',
        centsOff: null,
        strings,
        stringIndex: null,
        tuningMode: 'chromatic',
      }),
    )

    expect(markup).toContain('Chromatic')
    expect(markup).toContain('A')
    expect(markup).toContain('-- ct')
  })

  it('highlights the active preset string', () => {
    const markup = renderToStaticMarkup(
      React.createElement(NoteDisplay, {
        noteName: 'A',
        centsOff: -0.4,
        strings,
        stringIndex: 1,
        tuningMode: 'preset',
      }),
    )

    expect(markup).toContain('6:E')
    expect(markup).toContain('5:A')
    expect(markup).toContain('note-display-string-chip active')
  })

  it('shows unmatched hint when no preset string matches', () => {
    const markup = renderToStaticMarkup(
      React.createElement(NoteDisplay, {
        noteName: 'C',
        centsOff: 14.2,
        strings,
        stringIndex: null,
        tuningMode: 'preset',
      }),
    )

    expect(markup).toContain('No matching string in this preset')
    expect(markup).toContain('note-display-match-hint warning')
  })
})

describe('SignalInfo', () => {
  it('formats numeric frequency and tolerance', () => {
    const markup = renderToStaticMarkup(
      React.createElement(SignalInfo, { frequencyHz: 110.1234, centsOff: -3.456 }),
    )

    expect(markup).toContain('110.12 Hz')
    expect(markup).toContain('±3.46 ct')
  })
})

describe('SpectralGraph', () => {
  it('renders the live input header and 24 bars', () => {
    const markup = renderToStaticMarkup(
      React.createElement(SpectralGraph, {
        frequencyHz: 110,
        confidence: 0.83,
        clarity: 0.74,
      }),
    )

    expect(markup).toContain('LIVE INPUT • SPECTRAL ANALYSIS')
    expect(markup.match(/class="spectral-bar"/g)).toHaveLength(24)
  })
})
