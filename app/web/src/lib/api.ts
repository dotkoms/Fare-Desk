import { itinerariesByRoute, routeKey } from '../data/mock'
import type { Airport, Itinerary, Query } from '../types'

/**
 * The Fare Desk data plane. In development the local service on :8080 is used
 * unless VITE_API_BASE says otherwise; in a production build an unset base
 * means "fixtures only" rather than a failing request on every search.
 */
const configuredBase = (import.meta.env.VITE_API_BASE as string | undefined)?.trim()
export const API_BASE = configuredBase
  ? configuredBase.replace(/\/+$/, '')
  : import.meta.env.DEV
    ? 'http://127.0.0.1:8080'
    : ''
export const apiEnabled = API_BASE !== ''

export type Freshness = 'bundled' | 'fixture' | 'live' | 'cached' | 'stale'

export type Provenance = {
  dataCardId: string
  source: string
  observedAt: string
  freshness: Freshness
  ageSeconds: number
  expiresAt?: string
  degraded: boolean
  degradedReason?: string
  sampleSize: number
  notes: string[]
}

export type SearchOutcome = {
  itineraries: Itinerary[]
  provenance: Provenance
  notice?: string
}

export class ApiRequestError extends Error {
  readonly status: number
  readonly code: string

  constructor(status: number, code: string, message: string) {
    super(message)
    this.name = 'ApiRequestError'
    this.status = status
    this.code = code
  }
}

type ApiPlace = {
  code: string
  city: string
  name: string
  countryName: string
}

type ApiSegment = {
  from: ApiPlace
  to: ApiPlace
  marketingCarrier: string
  serviceNumber: string
  departLocal: string
  arriveLocal: string
  durationMinutes: number
}

type ApiItinerary = {
  id: string
  segments: ApiSegment[]
  totalPrice: number
  currency: string
  totalDurationMinutes: number
  stops: number
  fareFamily?: string
  bagsIncluded: { carryOn: number; checked: number }
  fareRules: { refundable: boolean; changeFee?: number }
  priceBreakdown: { fare: number; bags: number; bagFeeEstimated: boolean }
  providerName: string
  verifiedAt: string | null
  risk: string[]
}

type ApiSearchResponse = {
  itineraries: ApiItinerary[]
  provenance: Provenance
}

type MappedSearch = {
  itineraries: Itinerary[]
  provenance: Provenance
}

type ApiErrorBody = { error?: { code?: string; message?: string } }

const RISK_LABELS: Record<string, Itinerary['risk'][number]> = {
  'self-transfer': 'Self-transfer',
  'ota-only': 'OTA only',
  'hidden-city': 'Hidden city',
}

function toAirport(place: ApiPlace): Airport {
  return {
    code: place.code,
    city: place.city,
    name: place.name,
    country: place.countryName,
  }
}

function clock(value: string) {
  return value.slice(11, 16)
}

function toItinerary(item: ApiItinerary): Itinerary {
  if (item.currency !== 'EUR') {
    throw new Error(`Unsupported currency ${item.currency} from the data plane`)
  }
  const verifiedMinutesAgo = item.verifiedAt
    ? Math.max(0, Math.round((Date.now() - Date.parse(item.verifiedAt)) / 60_000))
    : null

  return {
    id: item.id,
    fareFamily: item.fareFamily ?? 'Economy',
    segments: item.segments.map((segment) => ({
      from: toAirport(segment.from),
      to: toAirport(segment.to),
      carrier: segment.marketingCarrier,
      flight: segment.serviceNumber,
      depart: clock(segment.departLocal),
      arrive: clock(segment.arriveLocal),
      durationMinutes: segment.durationMinutes,
    })),
    total: Math.round(item.totalPrice),
    currency: 'EUR',
    durationMinutes: item.totalDurationMinutes,
    stops: item.stops,
    bagsIncluded: item.bagsIncluded.checked,
    bagFee: item.priceBreakdown.bags,
    refundable: item.fareRules.refundable,
    changeFee: item.fareRules.changeFee ?? 0,
    provider: item.providerName,
    verifiedMinutesAgo,
    risk: item.risk
      .map((risk) => RISK_LABELS[risk])
      .filter((risk): risk is Itinerary['risk'][number] => Boolean(risk)),
  }
}

export function localProvenance(sampleSize: number): Provenance {
  return {
    dataCardId: 'fares-fixture-local',
    source: 'fixtures',
    observedAt: new Date().toISOString(),
    freshness: 'fixture',
    ageSeconds: 0,
    degraded: false,
    sampleSize,
    notes: ['Bundled demo fixtures in the web app.'],
  }
}

function localOutcome(query: Query, degradedReason?: string): SearchOutcome {
  const itineraries = itinerariesByRoute[routeKey(query.origin, query.destination)] ?? []
  const provenance = localProvenance(itineraries.length)
  if (degradedReason) {
    provenance.degraded = true
    provenance.degradedReason = degradedReason
  }
  return {
    itineraries,
    provenance,
    notice: degradedReason,
  }
}

async function postSearch(body: unknown): Promise<MappedSearch> {
  const response = await fetch(`${API_BASE}/api/search`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
    signal: AbortSignal.timeout(15_000),
  })

  if (!response.ok) {
    let code = 'http_error'
    let message = `The data plane returned ${response.status}.`
    try {
      const payload = (await response.json()) as ApiErrorBody
      code = payload.error?.code ?? code
      message = payload.error?.message ?? message
    } catch {
      // A non-JSON error page keeps the status message.
    }
    throw new ApiRequestError(response.status, code, message)
  }

  const payload = (await response.json()) as ApiSearchResponse
  return {
    itineraries: payload.itineraries.map(toItinerary),
    provenance: payload.provenance,
  }
}

function requestBody(query: Query) {
  return {
    origin: query.origin,
    destination: query.destination,
    departDate: query.depart,
    returnDate: query.tripType === 'return' ? query.returnDate : null,
    adults: query.passengers,
    checkedBags: query.bags,
    carryOnBags: 1,
    currency: 'EUR',
    source: 'auto',
  }
}

export async function searchFlights(query: Query): Promise<SearchOutcome> {
  if (!apiEnabled) return localOutcome(query)

  try {
    const response = await postSearch(requestBody(query))
    return { itineraries: response.itineraries, provenance: response.provenance }
  } catch (error) {
    if (error instanceof ApiRequestError && error.code === 'return_trip_not_supported') {
      // Live fares are one-way in this release. Ask the same service for the
      // fixture answer so the demo round trip still prices from one source.
      try {
        const response = await postSearch({ ...requestBody(query), source: 'fixtures' })
        return {
          itineraries: response.itineraries,
          provenance: response.provenance,
          notice: 'Live pricing is one-way in this release; this round trip is priced from demo fixtures.',
        }
      } catch {
        return localOutcome(query, 'Live pricing is one-way in this release; showing bundled demo fixtures.')
      }
    }

    if (error instanceof ApiRequestError && error.status < 500) {
      throw error
    }

    return localOutcome(query, 'The data plane is unreachable; showing bundled demo fixtures.')
  }
}
