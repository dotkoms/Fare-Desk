import { useEffect, useState } from 'react'
import { getRouteIntel, type RouteIntel, type Verdict } from '../lib/api'
import { formatMoney } from '../lib/format'
import Sparkline from './Sparkline'

type Props = {
  origin: string
  destination: string
  travelDate: string
  /** The selected flight's total, shown next to the route's own numbers. */
  currentPrice: number
  /** The selected flight's currency; route history uses the currency it was recorded in. */
  currency: string
}

const VERDICT: Record<Verdict, { label: string; className: string }> = {
  buy: { label: 'Book now', className: 'recommendation-book-now' },
  wait: { label: 'Wait', className: 'recommendation-wait' },
  watch: { label: 'Watch', className: 'recommendation-watch' },
  insufficient: { label: 'Not enough data', className: 'recommendation-not-enough-data' },
}

function ordinal(value: number) {
  const remainder = value % 100
  if (remainder >= 11 && remainder <= 13) return `${value}th`
  switch (value % 10) {
    case 1:
      return `${value}st`
    case 2:
      return `${value}nd`
    case 3:
      return `${value}rd`
    default:
      return `${value}th`
  }
}

type Result = { key: string } & ({ status: 'error'; message: string } | { status: 'ready'; intel: RouteIntel })

export default function Intel({ origin, destination, travelDate, currentPrice, currency }: Props) {
  const key = `${origin}-${destination}-${travelDate}`
  const [result, setResult] = useState<Result | null>(null)

  useEffect(() => {
    const controller = new AbortController()
    getRouteIntel(origin, destination, travelDate, controller.signal)
      .then(({ intel }) => setResult({ key, status: 'ready', intel }))
      .catch((cause: unknown) => {
        if (controller.signal.aborted) return
        const message = cause instanceof Error ? cause.message : 'Could not load route history.'
        setResult({ key, status: 'error', message })
      })
    return () => controller.abort()
  }, [key, origin, destination, travelDate])

  // A result for another route is stale: show loading until this route's answer arrives.
  const state = result?.key === key ? result : null

  if (!state) {
    return (
      <div className="intel" aria-busy="true">
        <span className="section-kicker">When to buy</span>
        <p className="intel-copy">Reading this route’s price history…</p>
      </div>
    )
  }

  if (state.status === 'error') {
    return (
      <div className="intel intel-empty">
        <span className="section-kicker">When to buy</span>
        <h3>Route history is unavailable.</h3>
        <p>{state.message}</p>
      </div>
    )
  }

  const { intel } = state
  const money = (value: number) => formatMoney(value, intel.currency)
  const verdict = intel.recommendation ?? 'insufficient'
  const enough = verdict !== 'insufficient' && intel.p10 !== null && intel.p90 !== null && intel.currentPrice !== null

  if (!enough) {
    return (
      <div className="intel intel-empty">
        <div className="intel-head">
          <span className="section-kicker">When to buy</span>
          <span className={`recommendation ${VERDICT.insufficient.className}`}>{VERDICT.insufficient.label}</span>
        </div>
        <h3>There is not enough data for this route yet.</h3>
        <p>
          Collecting baseline — {intel.sampleSize} of {intel.minObservations} daily observations. A buy or wait call
          before then would be a guess.
        </p>
        {intel.history.length > 1 ? <Sparkline values={intel.history.map((point) => point.price)} /> : null}
      </div>
    )
  }

  const p10 = intel.p10 as number
  const p90 = intel.p90 as number
  const today = intel.currentPrice as number
  // The band sits at 12–88% of the track; place today's fare on the same scale.
  const position = Math.max(3, Math.min(97, 12 + ((today - p10) / (p90 - p10 || 1)) * 76))
  const { label, className } = VERDICT[verdict]

  return (
    <div className="intel">
      <div className="intel-head">
        <div>
          <span className="section-kicker">Lowest fare on this route today</span>
          <div className="intel-price">
            {money(today)}
            <span>/ adult</span>
          </div>
          <div className="intel-sub">
            {intel.percentile !== null ? `${ordinal(Math.round(intel.percentile))} percentile · ` : ''}
            {intel.sampleSize} days observed · this flight {formatMoney(currentPrice, currency)}
          </div>
        </div>
        <span className={`recommendation ${className}`}>{label}</span>
      </div>

      <div className="range-block">
        <div className="range-labels">
          <span>{money(p10)}</span>
          <span>Middle 80% of fares</span>
          <span>{money(p90)}</span>
        </div>
        <div className="range-track">
          <div className="range-band" />
          <div className="range-marker" style={{ left: `${position}%` }} />
        </div>
      </div>

      <Sparkline values={intel.history.map((point) => point.price)} />

      <div className="intel-evidence">
        <div>
          <span className="section-kicker">In 7 days</span>
          <strong>{intel.expected7 ? `${money(intel.expected7.low)}–${money(intel.expected7.high)}` : '—'}</strong>
        </div>
        <div>
          <span className="section-kicker">In 21 days</span>
          <strong>{intel.expected21 ? `${money(intel.expected21.low)}–${money(intel.expected21.high)}` : '—'}</strong>
        </div>
        <div>
          <span className="section-kicker">Track record</span>
          <strong>
            {intel.trackRecord ? `${intel.trackRecord.correct} of ${intel.trackRecord.cases}` : 'None yet'}
          </strong>
        </div>
      </div>

      {intel.reasons?.map((reason) => (
        <p className="intel-copy" key={reason}>
          {reason}
        </p>
      ))}
      {intel.downside !== undefined ? (
        <p className="intel-downside">
          {Math.round(intel.downside)}% of past weeks would have taken this fare below {money(p10)}.
        </p>
      ) : null}
      {intel.changeMyMind ? <p className="intel-downside">{intel.changeMyMind}</p> : null}
    </div>
  )
}
