interface SignalInfoProps {
  frequencyHz: number | null
  centsOff: number | null
}

export function SignalInfo({ frequencyHz, centsOff }: SignalInfoProps) {
  return (
    <section className="signal-summary">
      <article className="signal-metric signal-metric-frequency">
        <p className="signal-label">Frequency</p>
        <p className="signal-value">{frequencyHz === null ? '--' : `${frequencyHz.toFixed(2)} Hz`}</p>
      </article>
      <article className="signal-metric signal-metric-tolerance">
        <p className="signal-label">Tolerance</p>
        <p className="signal-value">{centsOff === null ? '±-- ct' : `±${Math.abs(centsOff).toFixed(2)} ct`}</p>
      </article>
    </section>
  )
}
