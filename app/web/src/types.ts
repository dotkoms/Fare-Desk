export type Airport = {
  code: string
  city: string
  name: string
  country: string
  lat: number
  lng: number
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
  /** Where "Open on provider" goes; the purchase happens there, never here. */
  providerUrl: string
  /** Layovers shorter than the airport's minimum connection time, in plain words. */
  connectionWarnings: string[]
  /** Null when the source could not confirm the price (fixtures, demo data). */
  verifiedMinutesAgo: number | null
  risk: Array<'Self-transfer' | 'OTA only' | 'Hidden city'>
  fareFamily: string
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
