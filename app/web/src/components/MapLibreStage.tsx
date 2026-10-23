import { useEffect, useMemo, useRef, useState } from 'react'
import {
  LngLat,
  LngLatBounds,
  Map as MapLibreMap,
  Marker,
  setWorkerUrl,
  type GeoJSONSource,
  type MapLayerMouseEvent,
  type StyleSpecification,
} from 'maplibre-gl'
import 'maplibre-gl/dist/maplibre-gl.css'
import workerUrl from 'maplibre-gl/dist/maplibre-gl-worker.mjs?url'
import type { Feature, FeatureCollection, LineString, Point } from 'geojson'
import { feature } from 'topojson-client'
import land50 from 'world-atlas/land-50m.json'
import { airportByCode, type GlobeAirport } from '../data/globe'
import { formatDuration } from '../lib/format'
import { supportsWebGL } from '../lib/webgl'
import { NAGAI, PALM, glitter, graticule, greatCircle } from '../map/nagai'
import type { Itinerary } from '../types'
import GlobeFallback from './GlobeFallback'

setWorkerUrl(workerUrl)

type Props = {
  itineraries: Itinerary[]
  selectedId: string | null
  onSelect: (id: string) => void
  originCode: string
  destinationCode: string
}

type RouteProps = { key: string; itineraryId: string; selected: boolean; dashed: boolean }

const topo = land50 as unknown as Parameters<typeof feature>[0] & { objects: { land: never } }
const land = feature(topo, topo.objects.land) as unknown as FeatureCollection

// Height of the stage title and price at the top of the sky; clouds stay below it.
const TITLE_BAND = 110

const EMPTY: FeatureCollection = { type: 'FeatureCollection', features: [] }

// The poster frame around the data: the top band stays sky, the bottom clears the route strip.
const PADDING = { top: 150, bottom: 120, left: 72, right: 72 }

const prefersReducedMotion = () =>
  typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

const style: StyleSpecification = {
  version: 8,
  projection: { type: 'globe' },
  // No atmosphere: the sky is the poster's, drawn behind a transparent canvas.
  sky: { 'atmosphere-blend': 0, 'sky-color': 'rgba(0,0,0,0)', 'horizon-color': 'rgba(0,0,0,0)' },
  sources: {
    land: { type: 'geojson', data: land },
    graticule: { type: 'geojson', data: graticule() },
    glitter: { type: 'geojson', data: glitter() },
    routes: { type: 'geojson', data: EMPTY },
    airports: { type: 'geojson', data: EMPTY },
  },
  layers: [
    { id: 'sea', type: 'background', paint: { 'background-color': NAGAI.sea } },
    {
      id: 'graticule',
      type: 'line',
      source: 'graticule',
      paint: { 'line-color': NAGAI.white, 'line-opacity': 0.1, 'line-width': 0.8 },
    },
    // Sun glitter on the water; the land drawn over it hides the strokes that fall on shore.
    {
      id: 'glitter',
      type: 'line',
      source: 'glitter',
      layout: { 'line-cap': 'round' },
      paint: { 'line-color': '#FFFFFF', 'line-opacity': 0.42, 'line-width': ['interpolate', ['linear'], ['zoom'], 1, 0.8, 4, 1.6] },
    },
    // Pool-water shallows: two flat turquoise bands ringing every coast, widening as you zoom in.
    {
      id: 'shallows-outer',
      type: 'line',
      source: 'land',
      layout: { 'line-join': 'round' },
      paint: { 'line-color': NAGAI.shallow, 'line-width': ['interpolate', ['linear'], ['zoom'], 1, 3, 3, 12, 6, 28] },
    },
    {
      id: 'shallows-inner',
      type: 'line',
      source: 'land',
      layout: { 'line-join': 'round' },
      paint: { 'line-color': NAGAI.shallowLight, 'line-width': ['interpolate', ['linear'], ['zoom'], 1, 1.5, 3, 5, 6, 12] },
    },
    // Hard shadow, never blur: the land again, offset in screen space like a low sun.
    {
      id: 'land-shadow',
      type: 'fill',
      source: 'land',
      paint: { 'fill-color': NAGAI.shadow, 'fill-translate': [9, 11], 'fill-translate-anchor': 'viewport' },
    },
    { id: 'land', type: 'fill', source: 'land', paint: { 'fill-color': NAGAI.sand } },
    {
      id: 'coast',
      type: 'line',
      source: 'land',
      paint: { 'line-color': NAGAI.ink, 'line-opacity': 0.22, 'line-width': 0.7 },
    },
    {
      id: 'route-casing',
      type: 'line',
      source: 'routes',
      layout: { 'line-cap': 'round', 'line-join': 'round', 'line-sort-key': ['case', ['get', 'selected'], 1, 0] },
      paint: {
        'line-color': NAGAI.white,
        'line-width': ['case', ['get', 'selected'], 6.5, 3.5],
        'line-opacity': ['case', ['get', 'selected'], 1, 0.7],
      },
    },
    {
      id: 'route-idle',
      type: 'line',
      source: 'routes',
      filter: ['all', ['!', ['get', 'selected']], ['!', ['get', 'dashed']]],
      layout: { 'line-cap': 'round', 'line-join': 'round' },
      paint: { 'line-color': NAGAI.ink, 'line-width': 1.25, 'line-opacity': 0.6 },
    },
    {
      id: 'route-self-transfer',
      type: 'line',
      source: 'routes',
      filter: ['all', ['!', ['get', 'selected']], ['get', 'dashed']],
      paint: { 'line-color': NAGAI.ink, 'line-width': 1.25, 'line-opacity': 0.6, 'line-dasharray': [5, 3] },
    },
    {
      id: 'route-selected',
      type: 'line',
      source: 'routes',
      filter: ['get', 'selected'],
      layout: { 'line-cap': 'round', 'line-join': 'round' },
      paint: { 'line-color': NAGAI.ink, 'line-width': 2.5 },
    },
    // Wide invisible stroke so a 1 px route is still easy to hit.
    {
      id: 'route-hit',
      type: 'line',
      source: 'routes',
      paint: { 'line-color': '#000', 'line-opacity': 0, 'line-width': 14 },
    },
    {
      id: 'airport-ring',
      type: 'circle',
      source: 'airports',
      paint: {
        'circle-color': ['case', ['get', 'endpoint'], NAGAI.ink, NAGAI.white],
        'circle-radius': ['case', ['get', 'endpoint'], 7, 4.5],
        'circle-pitch-alignment': 'map',
      },
    },
    {
      id: 'airport-core',
      type: 'circle',
      source: 'airports',
      paint: {
        'circle-color': ['case', ['get', 'endpoint'], NAGAI.white, NAGAI.ink],
        'circle-radius': ['case', ['get', 'endpoint'], 2.6, 3],
        'circle-pitch-alignment': 'map',
      },
    },
  ],
}

type Puff = readonly [x: number, y: number, r: number]

// Flat white puffs over a hard flat shade; the base sinks behind the globe's edge.
const CUMULUS_LEFT: Puff[] = [
  [70, 238, 34], [112, 206, 46], [160, 176, 52], [150, 120, 44], [186, 84, 38], [214, 128, 42],
  [250, 182, 46], [298, 214, 36], [330, 240, 26], [128, 250, 40], [210, 252, 44], [270, 254, 36],
]
const CUMULUS_RIGHT: Puff[] = [[40, 132, 26], [78, 104, 36], [104, 64, 30], [128, 98, 34], [160, 124, 26], [96, 140, 30], [150, 146, 24]]

function Cumulus({
  className,
  width,
  height,
  puffs,
  scale,
}: {
  className: string
  width: number
  height: number
  puffs: Puff[]
  scale: number
}) {
  // Cel shading in three flat tones: cool shade low-left, a pale midtone, then the sunlit tops.
  const tone = (fill: string, dx: number, dy: number, scale: number) => (
    <g fill={fill}>
      {puffs.map(([x, y, r]) => (
        <circle key={`${x}-${y}`} cx={x + dx} cy={y + dy} r={r * scale} />
      ))}
    </g>
  )
  return (
    <svg
      className={className}
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      style={{ top: 30 - height, transform: `scale(${scale})` }}
    >
      {tone(NAGAI.cloudShade, -10, 12, 1)}
      {tone(NAGAI.cloudMid, -4, 5, 1)}
      {tone('#FFFFFF', 3, -2, 0.86)}
    </svg>
  )
}

function routeKey(itinerary: Itinerary) {
  return [itinerary.segments[0].from.code, ...itinerary.segments.map((segment) => segment.to.code)].join('-')
}

function labelElement(className: string, code: string, city?: string) {
  const element = document.createElement('div')
  element.className = className
  const strong = document.createElement('strong')
  strong.textContent = code
  element.append(strong)
  if (city) {
    const span = document.createElement('span')
    span.textContent = city
    element.append(span)
  }
  return element
}

/** Where the top edge of the globe sits on screen, from the projected size of a 10° arc. */
function limbY(map: MapLibreMap) {
  const center = map.getCenter()
  const a = map.project(center)
  const b = map.project([center.lng, Math.min(center.lat + 10, 89)])
  const radius = Math.hypot(a.x - b.x, a.y - b.y) / Math.sin((10 * Math.PI) / 180)
  return a.y - radius
}

export default function MapLibreStage({ itineraries, selectedId, onSelect, originCode, destinationCode }: Props) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const mapRef = useRef<MapLibreMap | null>(null)
  const markersRef = useRef<Marker[]>([])
  const onSelectRef = useRef(onSelect)
  const [ready, setReady] = useState(false)
  const [horizon, setHorizon] = useState<number | null>(null)
  const [webgl] = useState(supportsWebGL)

  useEffect(() => {
    onSelectRef.current = onSelect
  }, [onSelect])

  const selected = itineraries.find((itinerary) => itinerary.id === selectedId) ?? itineraries[0] ?? null

  // One line per airport sequence: twelve BER–LIS fares are one nonstop route.
  const { routes, airports } = useMemo(() => {
    const byKey = new Map<string, Feature<LineString, RouteProps>>()
    const codes = new Set<string>()
    const selectedKey = selected ? routeKey(selected) : null

    for (const itinerary of itineraries) {
      const key = routeKey(itinerary)
      const stops = key.split('-').map((code) => airportByCode(code))
      if (stops.some((stop) => !stop)) continue
      const points = stops as GlobeAirport[]
      points.forEach((point) => codes.add(point.code))
      const isSelected = key === selectedKey
      const existing = byKey.get(key)
      if (existing && !isSelected) continue
      const coordinates = points
        .slice(1)
        .flatMap((point, index) => greatCircle([points[index].lng, points[index].lat], [point.lng, point.lat]))
      byKey.set(key, {
        type: 'Feature',
        properties: {
          key,
          itineraryId: isSelected && selected ? selected.id : itinerary.id,
          selected: isSelected,
          dashed: itinerary.risk.includes('Self-transfer'),
        },
        geometry: { type: 'LineString', coordinates },
      })
    }

    const airportFeatures: Feature<Point, { code: string; endpoint: boolean }>[] = [...codes]
      .map((code) => airportByCode(code))
      .filter((airport): airport is GlobeAirport => Boolean(airport))
      .map((airport) => ({
        type: 'Feature',
        properties: {
          code: airport.code,
          endpoint: airport.code === originCode || airport.code === destinationCode,
        },
        geometry: { type: 'Point', coordinates: [airport.lng, airport.lat] },
      }))

    return {
      routes: { type: 'FeatureCollection', features: [...byKey.values()] } as FeatureCollection,
      airports: { type: 'FeatureCollection', features: airportFeatures } as FeatureCollection,
    }
  }, [itineraries, selected, originCode, destinationCode])

  // Create the map once.
  useEffect(() => {
    if (!webgl || !containerRef.current) return
    const map = new MapLibreMap({
      container: containerRef.current,
      style,
      center: [2, 30],
      zoom: 1.6,
      attributionControl: { compact: true, customAttribution: 'Natural Earth' },
      dragRotate: false,
      pitchWithRotate: false,
      touchPitch: false,
      maxPitch: 0,
      canvasContextAttributes: { antialias: true },
    })
    map.touchZoomRotate.disableRotation()
    mapRef.current = map

    const updateHorizon = () => setHorizon(limbY(map))
    map.on('move', updateHorizon)
    map.on('load', () => {
      updateHorizon()
      setReady(true)
    })
    map.on('mouseenter', 'route-hit', () => {
      map.getCanvas().style.cursor = 'pointer'
    })
    map.on('mouseleave', 'route-hit', () => {
      map.getCanvas().style.cursor = ''
    })
    map.on('click', 'route-hit', (event: MapLayerMouseEvent) => {
      const id = event.features?.[0]?.properties?.itineraryId
      if (typeof id === 'string') onSelectRef.current(id)
    })

    return () => {
      markersRef.current.forEach((marker) => marker.remove())
      map.remove()
      mapRef.current = null
    }
  }, [webgl])

  // Push data and labels whenever the results or the selection change.
  useEffect(() => {
    const map = mapRef.current
    if (!map || !ready) return
    ;(map.getSource('routes') as GeoJSONSource).setData(routes)
    ;(map.getSource('airports') as GeoJSONSource).setData(airports)

    markersRef.current.forEach((marker) => marker.remove())
    const markers: Marker[] = []
    for (const item of airports.features) {
      const { code, endpoint } = item.properties as { code: string; endpoint: boolean }
      const airport = airportByCode(code)
      if (!airport) continue
      const element = endpoint
        ? labelElement('nagai-label', code, airport.city)
        : labelElement('nagai-label is-minor', code)
      markers.push(
        new Marker({ element, anchor: 'bottom-left', offset: [8, -6] })
          .setLngLat([airport.lng, airport.lat])
          .addTo(map),
      )
    }

    const selectedRoute = routes.features.find((route) => route.properties?.selected)
    if (selected && selectedRoute) {
      const coordinates = (selectedRoute.geometry as LineString).coordinates
      const middle = coordinates[Math.floor(coordinates.length / 2)]
      const chip = document.createElement('div')
      chip.className = 'nagai-chip'
      chip.textContent = `${selected.segments[0].carrier.split(' ')[0]} · €${selected.total} · ${formatDuration(selected.durationMinutes)}`
      markers.push(new Marker({ element: chip, anchor: 'left', offset: [10, 0] }).setLngLat(middle as [number, number]).addTo(map))
    }
    markersRef.current = markers
  }, [ready, routes, airports, selected])

  // Fit the route, but never so close that the curve of the earth leaves the frame.
  useEffect(() => {
    const map = mapRef.current
    if (!map || !ready) return
    const points = airports.features.map((item) => (item.geometry as Point).coordinates as [number, number])
    if (points.length === 0) return
    const bounds = points.reduce(
      (box, point) => box.extend(point),
      new LngLatBounds(points[0], points[0]),
    )
    const camera = map.cameraForBounds(bounds, { padding: PADDING })
    if (!camera) return

    const { clientWidth: width, clientHeight: height } = map.getContainer()
    const cy = PADDING.top + (height - PADDING.top - PADDING.bottom) / 2
    const maxRadius = 0.95 * Math.hypot(width / 2, cy)
    const lat = LngLat.convert(camera.center ?? map.getCenter()).lat
    // Globe radius in px at zoom z is 512·2^z / (2π·cos φ); solve for the zoom that keeps the limb on screen.
    const limbZoom = Math.log2((maxRadius * 2 * Math.PI * Math.cos((lat * Math.PI) / 180)) / 512)
    const zoom = Math.min(camera.zoom ?? limbZoom, limbZoom)

    map.easeTo({
      center: camera.center,
      zoom,
      padding: PADDING,
      duration: prefersReducedMotion() ? 0 : 700,
    })
  }, [ready, originCode, destinationCode, airports])

  if (!webgl) return <GlobeFallback />

  const showSky = horizon !== null && horizon > -40
  // Clouds grow from the horizon but never up into the route title and price above them.
  const cloudRoom = horizon === null ? 0 : Math.max(0, Math.min(1, (horizon - TITLE_BAND) / 250))

  return (
    <div className="nagai-stage">
      <svg className="nagai-sky" aria-hidden="true" preserveAspectRatio="none">
        <defs>
          <linearGradient id="nagai-sky" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor={NAGAI.skyTop} />
            <stop offset="0.22" stopColor={NAGAI.skyHigh} />
            <stop offset="0.5" stopColor={NAGAI.skyMid} />
            <stop offset="0.78" stopColor={NAGAI.skyPale} />
            <stop offset="1" stopColor={NAGAI.skyLow} />
          </linearGradient>
        </defs>
        <rect width="100%" height="100%" fill="url(#nagai-sky)" />
      </svg>

      {showSky ? (
        <div className="nagai-horizon" style={{ top: horizon ?? 0 }} aria-hidden="true">
          <svg className="nagai-sun" width="96" height="96" viewBox="0 0 96 96">
            <circle cx="48" cy="48" r="40" fill={NAGAI.sun} />
          </svg>
          <Cumulus className="nagai-cumulus nagai-cumulus-left" width={360} height={280} puffs={CUMULUS_LEFT} scale={cloudRoom} />
          <Cumulus className="nagai-cumulus nagai-cumulus-right" width={190} height={170} puffs={CUMULUS_RIGHT} scale={cloudRoom * 0.8} />
        </div>
      ) : null}

      <div className="nagai-map" ref={containerRef} />

      <svg className="nagai-palm" viewBox="0 0 300 800" preserveAspectRatio="xMaxYMax meet" aria-hidden="true">
        <path d={PALM.trunk} fill={NAGAI.palm} />
        <path d={PALM.rings} fill="none" stroke={NAGAI.shallowLight} strokeOpacity="0.35" strokeWidth="1.5" />
        {PALM.fronds.map((frond, index) => (
          <g key={index}>
            <path d={frond.leaves} fill={NAGAI.palm} />
            <path d={frond.stem} fill="none" stroke={NAGAI.palm} strokeWidth="4" strokeLinecap="round" />
          </g>
        ))}
      </svg>
    </div>
  )
}
