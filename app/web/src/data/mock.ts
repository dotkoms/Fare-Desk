import type { Airport, Itinerary, RouteSnapshot } from '../types'

export const airports: Airport[] = [
  { code: 'BER', city: 'Berlin', name: 'Berlin Brandenburg', country: 'Germany' },
  { code: 'LIS', city: 'Lisbon', name: 'Humberto Delgado', country: 'Portugal' },
  { code: 'LHR', city: 'London', name: 'Heathrow', country: 'United Kingdom' },
  { code: 'BCN', city: 'Barcelona', name: 'El Prat', country: 'Spain' },
  { code: 'JFK', city: 'New York', name: 'John F. Kennedy', country: 'United States' },
  { code: 'AMS', city: 'Amsterdam', name: 'Schiphol', country: 'Netherlands' },
  { code: 'MAD', city: 'Madrid', name: 'Barajas', country: 'Spain' },
  { code: 'FCO', city: 'Rome', name: 'Fiumicino', country: 'Italy' },
  { code: 'STN', city: 'London', name: 'Stansted', country: 'United Kingdom' },
]

const byCode = Object.fromEntries(airports.map((airport) => [airport.code, airport]))

function airport(code: string): Airport {
  const value = byCode[code]
  if (!value) throw new Error(`Unknown fixture airport ${code}`)
  return value
}

const BER = airport('BER')
const LIS = airport('LIS')
const LHR = airport('LHR')
const BCN = airport('BCN')
const JFK = airport('JFK')
const AMS = airport('AMS')
const MAD = airport('MAD')
const FCO = airport('FCO')
const STN = airport('STN')

const berLis: Itinerary[] = [
  {
    id: 'iberia-mad',
    fareFamily: 'Economy Classic',
    segments: [
      { from: BER, to: MAD, carrier: 'Iberia', flight: 'IB 3671', depart: '06:40', arrive: '09:35', durationMinutes: 175 },
      { from: MAD, to: LIS, carrier: 'Iberia', flight: 'IB 3118', depart: '10:40', arrive: '11:00', durationMinutes: 80 },
    ],
    total: 129,
    currency: 'EUR',
    durationMinutes: 320,
    stops: 1,
    bagsIncluded: 1,
    refundable: false,
    changeFee: 75,
    provider: 'Iberia',
    verifiedMinutesAgo: 4,
    risk: [],
  },
  {
    id: 'tap-nonstop',
    fareFamily: 'Economy Light',
    segments: [{ from: BER, to: LIS, carrier: 'TAP Air Portugal', flight: 'TP 1041', depart: '07:10', arrive: '09:50', durationMinutes: 220 }],
    total: 119,
    currency: 'EUR',
    durationMinutes: 220,
    stops: 0,
    bagsIncluded: 1,
    refundable: false,
    changeFee: 75,
    provider: 'TAP Air Portugal',
    verifiedMinutesAgo: 4,
    risk: [],
  },
  {
    id: 'ita-fco',
    fareFamily: 'Economy Classic',
    segments: [
      { from: BER, to: FCO, carrier: 'ITA Airways', flight: 'AZ 731', depart: '08:30', arrive: '10:35', durationMinutes: 125 },
      { from: FCO, to: LIS, carrier: 'ITA Airways', flight: 'AZ 752', depart: '12:10', arrive: '14:20', durationMinutes: 130 },
    ],
    total: 128,
    currency: 'EUR',
    durationMinutes: 350,
    stops: 1,
    bagsIncluded: 1,
    refundable: false,
    changeFee: 80,
    provider: 'ITA Airways',
    verifiedMinutesAgo: 7,
    risk: [],
  },
  {
    id: 'easyjet-nonstop',
    fareFamily: 'Standard',
    segments: [{ from: BER, to: LIS, carrier: 'easyJet', flight: 'EZY 8901', depart: '12:20', arrive: '15:15', durationMinutes: 235 }],
    total: 149,
    currency: 'EUR',
    durationMinutes: 235,
    stops: 0,
    bagsIncluded: 0,
    refundable: false,
    changeFee: 60,
    provider: 'easyJet',
    verifiedMinutesAgo: 11,
    risk: [],
  },
  {
    id: 'ryanair-stn',
    fareFamily: 'Basic',
    segments: [
      { from: BER, to: STN, carrier: 'Ryanair', flight: 'FR 146', depart: '07:00', arrive: '07:55', durationMinutes: 115 },
      { from: STN, to: LIS, carrier: 'Ryanair', flight: 'FR 1882', depart: '09:10', arrive: '11:55', durationMinutes: 165 },
    ],
    total: 104,
    currency: 'EUR',
    durationMinutes: 355,
    stops: 1,
    bagsIncluded: 0,
    refundable: false,
    changeFee: 0,
    provider: 'Kiwi.com',
    verifiedMinutesAgo: 19,
    risk: ['Self-transfer', 'OTA only'],
  },
  {
    id: 'tap-flex',
    fareFamily: 'Economy Flex',
    segments: [{ from: BER, to: LIS, carrier: 'TAP Air Portugal', flight: 'TP 1045', depart: '18:00', arrive: '20:45', durationMinutes: 225 }],
    total: 189,
    currency: 'EUR',
    durationMinutes: 225,
    stops: 0,
    bagsIncluded: 1,
    refundable: true,
    changeFee: 0,
    provider: 'TAP Air Portugal',
    verifiedMinutesAgo: 4,
    risk: [],
  },
]

const berLhr: Itinerary[] = [
  {
    id: 'ba-lhr',
    fareFamily: 'Euro Traveller',
    segments: [{ from: BER, to: LHR, carrier: 'British Airways', flight: 'BA 991', depart: '07:25', arrive: '08:30', durationMinutes: 125 }],
    total: 168,
    currency: 'EUR',
    durationMinutes: 125,
    stops: 0,
    bagsIncluded: 1,
    refundable: false,
    changeFee: 70,
    provider: 'British Airways',
    verifiedMinutesAgo: 6,
    risk: [],
  },
  {
    id: 'easyjet-lhr',
    fareFamily: 'Standard',
    segments: [{ from: BER, to: LHR, carrier: 'easyJet', flight: 'EZY 8402', depart: '10:15', arrive: '11:25', durationMinutes: 130 }],
    total: 148,
    currency: 'EUR',
    durationMinutes: 130,
    stops: 0,
    bagsIncluded: 0,
    refundable: false,
    changeFee: 60,
    provider: 'easyJet',
    verifiedMinutesAgo: 10,
    risk: [],
  },
  {
    id: 'klm-ams',
    fareFamily: 'Economy',
    segments: [
      { from: BER, to: AMS, carrier: 'KLM', flight: 'KL 1820', depart: '08:10', arrive: '09:35', durationMinutes: 85 },
      { from: AMS, to: LHR, carrier: 'KLM', flight: 'KL 1011', depart: '10:25', arrive: '10:45', durationMinutes: 80 },
    ],
    total: 155,
    currency: 'EUR',
    durationMinutes: 215,
    stops: 1,
    bagsIncluded: 1,
    refundable: false,
    changeFee: 75,
    provider: 'KLM',
    verifiedMinutesAgo: 12,
    risk: [],
  },
]

const berBcn: Itinerary[] = [
  {
    id: 'vueling-bcn',
    fareFamily: 'Basic',
    segments: [{ from: BER, to: BCN, carrier: 'Vueling', flight: 'VY 6201', depart: '09:20', arrive: '11:45', durationMinutes: 145 }],
    total: 92,
    currency: 'EUR',
    durationMinutes: 145,
    stops: 0,
    bagsIncluded: 0,
    refundable: false,
    changeFee: 50,
    provider: 'Vueling',
    verifiedMinutesAgo: 8,
    risk: [],
  },
  {
    id: 'easyjet-bcn',
    fareFamily: 'Standard',
    segments: [{ from: BER, to: BCN, carrier: 'easyJet', flight: 'EZY 8703', depart: '13:05', arrive: '15:30', durationMinutes: 145 }],
    total: 96,
    currency: 'EUR',
    durationMinutes: 145,
    stops: 0,
    bagsIncluded: 0,
    refundable: false,
    changeFee: 60,
    provider: 'easyJet',
    verifiedMinutesAgo: 13,
    risk: [],
  },
  {
    id: 'ryanair-bcn',
    fareFamily: 'Basic',
    segments: [{ from: BER, to: BCN, carrier: 'Ryanair', flight: 'FR 134', depart: '06:15', arrive: '08:45', durationMinutes: 150 }],
    total: 84,
    currency: 'EUR',
    durationMinutes: 150,
    stops: 0,
    bagsIncluded: 0,
    refundable: false,
    changeFee: 0,
    provider: 'Ryanair',
    verifiedMinutesAgo: 18,
    risk: ['OTA only'],
  },
]

const berJfk: Itinerary[] = [
  {
    id: 'norse-jfk',
    fareFamily: 'Economy Light',
    segments: [{ from: BER, to: JFK, carrier: 'Norse Atlantic', flight: 'N0 401', depart: '12:40', arrive: '15:55', durationMinutes: 495 }],
    total: 389,
    currency: 'EUR',
    durationMinutes: 495,
    stops: 0,
    bagsIncluded: 1,
    refundable: false,
    changeFee: 120,
    provider: 'Norse Atlantic',
    verifiedMinutesAgo: 5,
    risk: [],
  },
  {
    id: 'united-lhr',
    fareFamily: 'Economy',
    segments: [
      { from: BER, to: LHR, carrier: 'United', flight: 'UA 963', depart: '09:10', arrive: '10:20', durationMinutes: 130 },
      { from: LHR, to: JFK, carrier: 'United', flight: 'UA 15', depart: '12:30', arrive: '15:35', durationMinutes: 485 },
    ],
    total: 420,
    currency: 'EUR',
    durationMinutes: 745,
    stops: 1,
    bagsIncluded: 1,
    refundable: false,
    changeFee: 100,
    provider: 'United',
    verifiedMinutesAgo: 9,
    risk: [],
  },
]

export const itinerariesByRoute: Record<string, Itinerary[]> = {
  'BER-LIS': berLis,
  'BER-LHR': berLhr,
  'BER-BCN': berBcn,
  'BER-JFK': berJfk,
}

export const snapshots: Record<string, RouteSnapshot> = {
  'BER-LIS': {
    route: 'BER → LIS',
    current: 119,
    typicalLow: 110,
    typicalHigh: 151,
    percentile: 12,
    sample: 84,
    recommendation: 'Book now',
    trackRecord: { correct: 7, cases: 10 },
    expected7: [122, 140],
    expected21: [150, 210],
    downside: '18% chance it falls below €110 if you wait',
    history: [151, 148, 155, 160, 158, 149, 145, 151, 147, 139, 142, 138, 145, 151, 149, 143, 136, 132, 128, 131, 127, 124, 121, 119],
  },
  'BER-LHR': {
    route: 'BER → LHR',
    current: 148,
    typicalLow: 120,
    typicalHigh: 210,
    percentile: 44,
    sample: 61,
    recommendation: 'Watch',
    trackRecord: { correct: 5, cases: 10 },
    expected7: [145, 170],
    expected21: [160, 205],
    downside: '23% chance it falls below €135 if you wait',
    history: [190, 182, 177, 185, 179, 174, 168, 171, 165, 162, 170, 166, 159, 162, 157, 154, 151, 149, 152, 148, 151, 147, 150, 148],
  },
  'BER-BCN': {
    route: 'BER → BCN',
    current: 96,
    typicalLow: 80,
    typicalHigh: 140,
    percentile: 20,
    sample: 73,
    recommendation: 'Book now',
    trackRecord: { correct: 6, cases: 10 },
    expected7: [98, 118],
    expected21: [120, 155],
    downside: '21% chance it falls below €88 if you wait',
    history: [132, 129, 125, 128, 124, 121, 118, 122, 119, 116, 114, 118, 112, 109, 111, 106, 104, 101, 99, 98, 97, 100, 98, 96],
  },
  'BER-JFK': {
    route: 'BER → JFK',
    current: 389,
    typicalLow: 340,
    typicalHigh: 520,
    percentile: 18,
    sample: 52,
    recommendation: 'Book now',
    trackRecord: { correct: 6, cases: 10 },
    expected7: [395, 460],
    expected21: [470, 610],
    downside: '17% chance it falls below €360 if you wait',
    history: [620, 590, 575, 580, 560, 545, 550, 530, 515, 505, 498, 510, 480, 465, 455, 440, 430, 420, 410, 402, 396, 392, 390, 389],
  },
}

export function routeKey(origin: string, destination: string) {
  return `${origin}-${destination}`
}
