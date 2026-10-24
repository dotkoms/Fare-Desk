export function formatDuration(minutes: number) {
  const hours = Math.floor(minutes / 60)
  const rest = minutes % 60
  return `${hours}h ${String(rest).padStart(2, '0')}`
}

/** A whole-unit price in the itinerary's currency: "€119", "$342", "£88". */
export function formatMoney(value: number, currency: string) {
  return new Intl.NumberFormat('en', { style: 'currency', currency, maximumFractionDigits: 0 }).format(
    Math.round(value),
  )
}

const EURO_AREA = new Set([
  'AT',
  'BE',
  'HR',
  'CY',
  'EE',
  'FI',
  'FR',
  'DE',
  'GR',
  'IE',
  'IT',
  'LV',
  'LT',
  'LU',
  'MT',
  'NL',
  'PT',
  'SK',
  'SI',
  'ES',
])
const NATIONAL: Record<string, string> = { US: 'USD', GB: 'GBP', CA: 'CAD', AU: 'AUD', CH: 'CHF', JP: 'JPY', MX: 'MXN' }

/** The currency a traveller departing from this country most likely pays in; USD otherwise. */
export function currencyForCountry(countryCode: string) {
  if (EURO_AREA.has(countryCode)) return 'EUR'
  return NATIONAL[countryCode] ?? 'USD'
}
