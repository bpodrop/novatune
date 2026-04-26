import { useMemo } from 'react'
import { clamp } from './math'

interface SpectralGraphProps {
  frequencyHz: number | null
  confidence: number | null
  clarity: number | null
}

export function SpectralGraph({ frequencyHz, confidence, clarity }: SpectralGraphProps) {
  const bars = useMemo(() => {
    const safeConfidence = confidence ?? 0
    const safeClarity = clarity ?? 0
    const center = frequencyHz === null ? 11 : clamp(((frequencyHz % 120) / 120) * 23, 0, 23)
    const phase = frequencyHz === null ? 0 : frequencyHz / 13

    return Array.from({ length: 24 }, (_, index) => {
      const spread = Math.exp(-Math.pow(index - center, 2) / 18)
      const wobble = (Math.sin(index * 0.9 + phase) + 1) * 0.5
      const height = 0.15 + spread * (0.7 + safeConfidence * 0.5) + wobble * safeClarity * 0.25
      return clamp(height, 0.08, 1)
    })
  }, [clarity, confidence, frequencyHz])

  return (
    <section className="spectral-graph">
      <header className="spectral-title">
        <span className="live-dot" />
        LIVE INPUT • SPECTRAL ANALYSIS
      </header>
      <div className="spectral-bars" aria-hidden="true">
        {bars.map((height, index) => (
          <span key={index} className="spectral-bar" style={{ height: `${Math.round(height * 100)}%` }} />
        ))}
      </div>
    </section>
  )
}
