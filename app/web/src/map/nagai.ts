import type { FeatureCollection, LineString, Position } from 'geojson'

/** The Nagai day palette: flat fills, one warm accent, ink for data. */
export const NAGAI = {
  skyTop: '#1B5FAE',
  skyHigh: '#1E6BB8',
  skyMid: '#4FAEDD',
  skyPale: '#A9DDEA',
  skyLow: '#EEF7F0',
  sea: '#1E86A8',
  seaDeep: '#14627F',
  /** Pool-water turquoise for the shallows ringing every coast. */
  shallow: '#2FA9C2',
  shallowLight: '#4CC8D4',
  /** Shadows in Nagai's light are cool and crisp, never grey. */
  shadow: '#16567A',
  sand: '#F4E5C9',
  ink: '#12324B',
  white: '#FDF8EE',
  sun: '#F2795C',
  cloudMid: '#D9EBF3',
  cloudShade: '#A9C7DD',
  palm: '#103A4A',
  /** Fronds further back, one step lighter: flat depth without shading. */
  palmBack: '#1F5A66',
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

/**
 * Palm fronds hanging into the top-right corner of a 420×300 box, back layer first, so the sky
 * is framed the way Nagai frames it.
 */
export const PALM = (
  [
    [2.3, 300, 0.5, true],
    [1.9, 250, 0.4, true],
    [2.75, 320, 0.45, true],
    [2.05, 310, 0.55, false],
    [2.5, 340, 0.6, false],
    [2.95, 290, 0.45, false],
    [1.65, 230, 0.4, false],
    [3.25, 230, 0.3, false],
  ] as const
).map(([angle, length, bend, back]) => ({ ...frond(450, -30, angle, length, bend), back }))

/** Sun glitter: short horizontal strokes scattered over the sea, drawn under the land. */
export function glitter(count = 1400): FeatureCollection<LineString> {
  let seed = 7
  const random = () => {
    seed = (seed * 16807) % 2147483647
    return seed / 2147483647
  }
  const features: FeatureCollection<LineString>['features'] = []
  for (let i = 0; i < count; i++) {
    const lat = -60 + random() * 125
    const lng = -180 + random() * 360
    const length = 0.6 + random() * 1.8
    features.push({
      type: 'Feature',
      properties: {},
      geometry: { type: 'LineString', coordinates: [[lng, lat], [lng + length, lat]] },
    })
  }
  return { type: 'FeatureCollection', features }
}

