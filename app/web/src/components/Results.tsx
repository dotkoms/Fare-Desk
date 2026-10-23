import type { Itinerary, Sort } from '../types'
import { formatDuration } from '../lib/format'

type Props = {
  itineraries: Itinerary[]
  selectedId: string | null
  onSelect: (id: string) => void
  onIntel: (id: string) => void
  sort: Sort
  onSort: (sort: Sort) => void
  loading: boolean
  error: string | null
  notice: string | null
  errorAction: { label: string; run: () => void }
  /** Present for a round trip: which direction the list shows, and the pair's total. */
  trip: {
    direction: 'out' | 'back'
    onDirection: (direction: 'out' | 'back') => void
    outbound: string
    back: string
    total: number
  } | null
}

function stopLabel(stops: number) {
  if (stops === 0) return 'Nonstop'
  return `${stops} stop${stops > 1 ? 's' : ''}`
}

function bagLabel(itinerary: Itinerary) {
  const fee = itinerary.bagFee ?? 0
  if (itinerary.bagsIncluded > 0) {
    return `${itinerary.bagsIncluded} checked bag${itinerary.bagsIncluded > 1 ? 's' : ''} included`
  }
  if (fee > 0) return `Bag added · €${fee}`
  return 'Carry-on only'
}

function ResultCard({
  itinerary,
  selected,
  onSelect,
  onIntel,
}: {
  itinerary: Itinerary
  selected: boolean
  onSelect: () => void
  onIntel: () => void
}) {
  const first = itinerary.segments[0]
  const last = itinerary.segments[itinerary.segments.length - 1]

  return (
    <article className={`result ${selected ? 'is-selected' : ''}`}>
      <button className="result-main" type="button" aria-pressed={selected} onClick={onSelect}>
        <div className="result-top">
          <div>
            <span className="result-time">{first.depart}</span>
            <span className="result-arrow">→</span>
            <span className="result-time">{last.arrive}</span>
          </div>
          <span className="result-route">
            {first.from.code} → {last.to.code}
          </span>
          <span className="result-duration">{formatDuration(itinerary.durationMinutes)}</span>
        </div>

        <div className="result-meta">
          {first.carrier} · {stopLabel(itinerary.stops)}
          {itinerary.stops > 0 ? ` · via ${itinerary.segments[0].to.code}` : ''}
        </div>

        <div className="result-bottom">
          <span className="result-price">
            €{itinerary.total} <small>total</small>
          </span>
          <span className="result-bag">{bagLabel(itinerary)}</span>
        </div>

        <div className="result-tags">
          {itinerary.verifiedMinutesAgo === null ? (
            <span className="tag-demo">Not verified</span>
          ) : (
            <span className="tag-verified">Verified {itinerary.verifiedMinutesAgo} min ago</span>
          )}
          {itinerary.risk.map((risk) => (
            <span className="tag-risk" key={risk}>
              {risk}
            </span>
          ))}
        </div>
      </button>

      <button className="result-intel" type="button" onClick={onIntel}>
        When to buy
      </button>
    </article>
  )
}

export default function Results({
  itineraries,
  selectedId,
  onSelect,
  onIntel,
  sort,
  onSort,
  loading,
  error,
  notice,
  errorAction,
  trip,
}: Props) {
  return (
    <section className="results" aria-label="Flight results" aria-busy={loading}>
      <div className="results-head">
        <div>
          <span className="section-kicker">Results</span>
          <strong>
            {loading ? 'Searching…' : `${itineraries.length} flight${itineraries.length === 1 ? '' : 's'}`}
          </strong>
        </div>
        <label className="sort-field">
          <span className="sr-only">Sort results</span>
          <select value={sort} onChange={(event) => onSort(event.target.value as Sort)}>
            <option value="best">Best</option>
            <option value="cheapest">Cheapest</option>
            <option value="fastest">Fastest</option>
          </select>
        </label>
      </div>

      {trip ? (
        <div className="trip-legs">
          <div className="trip-type" role="group" aria-label="Flight direction">
            <button
              type="button"
              className={trip.direction === 'out' ? 'is-active' : ''}
              aria-pressed={trip.direction === 'out'}
              onClick={() => trip.onDirection('out')}
            >
              Outbound · {trip.outbound}
            </button>
            <button
              type="button"
              className={trip.direction === 'back' ? 'is-active' : ''}
              aria-pressed={trip.direction === 'back'}
              onClick={() => trip.onDirection('back')}
            >
              Return · {trip.back}
            </button>
          </div>
          <p className="trip-total">
            <strong>€{trip.total}</strong> trip total · two one-way tickets
          </p>
        </div>
      ) : null}

      {notice ? (
        <p className="results-notice" role="status">
          {notice}
        </p>
      ) : null}

      <div className="result-list">
        {loading && itineraries.length === 0 ? (
          <div className="result-loading" role="status">
            <span className="sr-only">Searching flights</span>
            <span className="result-skeleton" aria-hidden="true" />
            <span className="result-skeleton" aria-hidden="true" />
            <span className="result-skeleton" aria-hidden="true" />
          </div>
        ) : error ? (
          <div className="empty-state">
            <h3>Could not load flights.</h3>
            <p>{error}</p>
            <button className="button-secondary" type="button" onClick={errorAction.run}>
              {errorAction.label}
            </button>
          </div>
        ) : itineraries.length === 0 ? (
          <div className="empty-state">
            <h3>No flights found for these dates.</h3>
            <p>Try nearby dates, or another airport in the same city.</p>
          </div>
        ) : (
          itineraries.map((itinerary) => (
            <ResultCard
              key={itinerary.id}
              itinerary={itinerary}
              selected={itinerary.id === selectedId}
              onSelect={() => onSelect(itinerary.id)}
              onIntel={() => onIntel(itinerary.id)}
            />
          ))
        )}
      </div>
    </section>
  )
}
