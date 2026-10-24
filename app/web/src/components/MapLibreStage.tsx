import { useEffect, useMemo, useRef, useState } from 'react'
import {
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
import { formatDuration, formatMoney } from '../lib/format'
import { supportsWebGL } from '../lib/webgl'
import { NAGAI, PALM, glitter, graticule, greatCircle } from '../map/nagai'
import type { Airport, Itinerary } from '../types'
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

// Natural Earth 50m land, public domain, already cut at the antimeridian (world-atlas rings are
// spherical, and a ring that wraps the dateline inverts the fill tile by tile). Served as a
// static file so it stays out of the bundle; PMTiles replaces it when zoom detail matters.
const LAND_URL = `${import.meta.env.BASE_URL}data/ne_50m_land.geojson`

// Height of the stage title and price at the top of the sky; clouds stay below it.
const TITLE_BAND = 110

const EMPTY: FeatureCollection = { type: 'FeatureCollection', features: [] }

// The poster frame around the data: the top band stays sky, the bottom clears the route strip.
// On desktop the console and the detail drawer float over the map, so the route fits between them.
function framePadding(width: number) {
  return width >= 1100
    ? { top: 170, bottom: 120, left: 460, right: 480 }
    : { top: 150, bottom: 120, left: 72, right: 72 }
}

// A gentle tilt brings the earth's horizon into the frame, the way a Nagai sky meets the sea.
const OVERVIEW_PITCH = 38
// Past this the horizon leaves the frame and the scene stops reading as a globe under a sky.
const MAX_OVERVIEW_ZOOM = 3.3

const prefersReducedMotion = () =>
  typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

const style: StyleSpecification = {
  version: 8,
  projection: { type: 'globe' },
  // No atmosphere: the sky is the poster's, drawn behind a transparent canvas.
  sky: { 'atmosphere-blend': 0, 'sky-color': 'rgba(0,0,0,0)', 'horizon-color': 'rgba(0,0,0,0)' },
  sources: {
    land: { type: 'geojson', data: LAND_URL },
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
      paint: {
        'line-color': '#FFFFFF',
        'line-opacity': 0.42,
        'line-width': ['interpolate', ['linear'], ['zoom'], 1, 0.8, 4, 1.6],
      },
    },
    // Pool-water shallows: two flat turquoise bands ringing every coast, widening as you zoom in.
    {
      id: 'shallows-outer',
      type: 'line',
      source: 'land',
      layout: { 'line-join': 'round' },
      paint: { 'line-color': NAGAI.shallow, 'line-width': ['interpolate', ['linear'], ['zoom'], 1, 2, 3, 6, 6, 12] },
    },
    {
      id: 'shallows-inner',
      type: 'line',
      source: 'land',
      layout: { 'line-join': 'round' },
      paint: {
        'line-color': NAGAI.shallowLight,
        'line-width': ['interpolate', ['linear'], ['zoom'], 1, 1, 3, 2.5, 6, 5],
      },
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
  [70, 238, 34],
  [112, 206, 46],
  [160, 176, 52],
  [150, 120, 44],
  [186, 84, 38],
  [214, 128, 42],
  [250, 182, 46],
  [298, 214, 36],
  [330, 240, 26],
  [128, 250, 40],
  [210, 252, 44],
  [270, 254, 36],
]
const CUMULUS_RIGHT: Puff[] = [
  [40, 132, 26],
  [78, 104, 36],
  [104, 64, 30],
  [128, 98, 34],
  [160, 124, 26],
  [96, 140, 30],
  [150, 146, 24],
]

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

/** Where the globe's horizon sits on screen: the highest projected point due north of the centre. */
function limbY(map: MapLibreMap) {
  const center = map.getCenter()
  const lat = (center.lat * Math.PI) / 180
  const lng = (center.lng * Math.PI) / 180
  let highest = Infinity
  for (let degrees = 2; degrees <= 90; degrees += 2) {
    const d = (degrees * Math.PI) / 180
    const lat2 = Math.asin(Math.sin(lat) * Math.cos(d) + Math.cos(lat) * Math.sin(d))
    const lng2 = lng + Math.atan2(0, Math.cos(d) - Math.sin(lat) * Math.sin(lat2))
    const y = map.project([(lng2 * 180) / Math.PI, (lat2 * 180) / Math.PI]).y
    if (y < highest) highest = y
  }
  return highest
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
  const { routes, airports, places } = useMemo(() => {
    // Coordinates come with each segment from the data plane; no airport list is bundled.
    const places = new Map<string, Airport>()
    for (const itinerary of itineraries) {
      for (const segment of itinerary.segments) {
        places.set(segment.from.code, segment.from)
        places.set(segment.to.code, segment.to)
      }
    }
    const byKey = new Map<string, Feature<LineString, RouteProps>>()
    const codes = new Set<string>()
    const selectedKey = selected ? routeKey(selected) : null

    for (const itinerary of itineraries) {
      const key = routeKey(itinerary)
      const points = key.split('-').map((code) => places.get(code) as Airport)
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
      .map((code) => places.get(code) as Airport)
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
      places,
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
      maxPitch: 50,
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
    // The eastern endpoint's label sits left of its dot, so it reads toward the route
    // and stays clear of the detail drawer on the right.
    const endpointLngs = [originCode, destinationCode].map((code) => places.get(code)?.lng ?? 0)
    const easternmost = Math.max(...endpointLngs)
    for (const item of airports.features) {
      const { code, endpoint } = item.properties as { code: string; endpoint: boolean }
      const airport = places.get(code)
      if (!airport) continue
      const element = endpoint
        ? labelElement('nagai-label', code, airport.city)
        : labelElement('nagai-label is-minor', code)
      markers.push(
        new Marker(
          endpoint && airport.lng === easternmost
            ? { element, anchor: 'bottom-right', offset: [-8, -6] }
            : { element, anchor: 'bottom-left', offset: [8, -6] },
        )
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
      chip.textContent = `${selected.segments[0].carrier.split(' ')[0]} · ${formatMoney(selected.total, selected.currency)} · ${formatDuration(selected.durationMinutes)}`
      markers.push(
        new Marker({ element: chip, anchor: 'left', offset: [10, 0] }).setLngLat(middle as [number, number]).addTo(map),
      )
    }
    markersRef.current = markers
  }, [ready, routes, airports, places, selected, originCode, destinationCode])

  // Fit the route between the floating panels, tilted so the horizon and sky stay in view.
  useEffect(() => {
    const map = mapRef.current
    if (!map || !ready) return
    const points = airports.features.map((item) => (item.geometry as Point).coordinates as [number, number])
    if (points.length === 0) return
    const bounds = points.reduce((box, point) => box.extend(point), new LngLatBounds(points[0], points[0]))
    const padding = framePadding(map.getContainer().clientWidth)
    const camera = map.cameraForBounds(bounds, { padding })
    if (!camera) return

    map.easeTo({
      center: camera.center,
      zoom: Math.min(camera.zoom ?? 3, MAX_OVERVIEW_ZOOM),
      pitch: OVERVIEW_PITCH,
      padding,
      duration: prefersReducedMotion() ? 0 : 700,
    })
  }, [ready, originCode, destinationCode, airports])

  if (!webgl) return <GlobeFallback />

  const showSky = horizon !== null && horizon > -40
  // Clouds grow from the horizon but never up into the route title and price above them.
  const cloudRoom = horizon === null ? 0 : Math.max(0, Math.min(1, (horizon - TITLE_BAND) / 160))

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
          <Cumulus
            className="nagai-cumulus nagai-cumulus-left"
            width={360}
            height={280}
            puffs={CUMULUS_LEFT}
            scale={cloudRoom}
          />
          <Cumulus
            className="nagai-cumulus nagai-cumulus-right"
            width={190}
            height={170}
            puffs={CUMULUS_RIGHT}
            scale={cloudRoom * 0.8}
          />
        </div>
      ) : null}

      <div className="nagai-map" ref={containerRef} />

      <svg className="nagai-palm" width="420" height="300" viewBox="0 0 420 300" aria-hidden="true">
        {PALM.map((frond, index) => (
          <g key={index}>
            <path d={frond.leaves} fill={frond.back ? NAGAI.palmBack : NAGAI.palm} />
            <path
              d={frond.stem}
              fill="none"
              stroke={frond.back ? NAGAI.palmBack : NAGAI.palm}
              strokeWidth="4"
              strokeLinecap="round"
            />
          </g>
        ))}
      </svg>
    </div>
  )
}
