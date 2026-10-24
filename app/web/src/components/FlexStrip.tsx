import { useEffect, useState } from 'react'
import { getFlexCalendar, type FlexDay } from '../lib/api'
import { formatMoney } from '../lib/format'

type Props = {
  origin: string
  destination: string
  date: string
  /** The calendar is quoted in USD; other currencies get no strip rather than a wrong one. */
  currency: string
  onPick: (date: string) => void
}

const WINDOW = 3

function shift(date: string, days: number) {
  const value = new Date(`${date}T00:00:00Z`)
  value.setUTCDate(value.getUTCDate() + days)
  return value.toISOString().slice(0, 10)
}

const weekday = new Intl.DateTimeFormat('en', { weekday: 'short', timeZone: 'UTC' })
const dayOfMonth = new Intl.DateTimeFormat('en', { day: 'numeric', month: 'short', timeZone: 'UTC' })

/** Cheapest fare for the days either side of the chosen departure; tap a day to search it. */
export default function FlexStrip({ origin, destination, date, currency, onPick }: Props) {
  const key = `${origin}-${destination}-${date}`
  const [result, setResult] = useState<{ key: string; days: FlexDay[] } | null>(null)

  useEffect(() => {
    if (currency !== 'USD') return
    const controller = new AbortController()
    getFlexCalendar(origin, destination, date, controller.signal)
      .then((calendar) => setResult({ key, days: calendar.days }))
      .catch(() => {
        if (!controller.signal.aborted) setResult({ key, days: [] })
      })
    return () => controller.abort()
  }, [key, origin, destination, date, currency])

  if (currency !== 'USD' || result?.key !== key) return null
  const byDate = new Map(result.days.map((day) => [day.date, day.price]))
  const window = Array.from({ length: WINDOW * 2 + 1 }, (_, index) => shift(date, index - WINDOW)).filter((day) =>
    byDate.has(day),
  )
  if (window.length < 2) return null
  const cheapest = Math.min(...window.map((day) => byDate.get(day) as number))

  return (
    <div className="flex-strip">
      <div className="flex-strip-days" role="group" aria-label="Cheapest fare by departure day">
        {window.map((day) => {
          const price = byDate.get(day) as number
          return (
            <button
              type="button"
              key={day}
              className={`flex-day${day === date ? ' is-current' : ''}${price === cheapest ? ' is-cheapest' : ''}`}
              aria-pressed={day === date}
              onClick={() => onPick(day)}
            >
              <span>{weekday.format(new Date(`${day}T00:00:00Z`))}</span>
              <small>{dayOfMonth.format(new Date(`${day}T00:00:00Z`))}</small>
              <strong>{formatMoney(price, 'USD')}</strong>
            </button>
          )
        })}
      </div>
      <p className="flex-strip-note">Cheapest per day on Skiplagged · indicative</p>
    </div>
  )
}
