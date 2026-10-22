import type { RouteSnapshot } from '../types'
import Sparkline from './Sparkline'

function clampPercent(value: number) {
  return Math.max(3, Math.min(97, value))
}

export default function Intel({
  snapshot,
  currentPrice,
}: {
  snapshot: RouteSnapshot | null
  currentPrice: number
}) {
  if (!snapshot) {
    return (
      <div className="intel intel-empty">
        <span className="section-kicker">When to buy</span>
        <h3>Not enough data for this route yet.</h3>
        <p>
          We have fewer than 14 daily observations. Current totals are visible, but a buy or wait call would be a
          guess.
        </p>
        <button className="button-secondary" type="button" disabled>
          Watch this route (demo)
        </button>
      </div>
    )
  }

  const position = clampPercent(
    ((currentPrice - snapshot.typicalLow) / (snapshot.typicalHigh - snapshot.typicalLow)) * 100,
  )
  const recommendation =
    currentPrice <= snapshot.typicalLow + (snapshot.typicalHigh - snapshot.typicalLow) * 0.3
      ? 'Book now'
      : currentPrice <= snapshot.typicalHigh
        ? 'Watch'
        : 'Wait'
  const percentile = Math.round(position)

  return (
    <div className="intel">
      <div className="intel-head">
        <div>
          <span className="section-kicker">Current total</span>
          <div className="intel-price">
            €{currentPrice}
            <span>/ person</span>
          </div>
          <div className="intel-sub">
            {percentile}th percentile · {snapshot.sample} observations
          </div>
        </div>
        <span className={`recommendation recommendation-${recommendation.toLowerCase().replaceAll(' ', '-')}`}>
          {recommendation}
        </span>
      </div>

      <div className="range-block">
        <div className="range-labels">
          <span>€{snapshot.typicalLow}</span>
          <span>Typical range</span>
          <span>€{snapshot.typicalHigh}</span>
        </div>
        <div className="range-track">
          <div className="range-band" />
          <div className="range-marker" style={{ left: `${position}%` }} />
        </div>
        <p className="range-caption">Middle 80% of fares paid for these dates.</p>
      </div>

      <Sparkline values={snapshot.history} />

      <div className="intel-evidence">
        <div>
          <span className="section-kicker">7 days</span>
          <strong>
            €{snapshot.expected7[0]}–€{snapshot.expected7[1]}
          </strong>
        </div>
        <div>
          <span className="section-kicker">21 days</span>
          <strong>
            €{snapshot.expected21[0]}–€{snapshot.expected21[1]}
          </strong>
        </div>
        <div>
          <span className="section-kicker">Track record</span>
          <strong>
            {snapshot.trackRecord ? `${snapshot.trackRecord.correct} of ${snapshot.trackRecord.cases}` : '—'}
          </strong>
        </div>
      </div>

      <p className="intel-copy">
        {recommendation === 'Book now'
          ? 'Below the typical range, and this route usually rises inside three weeks.'
          : recommendation === 'Wait'
            ? 'Above the typical range. Waiting is reasonable, but set a floor.'
            : 'Inside the typical range. Waiting and booking are both reasonable.'}
      </p>
      <p className="intel-downside">{snapshot.downside}.</p>

      <div className="intel-actions">
        <button className="button-secondary" type="button" disabled>
          Set alert at €{snapshot.typicalLow} (demo)
        </button>
        <span className="demo-note">Demo recommendation, not live advice.</span>
      </div>
    </div>
  )
}
