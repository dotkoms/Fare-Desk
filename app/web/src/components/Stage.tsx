import { Suspense, lazy } from 'react'
import type { Itinerary } from '../types'
import GlobeBoundary from './GlobeBoundary'
import Intel from './Intel'
import RouteBoard from './RouteBoard'
import { formatDuration } from '../lib/format'

// three.js is heavy; keep it out of the shell bundle and load it alongside the stage
const GlobeStage = lazy(() => import('./GlobeStage'))
// Spike: the flat-shaded MapLibre globe; ?map=globe brings back the three.js globe for comparison.
const MapLibreStage = lazy(() => import('./MapLibreStage'))
const useLegacyGlobe = new URLSearchParams(window.location.search).get('map') === 'globe'
const MapStage = useLegacyGlobe ? GlobeStage : MapLibreStage

type Props = {
  itinerary: Itinerary | null
  tab: 'itinerary' | 'intel'
  onTab: (tab: 'itinerary' | 'intel') => void
  routeLabel: string
  dateLabel: string
  passengers: number
  itineraries: Itinerary[]
  selectedId: string | null
  onSelect: (id: string) => void
  originCode: string
  destinationCode: string
  loading: boolean
  travelDate: string
}

export default function Stage({
  itinerary,
  tab,
  onTab,
  routeLabel,
  dateLabel,
  passengers,
  itineraries,
  selectedId,
  onSelect,
  originCode,
  destinationCode,
  loading,
  travelDate,
}: Props) {
  if (!itinerary) {
    return (
      <section className="stage stage-empty" aria-label="Selected flight" aria-busy={loading}>
        <div className="stage-empty-inner">
          <span className="stage-kicker">{loading ? 'Searching' : 'No route selected'}</span>
          <h2>{loading ? 'Finding flights.' : 'Search a route to see the fare desk.'}</h2>
        </div>
      </section>
    )
  }

  return (
    <section className="stage" aria-label="Selected flight">
      <div className="stage-map">
        <GlobeBoundary>
          <Suspense fallback={<div className="globe-wrap globe-loading" aria-hidden="true" />}>
            <MapStage
              itineraries={itineraries}
              selectedId={selectedId}
              onSelect={onSelect}
              originCode={originCode}
              destinationCode={destinationCode}
            />
          </Suspense>
        </GlobeBoundary>

        <div className="stage-head">
          <span className="stage-kicker">{dateLabel}</span>
          <h2>{routeLabel}</h2>
        </div>

        <div className="stage-price">
          <span>€{itinerary.total}</span>
          <small>{passengers > 1 ? `total · ${passengers} travellers` : 'total'}</small>
        </div>

        <div className="route-overlay">
          <RouteBoard itinerary={itinerary} />
        </div>
      </div>

      <div className="stage-detail">
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
                <strong>
                  {itinerary.bagsIncluded > 0
                    ? `${itinerary.bagsIncluded} checked`
                    : (itinerary.bagFee ?? 0) > 0
                      ? `1 checked · €${itinerary.bagFee}`
                      : 'Carry-on only'}
                </strong>
              </div>
            </div>

            <div className="fare-rules">
              <span className={itinerary.refundable ? 'rule-ok' : 'rule-warn'}>
                {itinerary.refundable ? 'Refundable' : 'Non-refundable'}
              </span>
              <span>
                {itinerary.changeFee === 0 ? 'Free changes' : `Change €${itinerary.changeFee} + fare difference`}
              </span>
            </div>

            <div className="provider-row">
              <div>
                <span className="section-kicker">Provider</span>
                <strong>{itinerary.provider}</strong>
                <span className="provider-note">
                  {itinerary.verifiedMinutesAgo === null
                    ? 'Price not verified — confirm it on the provider'
                    : `Verified ${itinerary.verifiedMinutesAgo} min ago`}
                </span>
              </div>
              <a
                className="button-primary button-link"
                href={itinerary.providerUrl}
                target="_blank"
                rel="noopener noreferrer"
              >
                Open on {itinerary.provider}
              </a>
            </div>

            {itinerary.connectionWarnings.map((warning) => (
              <p className="stage-warning" key={warning}>
                {warning}
              </p>
            ))}

            <p className="stage-note">
              We don’t book or take payment. The final price is confirmed on {itinerary.provider}.
            </p>
          </div>
        ) : (
          <Intel
            origin={itinerary.segments[0].from.code}
            destination={itinerary.segments[itinerary.segments.length - 1].to.code}
            travelDate={travelDate}
            currentPrice={itinerary.total}
          />
        )}
      </div>
    </section>
  )
}
