import type { Itinerary, RouteSnapshot } from '../types'
import Intel from './Intel'
import RouteBoard from './RouteBoard'
import { formatDuration } from '../lib/format'

type Props = {
  itinerary: Itinerary | null
  snapshot: RouteSnapshot | null
  tab: 'itinerary' | 'intel'
  onTab: (tab: 'itinerary' | 'intel') => void
  routeLabel: string
  dateLabel: string
  passengers: number
  bags: number
}

export default function Stage({ itinerary, snapshot, tab, onTab, routeLabel, dateLabel, passengers, bags }: Props) {
  if (!itinerary) {
    return (
      <section className="stage stage-empty" aria-label="Selected flight">
        <div className="stage-empty-inner">
          <span className="stage-kicker">No route selected</span>
          <h2>Search a fixture route to see the fare desk.</h2>
          <p>Try BER → LIS, BER → LHR, BER → BCN or BER → JFK.</p>
        </div>
      </section>
    )
  }

  return (
    <section className="stage" aria-label="Selected flight">
      <div className="stage-head">
        <div>
          <span className="stage-kicker">{dateLabel}</span>
          <h2>{routeLabel}</h2>
          <p className="stage-sub">
            {passengers} traveller{passengers === 1 ? '' : 's'} · {bags} checked bag{bags === 1 ? '' : 's'} · demo
            fixtures
          </p>
        </div>
        <div className="stage-price">
          <span>€{itinerary.total}</span>
          <small>total</small>
        </div>
      </div>

      <RouteBoard itinerary={itinerary} />

      <div className="stage-tabs" role="tablist" aria-label="Flight detail">
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'itinerary'}
          className={tab === 'itinerary' ? 'is-active' : ''}
          onClick={() => onTab('itinerary')}
        >
          Itinerary
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'intel'}
          className={tab === 'intel' ? 'is-active' : ''}
          onClick={() => onTab('intel')}
        >
          When to buy
        </button>
      </div>

      {tab === 'itinerary' ? (
        <div className="itinerary-panel">
          <div className="facts">
            <div>
              <span className="section-kicker">Duration</span>
              <strong>{formatDuration(itinerary.durationMinutes)}</strong>
            </div>
            <div>
              <span className="section-kicker">Stops</span>
              <strong>{itinerary.stops === 0 ? 'Nonstop' : itinerary.stops}</strong>
            </div>
            <div>
              <span className="section-kicker">Fare</span>
              <strong>{itinerary.fareFamily}</strong>
            </div>
            <div>
              <span className="section-kicker">Bags</span>
              <strong>{itinerary.bagsIncluded} checked</strong>
            </div>
          </div>

          <div className="fare-rules">
            <span className={itinerary.refundable ? 'rule-ok' : 'rule-warn'}>
              {itinerary.refundable ? 'Refundable' : 'Non-refundable'}
            </span>
            <span>{itinerary.changeFee === 0 ? 'Free changes' : `Change €${itinerary.changeFee} + fare difference`}</span>
          </div>

          <div className="provider-row">
            <div>
              <span className="section-kicker">Provider</span>
              <strong>{itinerary.provider}</strong>
              <span className="provider-note">Verified {itinerary.verifiedMinutesAgo} min ago</span>
            </div>
            <button className="button-primary" type="button" disabled title="Demo only — no booking">
              Open on {itinerary.provider} (demo)
            </button>
          </div>

          <p className="stage-note">
            Fare Desk does not book or take payment. This button would open the provider’s site in the live app.
          </p>
        </div>
      ) : (
        <Intel snapshot={snapshot} currentPrice={itinerary.total} />
      )}
    </section>
  )
}
