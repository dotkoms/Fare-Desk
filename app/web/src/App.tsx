import { useEffect, useMemo, useState } from 'react'
import DataBadge from './components/DataBadge'
import Results from './components/Results'
import SearchPanel from './components/SearchPanel'
import Stage from './components/Stage'
import { searchFlights, type Provenance } from './lib/api'
import type { Itinerary, Query, Sort } from './types'

function isoDaysFromNow(days: number) {
  const date = new Date()
  date.setDate(date.getDate() + days)
  return date.toISOString().slice(0, 10)
}

// Opens on the New York corridor the collector watches, a month out.
const initialQuery: Query = {
  tripType: 'return',
  origin: 'JFK',
  destination: 'LAX',
  depart: isoDaysFromNow(30),
  returnDate: isoDaysFromNow(37),
  passengers: 1,
  bags: 0,
  currency: 'USD',
}

function bestScore(total: number, durationMinutes: number) {
  return total + durationMinutes * 0.6
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short', year: 'numeric' }).format(
    new Date(`${value}T00:00:00`),
  )
}

function formatDateRange(query: Query) {
  if (query.tripType === 'oneway') return formatDate(query.depart)
  const depart = new Date(`${query.depart}T00:00:00`)
  const returnDate = new Date(`${query.returnDate}T00:00:00`)
  const sameMonth = depart.getMonth() === returnDate.getMonth() && depart.getFullYear() === returnDate.getFullYear()

  if (sameMonth) {
    const month = new Intl.DateTimeFormat('en-GB', { month: 'short', year: 'numeric' }).format(returnDate)
    return `${depart.getDate()}–${returnDate.getDate()} ${month}`
  }

  return `${formatDate(query.depart)} – ${formatDate(query.returnDate)}`
}

function sortItineraries<T extends { total: number; durationMinutes: number }>(items: T[], sort: Sort) {
  const copy = [...items]
  if (sort === 'cheapest') return copy.sort((a, b) => a.total - b.total)
  if (sort === 'fastest') return copy.sort((a, b) => a.durationMinutes - b.durationMinutes)
  return copy.sort((a, b) => bestScore(a.total, a.durationMinutes) - bestScore(b.total, b.durationMinutes))
}

export default function App() {
  const [query, setQuery] = useState<Query>(initialQuery)
  const [activeQuery, setActiveQuery] = useState<Query>(initialQuery)
  const [sort, setSort] = useState<Sort>('best')
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [tab, setTab] = useState<'itinerary' | 'intel'>(() =>
    new URLSearchParams(window.location.search).get('tab') === 'intel' ? 'intel' : 'itinerary',
  )
  const [results, setResults] = useState<Itinerary[]>([])
  const [returns, setReturns] = useState<Itinerary[]>([])
  const [selectedReturnId, setSelectedReturnId] = useState<string | null>(null)
  const [direction, setDirection] = useState<'out' | 'back'>('out')
  const [provenance, setProvenance] = useState<Provenance | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)
  const [refreshKey, setRefreshKey] = useState(0)

  const sorted = useMemo(() => sortItineraries(results, sort), [results, sort])
  const selected = sorted.find((itinerary) => itinerary.id === selectedId) ?? sorted[0] ?? null
  const sortedReturns = useMemo(() => sortItineraries(returns, sort), [returns, sort])
  const selectedReturn =
    sortedReturns.find((itinerary) => itinerary.id === selectedReturnId) ?? sortedReturns[0] ?? null
  const showingReturns = direction === 'back' && sortedReturns.length > 0
  const routeLabel = `${activeQuery.origin} → ${activeQuery.destination}`
  const notice = provenance?.degraded ? (provenance.degradedReason ?? null) : null

  useEffect(() => {
    let cancelled = false

    async function load() {
      setError(null)
      setLoading(true)
      try {
        const outcome = await searchFlights(activeQuery)
        if (cancelled) return
        setResults(outcome.itineraries)
        setReturns(outcome.returns)
        setProvenance(outcome.provenance)
      } catch (cause) {
        if (cancelled) return
        setResults([])
        setReturns([])
        setProvenance(null)
        setError(cause instanceof Error ? cause : new Error('Could not load flights.'))
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    void load()

    return () => {
      cancelled = true
    }
  }, [activeQuery, refreshKey])

  const retry = { label: 'Try again', run: () => setRefreshKey((key) => key + 1) }

  return (
    <div className="app">
      <aside className="console">
        <header className="console-head">
          <div>
            <div className="wordmark">
              Fare<span>Desk</span>
            </div>
          </div>
          <DataBadge provenance={provenance} loading={loading} error={error?.message ?? null} />
        </header>

        <SearchPanel
          query={query}
          onChange={(patch) => setQuery((current) => ({ ...current, ...patch }))}
          onSearch={() => {
            setActiveQuery(query)
            setSelectedId(null)
            setSelectedReturnId(null)
            setDirection('out')
            setTab('itinerary')
          }}
        />

        <Results
          itineraries={showingReturns ? sortedReturns : sorted}
          selectedId={(showingReturns ? selectedReturn : selected)?.id ?? null}
          onSelect={(id) => {
            if (showingReturns) setSelectedReturnId(id)
            else setSelectedId(id)
            setTab('itinerary')
          }}
          onIntel={(id) => {
            if (showingReturns) setSelectedReturnId(id)
            else setSelectedId(id)
            setTab('intel')
          }}
          sort={sort}
          onSort={setSort}
          loading={loading}
          error={error?.message ?? null}
          errorAction={retry}
          notice={notice}
          trip={
            sortedReturns.length > 0 && selected && selectedReturn
              ? {
                  direction,
                  onDirection: setDirection,
                  outbound: `${activeQuery.origin} → ${activeQuery.destination}`,
                  back: `${activeQuery.destination} → ${activeQuery.origin}`,
                  total: selected.total + selectedReturn.total,
                  currency: selected.currency,
                }
              : null
          }
        />
      </aside>

      <main className="workspace">
        <Stage
          itinerary={selected}
          returnItinerary={selectedReturn}
          tab={tab}
          onTab={setTab}
          routeLabel={routeLabel}
          dateLabel={formatDateRange(activeQuery)}
          passengers={activeQuery.passengers}
          loading={loading}
          travelDate={activeQuery.depart}
          itineraries={sorted}
          selectedId={selected?.id ?? null}
          onSelect={setSelectedId}
          originCode={activeQuery.origin}
          destinationCode={activeQuery.destination}
        />
      </main>
    </div>
  )
}
