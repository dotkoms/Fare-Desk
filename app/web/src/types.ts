export type Airport = {
  code: string
  city: string
  name: string
  country: string
}

export type Segment = {
  from: Airport
  to: Airport
  carrier: string
  flight: string
  depart: string
  arrive: string
  durationMinutes: number
}

export type Itinerary = {
  id: string
  segments: Segment[]
  total: number
  currency: 'EUR'
  durationMinutes: number
  stops: number
  bagsIncluded: number
  /** Money added for checked bags the fare does not include. */
  bagFee?: number
  refundable: boolean
  changeFee: number
  provider: string
  /** Null when the source could not confirm the price (fixtures, demo data). */
  verifiedMinutesAgo: number | null
  risk: Array<'Self-transfer' | 'OTA only' | 'Hidden city'>
  fareFamily: string
}

export type Recommendation = 'Book now' | 'Wait' | 'Watch' | 'Not enough data'

export type RouteSnapshot = {
  route: string
  current: number
  typicalLow: number
  typicalHigh: number
  percentile: number
  sample: number
  recommendation: Recommendation
  trackRecord: { correct: number; cases: number } | null
  expected7: [number, number]
  expected21: [number, number]
  downside: string
  history: number[]
}

export type Sort = 'best' | 'cheapest' | 'fastest'

export type Query = {
  tripType: 'return' | 'oneway'
  origin: string
  destination: string
  depart: string
  returnDate: string
  passengers: number
  bags: number
}
