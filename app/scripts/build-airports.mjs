#!/usr/bin/env node
// Builds app/data/airports.json from OurAirports (public domain) plus IANA
// timezones from mwgg/Airports (MIT, a maintained OurAirports derivative).
// Run from the repository root:
//
//   node app/scripts/build-airports.mjs
//
// Output is committed so the API has no network dependency at startup.

import { writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const HERE = dirname(fileURLToPath(import.meta.url))
const OUT = join(HERE, '..', 'data', 'airports.json')

const OURAIRPORTS = 'https://davidmegginson.github.io/ourairports-data/airports.csv'
const TZ_SOURCE = 'https://raw.githubusercontent.com/mwgg/Airports/master/airports.json'

// Major multi-airport cities. Metro codes follow the IATA city code where one
// exists; the API uses them to expand a city search into its airports.
const METROS = {
  LHR: 'LON', LGW: 'LON', STN: 'LON', LTN: 'LON', LCY: 'LON', SEN: 'LON',
  CDG: 'PAR', ORY: 'PAR', BVA: 'PAR',
  JFK: 'NYC', LGA: 'NYC', EWR: 'NYC',
  FCO: 'ROM', CIA: 'ROM',
  MXP: 'MIL', LIN: 'MIL', BGY: 'MIL',
  NRT: 'TYO', HND: 'TYO',
  KIX: 'OSA', ITM: 'OSA',
  IAD: 'WAS', DCA: 'WAS', BWI: 'WAS',
  ORD: 'CHI', MDW: 'CHI',
  GRU: 'SAO', CGH: 'SAO', VCP: 'SAO',
  PVG: 'SHA', SHA: 'SHA', PEK: 'BJS', PKX: 'BJS',
  MOW: 'MOW', SVO: 'MOW', DME: 'MOW', VKO: 'MOW',
  TPE: 'TPE', TSA: 'TPE',
  ICN: 'SEL', GMP: 'SEL',
  YYZ: 'YTO', YTZ: 'YTO',
}

function parseCsv(text) {
  const rows = []
  let row = []
  let field = ''
  let quoted = false
  for (let i = 0; i < text.length; i++) {
    const ch = text[i]
    if (quoted) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          field += '"'
          i++
        } else {
          quoted = false
        }
      } else {
        field += ch
      }
      continue
    }
    if (ch === '"') quoted = true
    else if (ch === ',') {
      row.push(field)
      field = ''
    } else if (ch === '\n') {
      row.push(field)
      rows.push(row)
      row = []
      field = ''
    } else if (ch !== '\r') {
      field += ch
    }
  }
  if (field.length || row.length) {
    row.push(field)
    rows.push(row)
  }
  return rows
}

const [ourRes, tzRes] = await Promise.all([fetch(OURAIRPORTS), fetch(TZ_SOURCE)])
if (!ourRes.ok) throw new Error(`OurAirports ${ourRes.status}`)
if (!tzRes.ok) throw new Error(`timezone source ${tzRes.status}`)

const ourRows = parseCsv(await ourRes.text())
const ourHeader = ourRows[0]
const col = Object.fromEntries(ourHeader.map((name, index) => [name, index]))

const tzByIcao = new Map()
const tzByIata = new Map()
for (const airport of Object.values(await tzRes.json())) {
  if (!airport.tz) continue
  if (airport.icao) tzByIcao.set(airport.icao, airport.tz)
  if (airport.iata) tzByIata.set(airport.iata, airport.tz)
}
// Country names come from the runtime's CLDR data; the static list below is the fallback.
const regionNames = new Intl.DisplayNames(['en'], { type: 'region' })
const countryName = (iso) => {
  try {
    return regionNames.of(iso) || COUNTRY_NAMES[iso] || iso
  } catch {
    return COUNTRY_NAMES[iso] || iso
  }
}
const COUNTRY_NAMES = {
  AE: 'United Arab Emirates', AR: 'Argentina', AT: 'Austria', AU: 'Australia', BE: 'Belgium',
  BG: 'Bulgaria', BR: 'Brazil', CA: 'Canada', CH: 'Switzerland', CL: 'Chile', CN: 'China',
  CO: 'Colombia', CY: 'Cyprus', CZ: 'Czechia', DE: 'Germany', DK: 'Denmark', EG: 'Egypt',
  ES: 'Spain', ET: 'Ethiopia', FI: 'Finland', FR: 'France', GB: 'United Kingdom', GR: 'Greece',
  HK: 'Hong Kong', HR: 'Croatia', HU: 'Hungary', ID: 'Indonesia', IE: 'Ireland', IL: 'Israel',
  IN: 'India', IS: 'Iceland', IT: 'Italy', JP: 'Japan', KE: 'Kenya', KR: 'South Korea',
  LT: 'Lithuania', LU: 'Luxembourg', LV: 'Latvia', MA: 'Morocco', MX: 'Mexico', MY: 'Malaysia',
  NG: 'Nigeria', NL: 'Netherlands', NO: 'Norway', NZ: 'New Zealand', PE: 'Peru', PH: 'Philippines',
  PL: 'Poland', PT: 'Portugal', QA: 'Qatar', RO: 'Romania', RS: 'Serbia', RU: 'Russia',
  SA: 'Saudi Arabia', SE: 'Sweden', SG: 'Singapore', SI: 'Slovenia', SK: 'Slovakia',
  TH: 'Thailand', TR: 'Turkey', TW: 'Taiwan', UA: 'Ukraine', US: 'United States',
  VN: 'Vietnam', ZA: 'South Africa',
}

const places = []
const missingTz = []
for (const row of ourRows.slice(1)) {
  if (row.length <= col.iata_code) continue
  const type = row[col.type]
  if (type !== 'large_airport' && type !== 'medium_airport') continue
  if (row[col.scheduled_service] !== 'yes') continue
  const code = row[col.iata_code]
  if (!code || code.length !== 3) continue
  const lat = Number(row[col.latitude_deg])
  const lng = Number(row[col.longitude_deg])
  if (!Number.isFinite(lat) || !Number.isFinite(lng)) continue
  const city = row[col.municipality] || row[col.name]
  const iso = row[col.iso_country]
  const tz = tzByIcao.get(row[col.icao_code]) || tzByIata.get(code)
  if (!tz) {
    missingTz.push(code)
    continue
  }
  places.push({
    id: `airport:${code}`,
    code,
    name: row[col.name],
    city,
    country: iso,
    countryName: countryName(iso),
    lat,
    lng,
    timezone: tz,
    metro: METROS[code] || null,
    scheduledService: true,
    // Ranking only: large airports win ties in place search.
    size: type === 'large_airport' ? 'large' : 'medium',
  })
}

places.sort((a, b) => a.code.localeCompare(b.code))

const output = {
  generatedAt: new Date().toISOString(),
  sources: [
    { name: 'OurAirports', url: OURAIRPORTS, licence: 'Public domain', fields: 'airports, coordinates, scheduled service, size' },
    { name: 'mwgg/Airports', url: TZ_SOURCE, licence: 'MIT', fields: 'IANA timezone' },
  ],
  note: 'Large and medium airports with scheduled service. Timezones joined by IATA code; airports without a timezone are omitted.',
  places,
}

await writeFile(OUT, JSON.stringify(output))
console.log(`wrote ${places.length} airports to ${OUT}`)
console.log(`omitted ${missingTz.length} airports without a timezone: ${missingTz.slice(0, 20).join(', ')}`)
