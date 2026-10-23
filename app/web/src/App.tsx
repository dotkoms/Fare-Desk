import { useEffect, useMemo, useState } from 'react'
import DataBadge from './components/DataBadge'
import Results from './components/Results'
import SearchPanel from './components/SearchPanel'
import Stage from './components/Stage'
import { apiEnabled, localProvenance, searchFlights, type Provenance } from './lib/api'
import { itinerariesByRoute, routeKey, snapshots } from './data/mock'
import type { Itinerary, Query, Sort } from './types'

const initialQuery: Query = {
  tripType: 'return',
  origin: 'BER',
  destination: 'LIS',
  depart: '2027-03-01',
  returnDate: '2027-03-08',
  passengers: 1,
  bags: 1,
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
  const [provenance, setProvenance] = useState<Provenance | null>(null)
  const [loading, setLoading] = useState(apiEnabled)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [refreshKey, setRefreshKey] = useState(0)

  const route = routeKey(activeQuery.origin, activeQuery.destination)

  // Without a data plane, the bundled fixtures are the answer and no request
  // is attempted; that keeps the demo working when the API is not running.
  const offlineResults = useMemo<Itinerary[]>(
    () => (apiEnabled ? [] : (itinerariesByRoute[route] ?? [])),
    [route],
  )
  const offlineProvenance = useMemo(
    () => (apiEnabled ? null : localProvenance(offlineResults.length)),
    [offlineResults.length],
  )
  const visibleResults = apiEnabled ? results : offlineResults
  const visibleProvenance = apiEnabled ? provenance : offlineProvenance
  const visibleLoading = apiEnabled && loading

  const sorted = useMemo(() => sortItineraries(visibleResults, sort), [visibleResults, sort])
  const selected = sorted.find((itinerary) => itinerary.id === selectedId) ?? sorted[0] ?? null
  const snapshot = snapshots[route] ?? null
  const routeLabel = `${activeQuery.origin} → ${activeQuery.destination}`

  useEffect(() => {
    if (!apiEnabled) return

    let cancelled = false

    async function load() {
      setError(null)
      setNotice(null)
      setLoading(true)
      setResults([])
      setProvenance(null)
      try {
        const outcome = await searchFlights(activeQuery)
        if (cancelled) return
        setResults(outcome.itineraries)
        setProvenance(outcome.provenance)
        setNotice(outcome.notice ?? outcome.provenance.degradedReason ?? null)
      } catch (cause) {
        if (cancelled) return
        setResults([])
        setProvenance(null)
        setError(cause instanceof Error ? cause.message : 'Could not load flights.')
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    void load()

    return () => {
      cancelled = true
    }
  }, [activeQuery, refreshKey])

  return (
    <div className="app">
      <aside className="console">
        <header className="console-head">
          <div>
            <div className="wordmark">
              Fare<span>Desk</span>
            </div>
          </div>
          <DataBadge provenance={visibleProvenance} loading={visibleLoading} error={error} />
        </header>

        <SearchPanel
          query={query}
          onChange={(patch) => setQuery((current) => ({ ...current, ...patch }))}
          onSearch={() => {
            setActiveQuery(query)
            setSelectedId(null)
            setTab('itinerary')
          }}
        />

        <Results
          itineraries={sorted}
          selectedId={selected?.id ?? null}
          onSelect={(id) => {
            setSelectedId(id)
            setTab('itinerary')
          }}
          onIntel={(id) => {
            setSelectedId(id)
            setTab('intel')
          }}
          sort={sort}
          onSort={setSort}
          loading={visibleLoading}
          error={error}
          notice={notice}
          onRetry={() => setRefreshKey((key) => key + 1)}
        />
      </aside>

      <main className="workspace">
        <Stage
          itinerary={selected}
          snapshot={snapshot}
          tab={tab}
          onTab={setTab}
          routeLabel={routeLabel}
          dateLabel={formatDateRange(activeQuery)}
          passengers={activeQuery.passengers}
          loading={visibleLoading}
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
