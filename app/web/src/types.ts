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
  currency: string
  durationMinutes: number
  stops: number
  bagsIncluded: number
  /** Money added for checked bags the fare does not include. */
  bagFee?: number
  refundable: boolean
  changeFee: number
  /** False when the source gave no fare rules; then refundable and changeFee mean nothing. */
  rulesKnown: boolean
  provider: string
  /** Where "Open on provider" goes; the purchase happens there, never here. */
  providerUrl: string
  /** Layovers shorter than the airport's minimum connection time, in plain words. */
  connectionWarnings: string[]
  /** Null when the source could not confirm the price. */
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
  /** What the data plane quotes in; follows the origin airport's country. */
  currency: string
}
