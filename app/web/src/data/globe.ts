import { feature } from 'topojson-client'
import landTopo from 'world-atlas/land-110m.json'
import type { Airport } from '../types'

export type GlobeAirport = Airport & { lat: number; lng: number; traffic: number }

export const globeAirports: GlobeAirport[] = [
  { code: 'BER', lat: 52.3667, lng: 13.5033, city: 'Berlin', name: 'Brandenburg', country: 'Germany', traffic: 34 },
  { code: 'LIS', lat: 38.7742, lng: -9.1342, city: 'Lisbon', name: 'Humberto Delgado', country: 'Portugal', traffic: 34 },
  { code: 'LHR', lat: 51.47, lng: -0.4543, city: 'London', name: 'Heathrow', country: 'United Kingdom', traffic: 79 },
  { code: 'BCN', lat: 41.2974, lng: 2.0833, city: 'Barcelona', name: 'El Prat', country: 'Spain', traffic: 55 },
  { code: 'JFK', lat: 40.6413, lng: -73.7781, city: 'New York', name: 'John F. Kennedy', country: 'United States', traffic: 62 },
  { code: 'AMS', lat: 52.3105, lng: 4.7683, city: 'Amsterdam', name: 'Schiphol', country: 'Netherlands', traffic: 66 },
  { code: 'MAD', lat: 40.4936, lng: -3.5668, city: 'Madrid', name: 'Barajas', country: 'Spain', traffic: 60 },
  { code: 'FCO', lat: 41.8003, lng: 12.2389, city: 'Rome', name: 'Fiumicino', country: 'Italy', traffic: 40 },
  { code: 'STN', lat: 51.885, lng: 0.235, city: 'London', name: 'Stansted', country: 'United Kingdom', traffic: 28 },
  { code: 'CDG', lat: 49.0097, lng: 2.5479, city: 'Paris', name: 'Charles de Gaulle', country: 'France', traffic: 70 },
  { code: 'FRA', lat: 50.0379, lng: 8.5622, city: 'Frankfurt', name: 'Frankfurt', country: 'Germany', traffic: 70 },
  { code: 'MUC', lat: 48.3538, lng: 11.7861, city: 'Munich', name: 'Munich', country: 'Germany', traffic: 48 },
  { code: 'ZRH', lat: 47.4647, lng: 8.5492, city: 'Zurich', name: 'Zurich', country: 'Switzerland', traffic: 31 },
  { code: 'VIE', lat: 48.1103, lng: 16.5697, city: 'Vienna', name: 'Vienna', country: 'Austria', traffic: 30 },
  { code: 'CPH', lat: 55.618, lng: 12.6508, city: 'Copenhagen', name: 'Kastrup', country: 'Denmark', traffic: 30 },
  { code: 'ARN', lat: 59.6519, lng: 17.9186, city: 'Stockholm', name: 'Arlanda', country: 'Sweden', traffic: 25 },
  { code: 'OSL', lat: 60.1939, lng: 11.1004, city: 'Oslo', name: 'Gardermoen', country: 'Norway', traffic: 28 },
  { code: 'HEL', lat: 60.3172, lng: 24.9633, city: 'Helsinki', name: 'Vantaa', country: 'Finland', traffic: 20 },
  { code: 'DUB', lat: 53.4213, lng: -6.2701, city: 'Dublin', name: 'Dublin', country: 'Ireland', traffic: 33 },
  { code: 'MAN', lat: 53.3537, lng: -2.275, city: 'Manchester', name: 'Manchester', country: 'United Kingdom', traffic: 28 },
  { code: 'EDI', lat: 55.95, lng: -3.3725, city: 'Edinburgh', name: 'Edinburgh', country: 'United Kingdom', traffic: 14 },
  { code: 'BRU', lat: 50.9014, lng: 4.4844, city: 'Brussels', name: 'Brussels', country: 'Belgium', traffic: 25 },
  { code: 'GVA', lat: 46.2381, lng: 6.1089, city: 'Geneva', name: 'Geneva', country: 'Switzerland', traffic: 18 },
  { code: 'PRG', lat: 50.1008, lng: 14.26, city: 'Prague', name: 'Vaclav Havel', country: 'Czechia', traffic: 18 },
  { code: 'WAW', lat: 52.1657, lng: 20.9671, city: 'Warsaw', name: 'Chopin', country: 'Poland', traffic: 20 },
  { code: 'BUD', lat: 47.4369, lng: 19.2556, city: 'Budapest', name: 'Liszt Ferenc', country: 'Hungary', traffic: 16 },
  { code: 'IST', lat: 41.2753, lng: 28.7519, city: 'Istanbul', name: 'Istanbul', country: 'Turkey', traffic: 80 },
  { code: 'ATH', lat: 37.9364, lng: 23.9445, city: 'Athens', name: 'Eleftherios Venizelos', country: 'Greece', traffic: 25 },
  { code: 'DXB', lat: 25.2532, lng: 55.3657, city: 'Dubai', name: 'Dubai', country: 'UAE', traffic: 87 },
  { code: 'SIN', lat: 1.3644, lng: 103.9915, city: 'Singapore', name: 'Changi', country: 'Singapore', traffic: 68 },
  { code: 'HKG', lat: 22.308, lng: 113.9185, city: 'Hong Kong', name: 'Hong Kong', country: 'Hong Kong', traffic: 70 },
  { code: 'NRT', lat: 35.772, lng: 140.3929, city: 'Tokyo', name: 'Narita', country: 'Japan', traffic: 35 },
  { code: 'LAX', lat: 33.9416, lng: -118.4085, city: 'Los Angeles', name: 'Los Angeles', country: 'United States', traffic: 88 },
  { code: 'SFO', lat: 37.6213, lng: -122.379, city: 'San Francisco', name: 'San Francisco', country: 'United States', traffic: 50 },
  { code: 'ORD', lat: 41.9742, lng: -87.9073, city: 'Chicago', name: "O'Hare", country: 'United States', traffic: 80 },
  { code: 'MIA', lat: 25.7959, lng: -80.287, city: 'Miami', name: 'Miami', country: 'United States', traffic: 52 },
  { code: 'YYZ', lat: 43.6777, lng: -79.6248, city: 'Toronto', name: 'Pearson', country: 'Canada', traffic: 50 },
  { code: 'GRU', lat: -23.4356, lng: -46.4731, city: 'Sao Paulo', name: 'Guarulhos', country: 'Brazil', traffic: 40 },
  { code: 'JNB', lat: -26.1367, lng: 28.2411, city: 'Johannesburg', name: 'O.R. Tambo', country: 'South Africa', traffic: 21 },
  { code: 'SYD', lat: -33.9399, lng: 151.1753, city: 'Sydney', name: 'Kingsford Smith', country: 'Australia', traffic: 44 },
  { code: 'AKL', lat: -37.0082, lng: 174.785, city: 'Auckland', name: 'Auckland', country: 'New Zealand', traffic: 21 },
]

const topo = landTopo as unknown as { objects: { land: any } }
export const landFeatures = (feature(topo as any, topo.objects.land) as any).features as Array<
  Record<string, unknown>
>

export function airportByCode(code: string) {
  return globeAirports.find((airport) => airport.code === code)
}
