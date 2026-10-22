# Globe implementation with react-globe.gl

`react-globe.gl` is the v1 choice for the globe: it gives us airports, great-circle arcs, multi-leg paths, picking, and camera control without building a Three.js scene from scratch. This document maps the design decisions in [16-ui-ux.md](16-ui-ux.md), [17-design-system.md](17-design-system.md) and [18-color.md](18-color.md) to concrete props and data shapes.

**Scope:** the globe is the lobby and the Explore surface. The route decision uses a flat route board, not the globe. See [17-design-system.md](17-design-system.md#globe-vs-flat).

---

## 1. Install

```bash
npm install react-globe.gl three
```

`react-globe.gl` is MIT-licensed and wraps `three-globe` and Three.js. No API key, no tile server, no Mapbox token.

```tsx
import Globe, { GlobeMethods } from "react-globe.gl";
import { useRef } from "react";

const globeRef = useRef<GlobeMethods | undefined>(undefined);

<Globe ref={globeRef} />
```

---

## 2. Land: polygons, not textures

Do not use NASA Blue Marble, night lights, clouds or a starfield. They add 1–4 MB, fight label contrast, and make the globe feel like a screensaver.

Use Natural Earth land polygons on a flat-colour sphere.

```tsx
import * as THREE from "three";
import land from "./data/ne_110m_land.json"; // GeoJSON FeatureCollection

const paperGlobe = new THREE.MeshPhongMaterial({
  color: "#DCE9F5",
  shininess: 0,
});

<Globe
  backgroundColor="transparent"
  globeMaterial={paperGlobe}
  polygonsData={land.features}
  polygonCapColor={() => "#E9EEF4"}
  polygonSideColor={() => "transparent"}
  polygonStrokeColor={() => "#C9D4DF"}
  polygonAltitude={() => 0.005}
  showAtmosphere
  atmosphereColor="#9FB8C9"
  atmosphereAltitude={0.12}
  rendererConfig={{ antialias: false, alpha: true }}
/>
```

- `polygonAltitude={0.005}` avoids z-fighting with the sphere. Larger values extrude the land; keep it flat.
- Natural Earth 110m is enough at world/continent scale; use 50m only if the camera gets close.
- `world-atlas` + `topojson-client` is a convenient npm route to the same public-domain data.
- A dotted `hexPolygonsData` land layer is an optional stylistic alternative, similar to the GitHub globe. It is heavier; use it only if the look is worth the frame budget.
- Atmosphere: a 2px rim at 12% is enough. No glow, no bloom.
- `showGraticules` exists, but it draws a full graticule; if you use it, hide it at route scale.

---

## 3. Airports: `pointsData`

```tsx
type Airport = {
  code: string;       // IATA
  lat: number;
  lng: number;
  traffic: number;    // annual passengers, for sizing
};

const airports: Airport[] = [
  { code: "BER", lat: 52.3667, lng: 13.5033, traffic: 34_000_000 },
  { code: "LIS", lat: 38.7742, lng: -9.1342, traffic: 34_000_000 },
];

<Globe
  pointsData={airports}
  pointLat="lat"
  pointLng="lng"
  pointColor={() => "#0B1B2B"}
  pointRadius={(d) => 0.1 + Math.log10(d.traffic / 1_000_000) * 0.08}
  pointAltitude={0.01}
  pointLabel={(d) => d.code}
  onPointClick={(d) => pickAirport(d)}
  onPointHover={(d) => setHovered(d)}
/>
```

- `pointRadius` is in **angular degrees**, not pixels. Around `0.2–0.4` works at world view; shrink it as the camera descends.
- `pointAltitude` lifts the marker off the surface to avoid z-fighting.
- `pointLabel` returns an HTML tooltip string. It is not accessible and does not collide; the real labels are the HTML overlay in section 6.
- `pointsMerge` is much faster for thousands of airports, but **disables per-point hover**. Use it for the static background layer and keep a separate small `pointsData` layer for the top 100–300 interactive airports.
- Filter to airports with scheduled service. Do not draw 7,000 points at once.

---

## 4. Direct flights: `arcsData`

Arcs are great-circle lines between two coordinates. Use them for nonstop routes.

```tsx
type Route = {
  id: string;
  from: Airport;
  to: Airport;
  color: string;
};

const routes: Route[] = [
  { id: "ber-lis", from: airports[0], to: airports[1], color: "#5F7488" },
];

<Globe
  arcsData={routes}
  arcStartLat={(d) => d.from.lat}
  arcStartLng={(d) => d.from.lng}
  arcEndLat={(d) => d.to.lat}
  arcEndLng={(d) => d.to.lng}
  arcColor={(d) => d.color}
  arcStroke={(d) => (d.id === selectedId ? 2.5 : d.id === hoveredId ? 1.5 : 1)}
  arcAltitudeAutoScale={0.3}
  arcsTransitionDuration={0}
  onArcClick={(d) => selectRoute(d)}
  onArcHover={(d) => previewRoute(d)}
/>
```

- `arcAltitudeAutoScale` controls how high arcs rise with distance. `0.25–0.35` is a good default.
- To raise only the selected arc, render a second `arcsData` layer containing just that route with a slightly higher `arcAltitude`. Do not set `arcAltitude` to `0` for the rest; that flattens every arc.
- `arcStroke` is width. Base 1, hover 1.5, selected 2.5 plus a casing. If the library only supports one stroke, draw a second slightly wider arc underneath in the surface colour to fake the casing.
- `arcsTransitionDuration={0}` disables the draw-on animation. Fade new arcs in with opacity over ~200ms after the camera settles.
- Do not set `arcDashAnimateTime`: a moving dash implies live tracking, which a fare search does not have.
- `arcColor` can return a gradient array. Do not; use one colour per state.

---

## 5. Connecting itineraries: `pathsData`

A one-stop itinerary is not a direct arc. Use a path through the connection airport.

```tsx
type Itinerary = {
  id: string;
  color: string;
  points: Array<{ lat: number; lng: number }>;
};

const itineraries: Itinerary[] = [
  {
    id: "ber-ams-lis",
    color: "#0B6E99",
    points: [
      { lat: 52.3667, lng: 13.5033 }, // BER
      { lat: 52.3105, lng: 4.7683 },  // AMS
      { lat: 38.7742, lng: -9.1342 }, // LIS
    ],
  },
];

<Globe
  pathsData={itineraries}
  pathPoints="points"
  pathPointLat="lat"
  pathPointLng="lng"
  pathPointAlt={0.01}
  pathColor="color"
  pathStroke={(d) => (d.id === selectedId ? 2.5 : 1.5)}
  pathResolution={2}
  pathTransitionDuration={0}
  onPathClick={(d) => selectItinerary(d)}
  onPathHover={(d) => previewItinerary(d)}
/>
```

- `pathsData` accepts arbitrary points, so the same layer works for real ADS-B tracks later.
- Add a connection marker at the intermediate point: a small `pointsData` entry or a `ringsData` ring.
- For a direct route, prefer arcs; paths are heavier and unnecessary.

---

## 6. Labels: HTML overlay, not 3D text

`labelsData` renders 3D text meshes. It has no collision detection, no screen-space sizing and no accessibility. Do not use it for the main labels.

Use `getScreenCoords` and a DOM overlay:

```tsx
const [labels, setLabels] = useState<Array<Airport & { x: number; y: number }>>([]);

const updateLabels = () => {
  const globe = globeRef.current;
  if (!globe) return;
  const next = airports.map((a) => ({
    ...a,
    ...globe.getScreenCoords(a.lat, a.lng, 0.01),
  }));
  setLabels(next);
};

<Globe
  ref={globeRef}
  onGlobeReady={updateLabels}
  onZoom={updateLabels}
/>
```

Then render absolutely positioned labels in a sibling `<div>` over the canvas. Rank them (pinned first, then traffic, then deal strength), collision-test the boxes, and hide the ones that do not fit. Budgets: 12 labels at world view, 24 at continent, 40 at region, endpoints plus three connections at route scale.

- `getScreenCoords` can return coordinates for airports on the far side of the globe. Test the surface normal against the camera direction, or compare against the horizon, and fade those labels out.
- Labels are `pointer-events: none` and `aria-hidden="true"`; the list carries the accessible information.
- Use IATA only at world view (`BER`), `BER Berlin` at region view, and `BER Berlin / 10:40` at route view.
- A 2px halo in the surface colour plus a 1px shadow is enough.

`htmlElementsData` is a newer alternative for anchoring HTML elements directly to coordinates, but you still have to manage collision and priority. One overlay is easier to reason about.

---

## 7. State wiring

```ts
type GlobeState = {
  hoveredId: string | null;   // pointer preview
  focusedId: string | null;   // keyboard preview
  selectedId: string | null;  // persistent, in URL
};
```

- `previewId = hoveredId ?? focusedId` styles the arc and card but never moves the camera.
- `selectedId` calls `ensureVisible` only if the route is outside the safe rectangle; otherwise the camera stays still.
- `onPointClick`: if origin is empty, set origin; otherwise set destination.
- `onArcClick` / `onPathClick`: set `selectedId`, scroll the matching card into view.
- `onGlobeClick`: clear selection.
- `onZoom`: update label positions (throttle to ~15fps) and record `camera.manualAt` for auto-fit suppression.

Camera:

```ts
globeRef.current?.pointOfView({ lat, lng, altitude }, 350 + 450 * (angle / Math.PI));
```

Clamp to 350–800ms, interrupt on pointer input, and jump instantly under `prefers-reduced-motion`.

---

## 8. Performance budgets

| Tier | Arcs | Points | Labels | Pixel ratio | Effects |
|---|---:|---:|---:|---:|---|
| A desktop | 300 | 1,500 | 40 | 2.0 | atmosphere, graticule |
| B laptop | 180 | 900 | 32 | 1.5 | atmosphere |
| C mobile | 100 | 500 | 16 | 1.5 | no atmosphere, no graticule |
| D fallback | 0 | 0 | 0 | 1.0 | static poster + list |

- Lower `arcCurveResolution` (64 → 32) and `pathResolution` (2 → 1) on mobile.
- Set `rendererConfig={{ antialias: false, alpha: true }}`.
- Use `pointsMerge` for the static airport layer; keep the interactive layer small.
- Call `pauseAnimation()` when the tab is hidden or the mobile sheet is full; `resumeAnimation()` when visible.
- Show a static SVG/canvas poster immediately and crossfade to WebGL when the first frame is ready.
- Handle `webglcontextlost`: freeze, show the poster and a “Reload globe” action.

---

## 9. Design guardrails

- Deduplicate arcs by airport pair. Twelve BER–LIS itineraries are one arc with a “12 options” badge.
- A one-stop itinerary is a path with a connection dot, never a direct arc.
- No auto-rotation. No draw-on for every search. No pulsing rings. No plane animation.
- Selected arc is theme-aware: ink on light, white on dark, with a 2px casing.
- Dimming: 60% default, 35% during hover, 15% when another itinerary is selected. Never 8%.
- Cap arcs by zoom: 40 / 120 / 300. Halve on mobile.
- No colour-only encoding: every price pill prints the price and percentile.
- Do not use airline logos; they fight the data and require licensing.

---

## 10. Data and assets

| Asset | Source | Licence |
|---|---|---|
| Airports, coordinates, scheduled service | [OurAirports](https://ourairports.com/data/) | Public domain |
| Land polygons | [Natural Earth](https://www.naturalearthdata.com/) | Public domain |
| TopoJSON convenience | `world-atlas` + `topojson-client` | ISC / public domain |
| Airport codes / traffic | OurAirports, airport reports | Public domain / company reports |
| Great-circle geometry | computed from coordinates | — |

Do not bundle a route network from OpenFlights for production; it is a 2014 snapshot. Use live search or a current schedule source for Explore. See [14-cold-start-and-coverage.md](14-cold-start-and-coverage.md).

---

## 11. Test cases

- One airport, one arc, 300 arcs, 1,500 points.
- Antimeridian: AKL–SCL, NRT–HNL, LAX–SYD, BER–LIS.
- A one-stop itinerary: BER–AMS–LIS, with the connection dot.
- Selected route near the panel edge: `ensureVisible` should not move the camera if it is already safe.
- Label collision over Europe: 12 labels at world view, 40 at region.
- WebGL disabled or context lost: poster + list.
- Keyboard: focus a card, preview its arc, select with Enter, clear with Esc.
- Reduced motion: no camera flight, no arc fade, no selection ring.
