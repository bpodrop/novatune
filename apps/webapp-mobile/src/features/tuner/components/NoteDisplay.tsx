import type { TuningPresetString } from '../../../tuner'
import type { TuningMode } from '../useTunerController'

interface NoteDisplayProps {
  noteName: string
  centsOff: number | null
  strings: readonly TuningPresetString[]
  stringIndex: number | null
  tuningMode: TuningMode
}

export function NoteDisplay({
  noteName,
  centsOff,
  strings,
  stringIndex,
  tuningMode,
}: NoteDisplayProps) {
  const hasActiveString = stringIndex !== null && stringIndex >= 0 && stringIndex < strings.length
  const noMatch = centsOff !== null && !hasActiveString
  const matchHint =
    centsOff === null ? 'Waiting signal' : hasActiveString ? null : 'No matching string in this preset'

  return (
    <section className="note-display">
      <p className="note-display-main">{noteName}</p>
      <p className="note-display-offset">
        {centsOff === null ? '-- ct' : `${centsOff >= 0 ? '+' : ''}${centsOff.toFixed(1)} ct`}
      </p>
      {tuningMode === 'preset' ? (
        <div
          className={hasActiveString ? 'note-display-strings' : 'note-display-strings unmatched'}
          role="list"
          aria-label="Preset strings"
        >
          {strings.map((stringValue, index) => (
            <span
              key={`${stringValue.label}-${index}`}
              role="listitem"
              className={
                index === stringIndex ? 'note-display-string-chip active' : 'note-display-string-chip'
              }
            >
              {`${stringValue.displayNumber}:${stringValue.label}`}
            </span>
          ))}
        </div>
      ) : (
        <p className="note-display-string">Chromatic</p>
      )}
      {tuningMode === 'preset' && matchHint ? (
        <p className={noMatch ? 'note-display-match-hint warning' : 'note-display-match-hint'}>
          {matchHint}
        </p>
      ) : null}
    </section>
  )
}
