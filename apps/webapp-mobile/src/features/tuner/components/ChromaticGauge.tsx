import { clamp } from './math'

interface ChromaticGaugeProps {
  centsOff: number | null
}

export function ChromaticGauge({ centsOff }: ChromaticGaugeProps) {
  const gaugeMin = -10
  const gaugeMax = 10
  const gaugeSpan = gaugeMax - gaugeMin
  const hasSignal = centsOff !== null
  const cents = clamp(centsOff ?? 0, gaugeMin, gaugeMax)
  const absolute = Math.abs(cents)
  const inTune = hasSignal && absolute <= 3
  const spread = hasSignal ? (absolute / (gaugeSpan / 2)) * 44 : 0
  const leftMarkerPos = 50 - spread
  const rightMarkerPos = 50 + spread

  const stateClass = !hasSignal ? 'no-signal' : inTune ? 'in-tune' : cents < 0 ? 'too-low' : 'too-high'
  const guidanceArrow = !hasSignal ? '•' : inTune ? '✓' : cents < 0 ? '↑' : '↓'
  const guidanceLabel = !hasSignal
    ? 'Waiting signal'
    : inTune
      ? 'In tune'
      : cents < 0
        ? 'Too low · tighten'
        : 'Too high · loosen'

  return (
    <section className="chromatic-gauge" aria-label="Chromatic tuning gauge">
      <div className="gauge-markers">
        <span>-10</span>
        <span>-5</span>
        <span>0</span>
        <span>+5</span>
        <span>+10</span>
      </div>
      <div className="gauge-rail">
        <span className="target-line" />
        <span
          className={`gauge-indicator gauge-indicator-left ${stateClass}`}
          style={{ left: `${leftMarkerPos}%` }}
        />
        <span
          className={`gauge-indicator gauge-indicator-right ${stateClass}`}
          style={{ left: `${rightMarkerPos}%` }}
        />
      </div>
      <div className={`gauge-guidance ${stateClass}`} aria-live="polite">
        <span className="gauge-guidance-arrow" aria-hidden="true">
          {guidanceArrow}
        </span>
        <span>{guidanceLabel}</span>
      </div>
    </section>
  )
}
