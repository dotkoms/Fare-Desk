import type { Itinerary, Sort } from '../types'
import { formatDuration } from '../lib/format'

type Props = {
  itineraries: Itinerary[]
  selectedId: string | null
  onSelect: (id: string) => void
  onIntel: (id: string) => void
  sort: Sort
  onSort: (sort: Sort) => void
}

function stopLabel(stops: number) {
  if (stops === 0) return 'Nonstop'
  return `${stops} stop${stops > 1 ? 's' : ''}`
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
          <span className="result-bag">
            {itinerary.bagsIncluded > 0
              ? `${itinerary.bagsIncluded} checked bag${itinerary.bagsIncluded > 1 ? 's' : ''}`
              : 'Carry-on only'}
          </span>
        </div>

        <div className="result-tags">
          <span className="tag-verified">Verified {itinerary.verifiedMinutesAgo} min ago</span>
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

export default function Results({ itineraries, selectedId, onSelect, onIntel, sort, onSort }: Props) {
  return (
    <section className="results" aria-label="Flight results">
      <div className="results-head">
        <div>
          <span className="section-kicker">Results</span>
          <strong>
            {itineraries.length} flight{itineraries.length === 1 ? '' : 's'}
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

      <div className="result-list">
        {itineraries.length === 0 ? (
          <div className="empty-state">
            <h3>No fixture route for this pair.</h3>
            <p>Try BER → LIS, BER → LHR, BER → BCN or BER → JFK.</p>
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
