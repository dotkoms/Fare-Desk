import type { Airport, Itinerary, Query } from '../types'

/**
 * The Fare Desk data plane. In development requests go to this server's own origin and Vite
 * proxies /api to the local service on :8080, so any dev port works without a CORS allowance.
 * A production build needs VITE_API_BASE; there is no bundled fallback data.
 */
const configuredBase = (import.meta.env.VITE_API_BASE as string | undefined)?.trim()
export const API_BASE = configuredBase ? configuredBase.replace(/\/+$/, '') : ''

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
  /** Return flights for a round trip, each its own one-way ticket. Empty for one way. */
  returns: Itinerary[]
  provenance: Provenance
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

export type Place = Airport & { id: string; countryCode: string }

type ApiPlace = {
  id: string
  code: string
  city: string
  name: string
  country: string
  countryName: string
  lat: number
  lng: number
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
  providerUrl: string
  verifiedAt: string | null
  risk: string[]
  connectionWarnings?: string[]
}

type ApiSearchResponse = {
  itineraries: ApiItinerary[]
  returnItineraries?: ApiItinerary[]
  provenance: Provenance
}

type ApiErrorBody = { error?: { code?: string; message?: string } }

const RISK_LABELS: Record<string, Itinerary['risk'][number]> = {
  'self-transfer': 'Self-transfer',
  'ota-only': 'OTA only',
  'hidden-city': 'Hidden city',
}

function toPlace(place: ApiPlace): Place {
  return {
    id: place.id,
    code: place.code,
    city: place.city,
    name: place.name,
    country: place.countryName,
    countryCode: place.country,
    lat: place.lat,
    lng: place.lng,
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
      from: toPlace(segment.from),
      to: toPlace(segment.to),
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
    providerUrl: item.providerUrl,
    connectionWarnings: item.connectionWarnings ?? [],
    verifiedMinutesAgo,
    risk: item.risk.map((risk) => RISK_LABELS[risk]).filter((risk): risk is Itinerary['risk'][number] => Boolean(risk)),
  }
}

async function request<T>(path: string, init?: RequestInit & { timeoutMs?: number }): Promise<T> {
  let response: Response
  try {
    response = await fetch(`${API_BASE}${path}`, {
      ...init,
      signal: init?.signal ?? AbortSignal.timeout(init?.timeoutMs ?? 15_000),
    })
  } catch (cause) {
    if (cause instanceof DOMException && cause.name === 'AbortError') throw cause
    throw new ApiRequestError(0, 'unreachable', 'The Fare Desk data plane is not reachable.')
  }

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

  return (await response.json()) as T
}

export async function searchFlights(query: Query): Promise<SearchOutcome> {
  const payload = await request<ApiSearchResponse>('/api/search', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      origin: query.origin,
      destination: query.destination,
      departDate: query.depart,
      returnDate: query.tripType === 'return' ? query.returnDate : null,
      adults: query.passengers,
      checkedBags: query.bags,
      carryOnBags: 1,
      currency: 'EUR',
      source: 'auto',
    }),
  })
  return {
    itineraries: payload.itineraries.map(toItinerary),
    returns: (payload.returnItineraries ?? []).map(toItinerary),
    provenance: payload.provenance,
  }
}

export async function searchPlaces(text: string, signal?: AbortSignal): Promise<Place[]> {
  const params = new URLSearchParams({ q: text, limit: '8' })
  const payload = await request<{ places: ApiPlace[] }>(`/api/places?${params}`, { signal })
  return payload.places.map(toPlace)
}

/** The airport with exactly this IATA code, or null. */
export async function placeByCode(code: string): Promise<Place | null> {
  const places = await searchPlaces(code)
  return places.find((place) => place.code === code.toUpperCase()) ?? null
}

export type Verdict = 'buy' | 'wait' | 'watch' | 'insufficient'

export type RouteIntel = {
  route: string
  travelDate: string
  currency: string
  currentPrice: number | null
  percentile: number | null
  median: number | null
  p10: number | null
  p90: number | null
  history: Array<{ date: string; price: number }>
  sampleSize: number
  window: { from: string; to: string; days: number } | null
  recommendation: Verdict | null
  minObservations: number
  expected7?: { low: number; high: number }
  expected21?: { low: number; high: number }
  /** Percent of observed 7-day moves that would take today's fare below p10. */
  downside?: number
  backtest?: { cases: number; waitWins: number; avgSaving: number; worstLoss: number }
  trackRecord?: { correct: number; cases: number }
  reasons?: string[]
  changeMyMind?: string
}

export async function getRouteIntel(
  from: string,
  to: string,
  date: string,
  signal?: AbortSignal,
): Promise<{ intel: RouteIntel; provenance: Provenance }> {
  const params = new URLSearchParams({ from, to, date })
  return request(`/api/route-intel?${params}`, { signal })
}
