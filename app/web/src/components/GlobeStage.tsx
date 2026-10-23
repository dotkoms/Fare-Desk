import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import Globe, { type GlobeMethods } from 'react-globe.gl'
import { MeshPhongMaterial } from 'three'
import { airportByCode, globeAirports, landFeatures, type GlobeAirport } from '../data/globe'
import GlobeFallback from './GlobeFallback'
import { supportsWebGL } from '../lib/webgl'
import type { Itinerary } from '../types'

type Props = {
  itineraries: Itinerary[]
  selectedId: string | null
  onSelect: (id: string) => void
  originCode: string
  destinationCode: string
}

type DirectArc = {
  id: string
  from: GlobeAirport
  to: GlobeAirport
  itineraryId: string
  selected: boolean
  dimmed: boolean
}

type PathArc = {
  id: string
  points: GlobeAirport[]
  selected: boolean
  dimmed: boolean
}

const asArc = (obj: object) => obj as unknown as DirectArc
const asPath = (obj: object) => obj as unknown as PathArc

const COLORS = {
  ocean: '#C2DBEC',
  land: '#EFF3F7',
  coast: '#9FB6C8',
  atmosphere: '#9FB8C9',
  arc: '#5F7488',
  dim: '#B9C8D4',
  selected: '#0B1B2B',
  point: '#5F7488',
  label: '#0B1B2B',
  labelBg: 'rgba(255, 255, 255, 0.88)',
} as const

type Label = {
  code: string
  city: string
  x: number
  y: number
}

export default function GlobeStage({
  itineraries,
  selectedId,
  onSelect,
  originCode,
  destinationCode,
}: Props) {
  const globeRef = useRef<GlobeMethods | undefined>(undefined)
  const wrapRef = useRef<HTMLDivElement | null>(null)
  const [size, setSize] = useState({ width: 640, height: 380 })
  const [ready, setReady] = useState(false)
  const [hovered, setHovered] = useState<string | null>(null)
  const [labels, setLabels] = useState<Label[]>([])
  const [webgl] = useState(supportsWebGL)

  const selected = itineraries.find((itinerary) => itinerary.id === selectedId) ?? itineraries[0] ?? null

  const colors = COLORS

  const globeMaterial = useMemo(
    () => new MeshPhongMaterial({ color: COLORS.ocean, shininess: 0 }),
    [],
  )

  const { arcs, paths } = useMemo(() => {
    const direct = new Map<string, DirectArc>()
    const routePaths: PathArc[] = []

    for (const itinerary of itineraries) {
      if (itinerary.segments.length === 1) {
        const segment = itinerary.segments[0]
        const from = airportByCode(segment.from.code)
        const to = airportByCode(segment.to.code)
        if (!from || !to) continue
        const key = `${from.code}-${to.code}`
        if (!direct.has(key)) {
          direct.set(key, {
            id: key,
            from,
            to,
            itineraryId: itinerary.id,
            selected: false,
            dimmed: false,
          })
        }
      } else {
        const points = [itinerary.segments[0].from, ...itinerary.segments.map((segment) => segment.to)]
          .map((airport) => airportByCode(airport.code))
          .filter((airport): airport is GlobeAirport => Boolean(airport))
        if (points.length > 1) {
          routePaths.push({
            id: itinerary.id,
            points,
            selected: itinerary.id === selected?.id,
            dimmed: Boolean(selected) && itinerary.id !== selected?.id,
          })
        }
      }
    }

    const selectedDirect =
      selected && selected.segments.length === 1
        ? `${selected.segments[0].from.code}-${selected.segments[0].to.code}`
        : null

    const directArcs = [...direct.values()].map((arc) => ({
      ...arc,
      selected: arc.id === selectedDirect,
      dimmed: Boolean(selected) && arc.id !== selectedDirect,
      itineraryId: arc.id === selectedDirect && selected ? selected.id : arc.itineraryId,
    }))

    const pathArcs = routePaths.map((path) => ({
      ...path,
      selected: path.id === selected?.id,
      dimmed: Boolean(selected) && path.id !== selected?.id,
    }))

    return { arcs: directArcs, paths: pathArcs }
  }, [itineraries, selected])

  useEffect(() => {
    const element = wrapRef.current
    if (!element) return
    const observer = new ResizeObserver(([entry]) => {
      setSize({ width: entry.contentRect.width, height: entry.contentRect.height })
    })
    observer.observe(element)
    return () => observer.disconnect()
  }, [])

  const updateLabels = useCallback(() => {
    const globe = globeRef.current
    if (!globe) return
    const codes = new Set([originCode, destinationCode])
    if (hovered) codes.add(hovered)

    const next = globeAirports
      .filter((airport) => codes.has(airport.code))
      .map((airport) => {
        const point = globe.getScreenCoords(airport.lat, airport.lng, 0.01)
        if (!point) return null
        return { code: airport.code, city: airport.city, x: point.x, y: point.y }
      })
      .filter((label): label is Label => Boolean(label))

    setLabels(next)
  }, [originCode, destinationCode, hovered])

  useEffect(() => {
    const globe = globeRef.current
    if (!ready || !globe) return
    const controls = globe.controls()
    updateLabels()
    controls.addEventListener('change', updateLabels)
    return () => controls.removeEventListener('change', updateLabels)
    // size is a dependency so labels are recomputed after the wrap is measured
  }, [ready, updateLabels, size])

  useEffect(() => {
    if (!ready) return
    const from = airportByCode(originCode)
    const to = airportByCode(destinationCode)
    if (!from || !to) return
    const lat = (from.lat + to.lat) / 2
    let delta = to.lng - from.lng
    if (delta > 180) delta -= 360
    if (delta < -180) delta += 360
    const lng = from.lng + delta / 2
    globeRef.current?.pointOfView({ lat, lng, altitude: 2.4 }, 600)
  }, [ready, originCode, destinationCode])

  if (!webgl) {
    return <GlobeFallback />
  }

  return (
    <div className="globe-wrap" ref={wrapRef}>
      <Globe
        ref={globeRef}
        width={size.width}
        height={size.height}
        backgroundColor="rgba(0, 0, 0, 0)"
        showGlobe={true}
        animateIn={false}
        globeMaterial={globeMaterial}
        showAtmosphere
        atmosphereColor={colors.atmosphere}
        atmosphereAltitude={0.12}
        polygonsData={landFeatures}
        polygonCapColor={() => colors.land}
        polygonSideColor={() => colors.ocean}
        polygonStrokeColor={() => colors.coast}
        polygonAltitude={() => 0.005}
        polygonsTransitionDuration={0}
        pointsData={globeAirports}
        pointLat="lat"
        pointLng="lng"
        pointAltitude={0.01}
        pointColor={(airport) => {
          const code = (airport as GlobeAirport).code
          return code === originCode || code === destinationCode ? colors.selected : colors.point
        }}
        pointRadius={(airport) => {
          const item = airport as GlobeAirport
          return item.code === originCode || item.code === destinationCode
            ? 0.26
            : 0.08 + Math.log10(item.traffic) * 0.04
        }}
        onPointHover={(airport) => setHovered((airport as GlobeAirport | null)?.code ?? null)}
        arcsData={arcs}
        arcStartLat={(obj: object) => asArc(obj).from.lat}
        arcStartLng={(obj: object) => asArc(obj).from.lng}
        arcEndLat={(obj: object) => asArc(obj).to.lat}
        arcEndLng={(obj: object) => asArc(obj).to.lng}
        arcColor={(obj: object) => {
          const item = asArc(obj)
          return item.selected ? colors.selected : item.dimmed ? colors.dim : colors.arc
        }}
        arcStroke={(obj: object) => {
          const item = asArc(obj)
          return item.selected ? 2.5 : item.dimmed ? 0.9 : 1
        }}
        arcAltitudeAutoScale={0.3}
        arcsTransitionDuration={0}
        onArcClick={(obj: object) => onSelect(asArc(obj).itineraryId)}
        pathsData={paths}
        pathPoints={(obj: object) => asPath(obj).points}
        pathPointLat="lat"
        pathPointLng="lng"
        pathPointAlt={0.01}
        pathColor={(obj: object) => {
          const item = asPath(obj)
          return item.selected ? colors.selected : item.dimmed ? colors.dim : colors.arc
        }}
        pathStroke={(obj: object) => {
          const item = asPath(obj)
          return item.selected ? 2.5 : item.dimmed ? 0.9 : 1.5
        }}
        pathResolution={2}
        pathTransitionDuration={0}
        onPathClick={(obj: object) => onSelect(asPath(obj).id)}
        onGlobeReady={() => setReady(true)}
        rendererConfig={{ antialias: true, alpha: true }}
      />

      <div className="globe-labels" aria-hidden="true">
        {labels.map((label) => (
          <div
            key={label.code}
            className="globe-label"
            style={{ left: label.x, top: label.y, color: colors.label, background: colors.labelBg }}
          >
            <strong>{label.code}</strong>
            <span>{label.city}</span>
          </div>
        ))}
      </div>
    </div>
  )
}
