import type { FeatureCollection, LineString, Position } from 'geojson'

/** The Nagai day palette: flat fills, one warm accent, ink for data. */
export const NAGAI = {
  skyTop: '#3FA6D8',
  skyMid: '#8ED0E6',
  skyLow: '#E4F2EC',
  sea: '#1E86A8',
  seaDeep: '#14627F',
  sand: '#F0E2C4',
  ink: '#12324B',
  white: '#FDF8EE',
  sun: '#F2795C',
  cloudShade: '#BFDCE8',
  palm: '#103A4A',
} as const

const toRad = (deg: number) => (deg * Math.PI) / 180
const toDeg = (rad: number) => (rad * 180) / Math.PI

/** Points along the great circle from a to b, so routes curve with the globe. */
export function greatCircle(a: Position, b: Position, steps = 64): Position[] {
  const [lng1, lat1] = [toRad(a[0]), toRad(a[1])]
  const [lng2, lat2] = [toRad(b[0]), toRad(b[1])]
  const d =
    2 *
    Math.asin(
      Math.sqrt(
        Math.sin((lat2 - lat1) / 2) ** 2 + Math.cos(lat1) * Math.cos(lat2) * Math.sin((lng2 - lng1) / 2) ** 2,
      ),
    )
  if (d === 0) return [a, b]
  const points: Position[] = []
  for (let i = 0; i <= steps; i++) {
    const f = i / steps
    const A = Math.sin((1 - f) * d) / Math.sin(d)
    const B = Math.sin(f * d) / Math.sin(d)
    const x = A * Math.cos(lat1) * Math.cos(lng1) + B * Math.cos(lat2) * Math.cos(lng2)
    const y = A * Math.cos(lat1) * Math.sin(lng1) + B * Math.cos(lat2) * Math.sin(lng2)
    const z = A * Math.sin(lat1) + B * Math.sin(lat2)
    points.push([toDeg(Math.atan2(y, x)), toDeg(Math.atan2(z, Math.sqrt(x * x + y * y)))])
  }
  return points
}

/** A 10° graticule; faint curved lines are what make a flat-shaded sphere read as round. */
export function graticule(): FeatureCollection<LineString> {
  const features: FeatureCollection<LineString>['features'] = []
  for (let lng = -180; lng < 180; lng += 10) {
    const coordinates: Position[] = []
    for (let lat = -80; lat <= 80; lat += 2) coordinates.push([lng, lat])
    features.push({ type: 'Feature', properties: {}, geometry: { type: 'LineString', coordinates } })
  }
  for (let lat = -80; lat <= 80; lat += 10) {
    const coordinates: Position[] = []
    for (let lng = -180; lng <= 180; lng += 2) coordinates.push([lng, lat])
    features.push({ type: 'Feature', properties: {}, geometry: { type: 'LineString', coordinates } })
  }
  return { type: 'FeatureCollection', features }
}

/** One palm frond as a flat silhouette path: a curved spine with leaflets on both sides. */
function frond(ox: number, oy: number, angle: number, length: number, bend: number) {
  const spine: Array<[number, number, number]> = []
  for (let i = 0; i <= 24; i++) {
    const t = i / 24
    const a = angle + bend * t * t
    spine.push([ox + Math.cos(a) * length * t, oy + Math.sin(a) * length * t, a])
  }
  let leaves = ''
  for (let i = 3; i < spine.length; i++) {
    const [x, y, a] = spine[i]
    const t = i / 24
    const leaf = (1 - t) * 46 + 10
    for (const side of [-1, 1]) {
      const la = a + side * (1.05 - 0.35 * t) + 0.35
      leaves += `M${x.toFixed(1)} ${y.toFixed(1)}L${(x + Math.cos(la) * leaf).toFixed(1)} ${(y + Math.sin(la) * leaf).toFixed(1)}L${(x + Math.cos(a) * 4).toFixed(1)} ${(y + Math.sin(a) * 4).toFixed(1)}Z`
    }
  }
  const stem = 'M' + spine.map(([x, y]) => `${x.toFixed(1)} ${y.toFixed(1)}`).join(' L')
  return { leaves, stem }
}

/** Fronds hanging in from the top-right corner of a 360×260 box. */
export const PALM_FRONDS = (
  [
    [2.05, 300, 0.55],
    [2.45, 330, 0.6],
    [2.85, 280, 0.5],
    [1.7, 230, 0.45],
    [3.2, 220, 0.35],
  ] as const
).map(([angle, length, bend]) => frond(390, -30, angle, length, bend))
