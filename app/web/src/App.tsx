import { useEffect, useMemo, useState } from 'react'
import Results from './components/Results'
import SearchPanel from './components/SearchPanel'
import Stage from './components/Stage'
import { itinerariesByRoute, routeKey, snapshots } from './data/mock'
import type { Query, Sort } from './types'

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

type Theme = 'system' | 'light' | 'dark'

function initialTheme(): Theme {
  if (typeof window === 'undefined') return 'system'
  const fromUrl = new URLSearchParams(window.location.search).get('theme')
  if (fromUrl === 'light' || fromUrl === 'dark') return fromUrl
  try {
    const stored = window.localStorage.getItem('faredesk-theme')
    if (stored === 'light' || stored === 'dark') return stored
  } catch {
    // Storage can be unavailable in private windows.
  }
  return 'system'
}

export default function App() {
  const [query, setQuery] = useState<Query>(initialQuery)
  const [activeQuery, setActiveQuery] = useState<Query>(initialQuery)
  const [sort, setSort] = useState<Sort>('best')
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [tab, setTab] = useState<'itinerary' | 'intel'>(() =>
    new URLSearchParams(window.location.search).get('tab') === 'intel' ? 'intel' : 'itinerary',
  )
  const [theme, setTheme] = useState<Theme>(initialTheme)

  const key = routeKey(activeQuery.origin, activeQuery.destination)
  const itineraries = useMemo(() => itinerariesByRoute[key] ?? [], [key])
  const sorted = useMemo(() => sortItineraries(itineraries, sort), [itineraries, sort])
  const selected = sorted.find((itinerary) => itinerary.id === selectedId) ?? sorted[0] ?? null
  const snapshot = snapshots[key] ?? null
  const routeLabel = `${activeQuery.origin} → ${activeQuery.destination}`

  useEffect(() => {
    const root = document.documentElement
    if (theme === 'system') root.removeAttribute('data-theme')
    else root.dataset.theme = theme
    try {
      window.localStorage.setItem('faredesk-theme', theme)
    } catch {
      // Ignore unavailable storage.
    }
  }, [theme])

  return (
    <div className="app">
      <aside className="console">
        <header className="console-head">
          <div>
            <div className="wordmark">
              Fare<span>Desk</span>
            </div>
            <div className="brand-tag">Independent flight intelligence</div>
          </div>
          <span className="demo-badge">Demo data</span>
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
        />

        <div className="console-footer">
          <button
            type="button"
            className="theme-toggle"
            onClick={() => setTheme(theme === 'system' ? 'light' : theme === 'light' ? 'dark' : 'system')}
          >
            Theme: {theme}
          </button>
        </div>
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
          bags={activeQuery.bags}
        />
      </main>
    </div>
  )
}
