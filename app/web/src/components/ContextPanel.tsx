import { useEffect, useState } from 'react'
import { getReliability, getDoorToDoor, getAirfareBaseline } from '../lib/api'
import { formatMoney } from '../lib/format'

type Props = {
  origin: string
  destination: string
  travelDate: string
  price: number
  currency: string
}

export default function ContextPanel({ origin, destination, travelDate, price, currency }: Props) {
  const [reliability, setReliability] = useState<string | null>(null)
  const [doorToDoor, setDoorToDoor] = useState<string | null>(null)
  const [baseline, setBaseline] = useState<string | null>(null)

  useEffect(() => {
    const controller = new AbortController()
    const signal = controller.signal

    getReliability(origin, destination, price, signal)
      .then((data) => {
        const top = data.carriers[0]
        if (top && top.onTimeRate !== null) {
          setReliability(
            `${top.carrier} ${(top.onTimeRate * 100).toFixed(0)}% on-time (${top.flights} flights)`,
          )
        } else {
          setReliability('No BTS reliability data yet')
        }
      })
      .catch(() => setReliability('Reliability unavailable'))

    getDoorToDoor(origin, destination, travelDate, signal)
      .then((data) => {
        setDoorToDoor(
          `Door-to-door ${formatMoney(data.total, currency)} (${Math.round(data.totalMinutes / 60)}h ${data.totalMinutes % 60}m)`,
        )
      })
      .catch(() => setDoorToDoor('Door-to-door unavailable'))

    getAirfareBaseline(origin, destination, signal)
      .then((data) => {
        if (data.averageFare !== null && data.year && data.quarter) {
          setBaseline(
            `DOT avg ${formatMoney(data.averageFare, 'USD')} (Q${data.quarter} ${data.year})`,
          )
        } else {
          setBaseline('No DOT baseline yet')
        }
      })
      .catch(() => setBaseline('Baseline unavailable'))

    return () => controller.abort()
  }, [origin, destination, travelDate, price, currency])

  return (
    <div className="intel" style={{ marginTop: '1rem' }}>
      <span className="section-kicker">Trip context</span>
      <p className="intel-copy">{reliability ?? 'Loading reliability…'}</p>
      <p className="intel-copy">{doorToDoor ?? 'Loading door-to-door…'}</p>
      <p className="intel-copy">{baseline ?? 'Loading baseline…'}</p>
    </div>
  )
}
