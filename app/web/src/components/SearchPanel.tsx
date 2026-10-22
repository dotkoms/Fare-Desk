import type { Query } from '../types'
import { airports } from '../data/mock'

type Props = {
  query: Query
  onChange: (patch: Partial<Query>) => void
  onSearch: () => void
}

function AirportSelect({
  label,
  value,
  onChange,
}: {
  label: string
  value: string
  onChange: (value: string) => void
}) {
  return (
    <label className="field">
      <span className="field-label">{label}</span>
      <select value={value} onChange={(event) => onChange(event.target.value)}>
        {airports.map((airport) => (
          <option key={airport.code} value={airport.code}>
            {airport.code} · {airport.city}
          </option>
        ))}
      </select>
    </label>
  )
}

export default function SearchPanel({ query, onChange, onSearch }: Props) {
  return (
    <form
      className="search-panel"
      onSubmit={(event) => {
        event.preventDefault()
        onSearch()
      }}
    >
      <div className="trip-type" role="group" aria-label="Trip type">
        <button
          type="button"
          className={query.tripType === 'return' ? 'is-active' : ''}
          aria-pressed={query.tripType === 'return'}
          onClick={() => onChange({ tripType: 'return' })}
        >
          Return
        </button>
        <button
          type="button"
          className={query.tripType === 'oneway' ? 'is-active' : ''}
          aria-pressed={query.tripType === 'oneway'}
          onClick={() => onChange({ tripType: 'oneway' })}
        >
          One way
        </button>
      </div>

      <div className="places">
        <AirportSelect label="From" value={query.origin} onChange={(origin) => onChange({ origin })} />
        <button
          type="button"
          className="swap"
          aria-label="Swap origin and destination"
          onClick={() => onChange({ origin: query.destination, destination: query.origin })}
        >
          ⇄
        </button>
        <AirportSelect
          label="To"
          value={query.destination}
          onChange={(destination) => onChange({ destination })}
        />
      </div>

      <div className="field-row">
        <label className="field">
          <span className="field-label">Depart</span>
          <input
            type="date"
            value={query.depart}
            onChange={(event) => onChange({ depart: event.target.value })}
          />
        </label>
        <label className={`field ${query.tripType === 'oneway' ? 'is-disabled' : ''}`}>
          <span className="field-label">Return</span>
          <input
            type="date"
            value={query.returnDate}
            disabled={query.tripType === 'oneway'}
            onChange={(event) => onChange({ returnDate: event.target.value })}
          />
        </label>
      </div>

      <div className="field-row">
        <label className="field">
          <span className="field-label">Travellers</span>
          <input
            type="number"
            min={1}
            max={6}
            value={query.passengers}
            onChange={(event) => onChange({ passengers: Number(event.target.value) })}
          />
        </label>
        <label className="field">
          <span className="field-label">Checked bags</span>
          <input
            type="number"
            min={0}
            max={3}
            value={query.bags}
            onChange={(event) => onChange({ bags: Number(event.target.value) })}
          />
        </label>
      </div>

      <button className="button-primary" type="submit">
        Search flights
      </button>

      <p className="search-note">
        Demo fixtures only. Live prices arrive with the SerpAPI adapter.
      </p>
    </form>
  )
}
