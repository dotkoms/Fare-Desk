import type { Itinerary } from '../types'

function toMinutes(time: string) {
  const [hours, minutes] = time.split(':').map(Number)
  return hours * 60 + minutes
}

function layover(previousArrive: string, nextDepart: string) {
  const gap = toMinutes(nextDepart) - toMinutes(previousArrive)
  return (gap + 24 * 60) % (24 * 60)
}

function formatMinutes(minutes: number) {
  const hours = Math.floor(minutes / 60)
  const rest = minutes % 60
  return `${hours}h ${String(rest).padStart(2, '0')}`
}

export default function RouteBoard({ itinerary }: { itinerary: Itinerary }) {
  return (
    <div className="route-board" aria-label="Route and connections">
      {itinerary.segments.map((segment, index) => (
        <div className="segment" key={`${segment.flight}-${index}`}>
          <div className="segment-airport">
            <strong>{segment.from.code}</strong>
            <span>{segment.depart}</span>
          </div>
          <div className="segment-line">
            <span className="segment-dot" />
            <span className="segment-track" />
            <span className="segment-duration">{formatMinutes(segment.durationMinutes)}</span>
            <span className="segment-track" />
            <span className="segment-dot" />
          </div>
          <div className="segment-airport segment-airport-end">
            <strong>{segment.to.code}</strong>
            <span>{segment.arrive}</span>
          </div>
          <div className="segment-flight">
            {segment.carrier} {segment.flight}
          </div>

          {index < itinerary.segments.length - 1 && (
            <div className="layover">
              Layover {formatMinutes(layover(segment.arrive, itinerary.segments[index + 1].depart))} in{' '}
              {segment.to.city}
            </div>
          )}
        </div>
      ))}
    </div>
  )
}
