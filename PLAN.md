# Fleet Atlas — interactive flight and train finder

A 3D globe where every airport and major rail station is a node, every route is a line, and the left panel is the search console. Pick two places and a date window; the globe draws the candidate itineraries, then highlights the one you select.

**Status:** plan only. No implementation yet.

---

## 1. The product in one paragraph

A traveller opens Fleet Atlas, types “Berlin” and “Lisbon,” picks a week in March, and chooses air, rail or both. The globe spins to Europe and draws the candidate routes as arcs. The left panel lists itineraries with total price, door-to-door time, transfers and baggage. Hovering a result lifts its arc above the globe; clicking it flies the camera to the route, dims everything else, and opens the segment detail. Rail routes follow actual track geometry; flights follow great-circle arcs. The same search can mix modes, with airport-to-station transfers shown as short connector lines.

---

## 2. Goals and non-goals

### Goals

- Make route geography legible: you can see *why* a split, a positioning leg or a mixed-mode trip is cheaper.
- Search flights and trains from one query, with honest total cost.
- Highlight one itinerary clearly without losing the rest of the result set.
- Feel fast: results visible in under two seconds with cached data, globe interactions at 60fps.
- Work on a phone, where the panel becomes a bottom sheet.
- Keep the data provenance visible: “verified 4 minutes ago” vs “indicative.”

### Non-goals for v1

- Booking inside the app. Every result deeplinks to the operator.
- Full global rail coverage. Start with long-distance corridors in Europe, the UK, Japan and the US.
- Live seat maps or fare-class inventory.
- User accounts, saved trips, collaboration.
- AR/VR or a fully modelled Earth with terrain.
- A consumer fare feed. The backend uses the sources in [docs/02-data-sources.md](docs/02-data-sources.md) and [docs/03-public-apis-vs-scraping.md](docs/03-public-apis-vs-scraping.md).

---

## 3. The experience

### Wireframe

```
┌──────────────────────────┬─────────────────────────────────────────────┐
│ FLEET ATLAS              │                                             │
│                          │                                             │
│  [One way] [Return]      │              3D GLOBE                       │
│                          │                                             │
│  From   BER  Berlin      │        ●───────╮                            │
│  To     LIS  Lisbon      │       /         ╲                           │
│  ⇄                       │      ●           ●   arcs                   │
│                          │        airport codes, hover labels          │
│  Depart  Mar 1           │                                             │
│  Return  Mar 8           │                                             │
│                          │                                             │
│  1 adult   Economy       │                                             │
│  [x] Air  [x] Rail       │                                             │
│  Bags  1 checked         │                                             │
│                          │                                             │
│  [        Search       ] │                                             │
│                          │                                             │
│  12 results    Best ▾    │                                             │
│  ┌────────────────────┐  │                                             │
│  │ BER → LIS   6h 20  │  │                                             │
│  │ €89 · 1 stop · bag │  │                                             │
│  └────────────────────┘  │                                             │
│  ┌────────────────────┐  │                                             │
│  │ BER → LIS   7h 05  │  │                                             │
│  │ €74 · rail · 2 chg │  │                                             │
│  └────────────────────┘  │                                             │
└──────────────────────────┴─────────────────────────────────────────────┘
```

### Primary flow

1. **Land.** Globe slowly auto-rotates. No search yet. A hint reads “Where from?”
2. **Pick origin.** The combobox searches airports and stations by city, code or name. On selection the globe flies to the place, drops a ring, and draws the top 20 routes as faint arcs.
3. **Pick destination.** The arcs narrow to the pair. The panel shows “Set dates to search.”
4. **Set dates and filters.** Date range picker, passengers, cabin, bags, mode toggles, max stops, max duration.
5. **Search.** Panel shows skeletons. Existing arcs fade to 20%. Results arrive; arcs redraw at low opacity with the top result slightly brighter.
6. **Hover.** A result card lifts its arc, raises its altitude, and shows a tooltip with the segment endpoints and local times.
7. **Select.** Click a card or an arc. The camera frames the itinerary, the selected segments animate, all other arcs drop to 8% opacity, and the detail drawer opens.
8. **Compare.** Pin up to three results. The globe shows three colour-coded itineraries; the panel shows a comparison table.
9. **Book.** The detail drawer has one primary action: “Open on [operator].” It opens the verified deep link in a new tab.

### Empty, loading and error states

- **No origin:** globe idle, panel shows recent and popular searches.
- **Loading:** skeleton cards, a subtle progress line, globe does not block.
- **No results:** explain which filter removed the last result and offer to relax it.
- **Source failure:** partial results are fine; label which mode failed and when.
- **Expired price:** mark the card “price changed,” offer re-search, never show the old number as live.

---

## 4. The 3D scene

### Rendering choice

**Recommended for v1: `react-globe.gl` (Three.js).**

It is purpose-built for this interaction: arcs, points, labels, paths, rings, hover and click callbacks, camera flight, and animated dashes. It gets to a cinematic globe faster than assembling a renderer.

**Alternative if map fidelity becomes the priority:** MapLibre GL JS globe projection plus deck.gl layers. Better basemap, labels and terrain; more work to make arcs and picking behave correctly on a globe.

**Do not** start with a custom WebGL globe. The scene is the product’s face; build it on a library that already solved great-circle arcs and camera control.

### Scene composition

| Layer | Content | Notes |
|---|---|---|
| Globe | NASA Blue Marble texture, bump map, night lights blended at the terminator | 4k texture for desktop, 2k for mobile |
| Atmosphere | Rim glow and a thin haze | Subtle; no bloom on the whole scene |
| Stars | Static background texture or a low-count particle field | No animated starfield; it distracts from the routes |
| Airports | Points sized by scheduled departures; label with IATA at high zoom | Cap labels to the top 40 in view |
| Stations | Smaller amber points; label city, not code | Rail stations use city names |
| Flight arcs | Great-circle arcs, altitude scaled by distance | Cap at 300 visible arcs |
| Rail paths | Polyline along track geometry, lifted slightly above the surface | Fallback to a ground-hugging arc where geometry is missing |
| Connectors | Short lines between an airport and a station in the same city | 2–3 km visual gap, distinct dash pattern |
| Selection rings | Pulsing rings at origin, destination and transfer points | Stop pulsing after 3 cycles |
| Tooltip | HTML overlay following the pointer | Code, city, local time, price if known |

### Arc behaviour

- **Altitude:** `0.05 + 0.25 × (distance / maxDistance)`, clamped. Long-haul arcs sit higher; short hops stay near the surface.
- **Resolution:** 64 points for display, 128 for the selected itinerary.
- **Colour:** air cyan `#38BDF8`, rail amber `#F59E0B`, mixed itinerary uses each segment’s mode colour with a shared outline.
- **Selected:** white `#F8FAFC`, thicker stroke, altitude +0.08, drawn over everything.
- **Dimmed:** 8% opacity, no animation.
- **Animation:** draw-on over 600ms when results arrive; a slow dash along the selected route. Respect `prefers-reduced-motion`.

### Camera

- On origin: `pointOfView({lat, lng, altitude: 1.8}, 900)`.
- On search: frame the bounding box of the top results, with padding for the 380px panel.
- On select: frame the selected itinerary; for very long routes, centre on the midpoint and pull back to fit.
- Anti-meridian routes: split the arc at the date line for camera bounds; never let the camera take the long way around.
- Idle: auto-rotate at 0.3 rpm, disabled after the first interaction and by reduced-motion.

### Performance budget

- 500 arcs and 1,500 points at 60fps on an M1 MacBook Air and a mid-range Android.
- Initial JS under 1.5 MB gzipped; globe texture lazy-loaded after the first paint.
- Label collision solved by capping labels, not by rendering all and hiding.
- Pause the render loop when the tab is hidden and when the globe is idle.

---

## 5. The left panel

### Structure

1. **Brand and trip type** — one-way, return, multi-city (multi-city disabled in v1).
2. **Places** — origin and destination comboboxes with swap.
3. **Dates** — range picker with flexible ±1/±3 days toggle.
4. **Travellers and cabin** — adults, children, cabin, bags.
5. **Modes** — air, rail, both. Rail disabled outside covered markets.
6. **Filters** — nonstop only, max duration, departure window, total price ceiling.
7. **Search button** — sticky at the bottom of the form.
8. **Results header** — count, sort (best, cheapest, fastest, earliest), filter chips.
9. **Result cards** — virtualised list.
10. **Detail drawer** — segment timeline, fare rules, baggage, booking link.
11. **Compare tray** — appears when one or more results are pinned.

### Result card

```
BER → LIS                        €89
06:40 → 09:00 · 6h 20 · 1 stop
Air · BER–MAD–LIS · 1h 05 layover
Includes 1 checked bag · verified 4 min ago
```

Required fields: modes, departure and arrival in local time, total duration, transfers, total price with bags, verification age, and a risk badge when the itinerary is self-transfer or OTA-only.

### Filters that matter

- **Total price**, not fare.
- **Door-to-door hours**, including airport/station transfer.
- **Bag inclusion** — a toggle for “carry-on only” changes the ranking substantially.
- **Overnight** — exclude or prefer.
- **Verified only** — hide indicative prices.

### Mobile

Panel becomes a bottom sheet with three snap points: collapsed (search summary), half (results), full (filters and detail). The globe remains full-bleed behind it. Search and result selection must work one-handed.

---

## 6. Interaction state

```ts
type AppState = {
  query: {
    origin: Place | null;
    destination: Place | null;
    departDate: string | null;
    returnDate: string | null;
    flexibility: 0 | 1 | 3;
    passengers: number;
    cabin: 'economy' | 'premium' | 'business';
    bags: { carryOn: number; checked: number };
    modes: Array<'air' | 'rail'>;
  };
  results: {
    status: 'idle' | 'loading' | 'ready' | 'error';
    itineraries: Itinerary[];
    error?: string;
  };
  selection: {
    hoveredId: string | null;
    selectedId: string | null;
    comparedIds: string[];
  };
  camera: {
    target: { lat: number; lng: number; altitude: number } | null;
    autoRotate: boolean;
  };
};
```

Rules:

- `hoveredId` never changes the camera.
- `selectedId` always frames the itinerary and dims the rest.
- Clicking empty globe clears `selectedId` and returns the camera to the results bounds.
- Changing any query field invalidates results but keeps the last selection visible at 30% until the new search resolves.
- URL state: `?from=BER&to=LIS&depart=2027-03-01&return=2027-03-08&modes=air,rail`. Shareable and reload-safe.

---

## 7. Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ web/ — React + TypeScript + Vite                             │
│                                                              │
│  SearchPanel   GlobeScene   ResultList   DetailDrawer        │
│       │            │            │             │              │
│       └────────────┴────────────┴─────────────┘              │
│                        Zustand store                         │
│                        TanStack Query                        │
└───────────────────────────┬──────────────────────────────────┘
                            │ HTTPS / JSON
┌───────────────────────────▼──────────────────────────────────┐
│ crates/api — Rust (Axum + Tokio)                             │
│                                                              │
│  /places   /routes   /search   /itineraries/:id   /meta      │
│                                                              │
│  search service → sources → normalizer → ranker → verifier    │
│       │              │                                       │
│       │              ├─ SerpAPI, Travelpayouts, Duffel       │
│       │              ├─ DB, Darwin, Transitous, GTFS         │
│       │              ├─ Obscura pool (JS/SPA collectors)     │
│       │              └─ operator deeplinks (prices)          │
│       │                                                      │
│       └─ SQLite / Postgres cache + quota broker               │
└──────────────────────────────────────────────────────────────┘
```

Obscura is embedded as a Rust crate, not run as a separate Node service. The full feed inventory and browser policy are in [docs/12-rust-data-layer.md](docs/12-rust-data-layer.md).

### Why this split

- Browser never holds API keys.
- The API normalises every source into one itinerary shape, so the globe never knows whether a leg came from Google Flights or a GTFS feed.
- Browser-bound collectors sit behind the same `FareSource` trait, so an Obscura result is normalized exactly like an API result.
- Caching and quotas live in one place.
- The web app can run against fixtures for design work.

### Stack

- **Web:** React 19, TypeScript, Vite, `react-globe.gl`, Three.js, Zustand, TanStack Query, Tailwind CSS, Radix primitives for the combobox and date picker.
- **API:** Rust, Axum, Tokio, Serde, `quick-xml`; SQLx with SQLite locally and Postgres when deployed.
- **Browser:** [Obscura](https://github.com/h4ckf0r0day/obscura) embedded as a Rust crate for JS-heavy and session-bound sources; pinned tag, non-root container, egress allowlist.
- **Shared:** `crates/core` for types, great-circle math, fingerprinting, ranking and total-cost logic.
- **Testing:** `cargo test` for the data layer; Vitest and Testing Library for the panel; Playwright for end-to-end.
- **Deploy:** static web on Vercel/Cloudflare; the Rust API and Obscura in one container on Fly.io/Render, with a persistent volume for SQLite, or Postgres if multi-instance.

---

## 8. Data and sources

### Places

| Dataset | Use | Licence |
|---|---|---|
| [OurAirports](https://ourairports.com/data/) | Airports, coordinates, type, scheduled service | Public domain |
| [OpenFlights routes](https://openflights.org/data.html) | Demo route graph for idle arcs | ODbL — attribution required |
| GTFS stops | Rail stations and coordinates | Per feed |
| [Mobility Database](https://mobilitydatabase.org/) | Discover GTFS feeds | Metadata CC0; feeds vary |
| Nominatim / OSM | City search fallback | ODbL; rate-limited |

Filter airports to those with scheduled service. A globe with 40,000 private airstrips is unreadable.

### Fares

| Mode | Source | Use |
|---|---|---|
| Air | SerpAPI Google Flights | Live search and verification |
| Air | Travelpayouts | Cached baselines, popular routes |
| Air | Duffel | Booking and deep links later |
| Rail | Transitous | Routing and schedules |
| Rail | Operator deeplinks | Prices and booking |
| Rail | Omio affiliate | Fallback prices and deeplinks |
| Browser-bound | Obscura | JS rendering, SPA JSON capture, sessions |

**Never scrape Google, Southwest or easyJet in production.** Use SerpAPI or a licensed source. See [docs/03-public-apis-vs-scraping.md](docs/03-public-apis-vs-scraping.md). Obscura's role, limits and integration pattern are in [docs/12-rust-data-layer.md](docs/12-rust-data-layer.md).

### Geometry

- **Flights:** great-circle interpolation. No external geometry needed.
- **Rail:** GTFS `shapes.txt` where available; otherwise OSM rail geometry clipped to the corridor; otherwise a ground-hugging great-circle arc, labelled as approximate.
- **Connectors:** straight lines between an airport and a station within 50 km of each other, used for mixed itineraries.

### Cache

| Data | TTL |
|---|---|
| Place search | 24 hours |
| Idle route arcs | 24 hours |
| Search results | 10 minutes |
| Offer detail | Until the offer's `expires_at`, max 30 minutes |
| Baselines | 24 hours |

---

## 9. Search and ranking

### Pipeline

1. **Parse the query.** Resolve places, validate dates, expand nearby airports and stations (up to 3 each).
2. **Fan out.** Air adapter and rail adapter run in parallel. Each returns candidate legs.
3. **Normalise.** One itinerary schema, one fingerprint, one currency.
4. **Combine.** Build direct itineraries, then split, positioning, open-jaw and mixed-mode combinations from the candidate legs. Cap the combination search.
5. **Rank.** Generalized cost, not fare.
6. **Verify.** Re-price the top N. Mark the rest as indicative.
7. **Return.** Include geometry per segment, not per itinerary, so the globe can render mixed modes.

### Ranking

```
score = price
      + (valueOfTime × doorToDoorHours)
      + riskPremium
      + arrivalPenalty
      − residualValue
```

Defaults: value of time €15/hour for leisure, 0 for overnight hours the traveller would sleep, 40% miss probability for a 30-minute delay on a self-connection. All adjustable in the UI behind an “advanced” disclosure.

### Deduplication

Fingerprint every segment as `hash(mode + operator + service + date + departure minute)`. Never key on an offer ID. Two sources selling the same flight collapse to one row; keep the lowest verified total.

---

## 10. API contract

```ts
// GET /api/places?q=lis&limit=8
type Place = {
  id: string;              // 'airport:BER' | 'station:de:8011160'
  kind: 'airport' | 'station' | 'city';
  code?: string;           // IATA or station code
  name: string;
  city: string;
  country: string;
  lat: number;
  lng: number;
};

// GET /api/routes?origin=airport:BER&limit=20
type RoutePreview = {
  destination: Place;
  mode: 'air' | 'rail';
  cheapest?: number;
  currency?: string;
  observedAt?: string;
};

// POST /api/search
type SearchRequest = {
  origin: string;
  destination: string;
  departDate: string;       // ISO date
  returnDate?: string;
  flexibility?: 0 | 1 | 3;
  passengers: number;
  cabin: 'economy' | 'premium' | 'business';
  bags: { carryOn: number; checked: number };
  modes: Array<'air' | 'rail'>;
  maxStops?: number;
  maxDurationHours?: number;
  verifiedOnly?: boolean;
};

type SearchResponse = {
  itineraries: Itinerary[];
  meta: {
    searchedAt: string;
    sources: Array<{ id: string; status: 'ok' | 'error'; ms: number }>;
    partial: boolean;
  };
};
```

`Itinerary` and `Segment` are defined in `packages/core` and shared by the API, the panel and the globe. Each segment carries `from`, `to`, `mode`, `operator`, `service`, `departLocal`, `arriveLocal`, `durationMinutes`, `geometry` (arc or path), `price`, `bags`, `bookingUrl`, `verifiedAt`.

---

## 11. Repository layout

```
fleet/
├─ PLAN.md                      ← this file
├─ docs/                        ← research and reference
├─ agents/                      ← fare-hunting subagents
└─ app/                         ← implementation (new)
   ├─ Cargo.toml
   ├─ crates/
   │  ├─ core/                 types, geo, rank, fingerprint
   │  ├─ sources/              one adapter per feed
   │  ├─ browser/              Obscura pool and page helpers
   │  ├─ store/                SQLite/Postgres, ledger, quota
   │  ├─ api/                  Axum routes
   │  └─ scheduler/            watchlist and refreshes
   ├─ web/                     React + react-globe.gl
   ├─ sidecar/                 optional Playwright fallback only
   └─ data/
      ├─ airports.json
      ├─ stations.json
      └─ rail-lines.geojson
```

Keep the app in a subdirectory so the research docs stay readable and the app can be extracted to its own repo if it grows.

---

## 12. Phases and acceptance criteria

| Phase | Build | Done when |
|---|---|---|
| **0 — Globe spike** (2–3 days) | React app, `react-globe.gl`, 1,000 airports, 500 arcs, click/hover, camera flight | 60fps on desktop and a mid-range phone; arcs are clickable; no label clutter |
| **1 — Search shell** (3–4 days) | Left panel, combobox, date range, mode toggles, mock search, result cards, selection highlighting | A user can search a fixture route, hover a card, see the arc lift, and click to frame it |
| **2 — Air data** (5–7 days) | Rust API, places, route previews, SerpAPI adapter, cache, quota broker, verification | Real BER–LIS results with total price, bags and a verified timestamp |
| **2b — Obscura collector** (3–5 days, optional) | Embedded Obscura pool, JSON-response capture, one JS-heavy source | One browser-bound source produces structured offers without DOM scraping |
| **3 — Rail data** (4–6 days) | Transitous routing, station dataset, GTFS shapes, path layer, rail result cards | A rail itinerary renders on actual track geometry and can be selected |
| **4 — Mixed itineraries** (4–6 days) | Segment combination, airport–station connectors, generalized-cost ranking, compare tray | A mixed air+rail itinerary is ranked and rendered with both modes visible |
| **5 — Polish** (3–5 days) | Mobile bottom sheet, empty/loading/error states, accessibility, reduced motion, shareable URLs | Works one-handed on a phone; keyboard and screen-reader paths exist |
| **6 — Alerts** (optional) | Watchlist, price history, “alert me” on a route, integration with `agents/watchkeeper.md` | A saved search notifies on a new low |

Ship phase 1 as an internal demo before wiring live data. The interaction is the risk, not the API.

---

## 13. Performance, accessibility and testing

### Performance

- First contentful paint under 1.5s; globe visible under 2.5s on a fast connection.
- Interaction latency under 100ms for hover and selection.
- 500 arcs / 1,500 points at 60fps; degrade gracefully by reducing arc resolution and labels.
- No layout shift when results load; reserve card heights.
- Pause rendering when the tab is hidden.

### Accessibility

- The result list is the primary interface; the globe is an enhancement, not the only way to use the app.
- Keyboard: arrow keys move through results, Enter selects, Esc clears, `/` focuses search.
- `aria-live="polite"` announces result counts and selection changes.
- Visible focus rings on the panel and cards.
- Reduced motion disables auto-rotate, arc draw-on and pulsing rings.
- Colour is never the only signal: selected arcs also get a thicker stroke and a label.
- Provide a “list only” toggle that hides the globe on low-power devices.

### Testing

- **Unit:** great-circle interpolation, date-line splitting, fingerprinting, ranking, total cost.
- **Component:** combobox, date range, result card, filter state.
- **E2E (Playwright):** search → hover → select → detail; mobile bottom sheet; URL reload restores the search.
- **Visual:** screenshot the globe with 1, 10 and 300 arcs; check arc colours in light and dark.
- **Accessibility:** axe on the panel and result list; keyboard-only pass.
- **Data:** fixture search responses for air, rail and mixed; a contract test per adapter.
- **Browser:** a fixture page served locally, captured through Obscura with network access blocked; assert the JSON capture and timeout path.
- **Security:** an egress test proving the browser container cannot reach an unlisted host.

---

## 14. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Globe performance on low-end phones | Cap arcs and labels; lower texture resolution; list-only mode |
| Label clutter over Europe | Label only the top 40 by traffic; show the rest on hover |
| Arc picking is fiddly | Add a result list as the primary selector; arcs are a secondary affordance |
| Rail geometry gaps | Use GTFS shapes; fall back to OSM; label approximate paths |
| Mixed-mode transfers are wrong | Use a curated airport–station transfer table for the top 50 metros; show the transfer time explicitly |
| Prices differ from the operator | Verify before display; show “price changed” rather than a stale number |
| API cost blows up | Quota broker, cached previews, paid verification only for the shortlist |
| Scraping/ToS | Licensed sources only in production; no Google, Southwest or easyJet scraping |
| Obscura cannot beat active bot protection | Its own docs exclude Cloudflare interactive challenges, Akamai/Datadome and CAPTCHAs. Treat those sources as unreachable; use a partner API or drop them |
| Obscura is young (v0.1.x) | Pin a release tag; wrap it behind `crates/browser`; keep a Playwright sidecar as a fallback for one or two sources |
| Untrusted page JavaScript runs in process | Non-root container, read-only root, egress allowlist, no credentials, bounded page time and size |
| Date-line arcs break the camera | Split geometries at ±180°; test Auckland–Santiago and Tokyo–Honolulu |
| Accessibility of a 3D canvas | The list is complete without the globe; keyboard and screen-reader paths |
| Data licensing | Attribute OurAirports, OpenFlights, OSM and GTFS feeds in an “About data” panel |

---

## 15. Decisions

### Decided

- **Data layer:** Rust, one workspace.
- **Browser:** Obscura, embedded via its Rust crate, wrapped in `crates/browser`.
- **Frontend:** TypeScript, React, `react-globe.gl`.
- **Booking:** deeplink to the operator; no in-app checkout.
- **Price policy:** a price is displayed only after verification at the source.

### Still open

1. **Coverage:** global air, or a regional launch? Which rail markets in v1 — UK, Western Europe, Japan, US, or all four?
2. **Live or demo:** should the first build use fixtures, or go straight to SerpAPI? Fixtures let the interaction ship faster; live data changes the loading and error design.
3. **Visual direction:** realistic Blue Marble globe, or a stylised dark globe with simplified landmasses? The latter is cheaper and more distinctive; the former is more legible.
4. **Price display:** show indicative prices from cache, or verified only? Verified only is slower but honest.
5. **Hosting and budget:** a SerpAPI cap per month, and whether the API runs locally or deployed.
6. **Name:** “Fleet Atlas” is a working title.

---

## 16. The first commit

When implementation starts, the first commit should contain only:

- Vite + React + TypeScript scaffold.
- `react-globe.gl` with a textured globe, 1,000 airports and 500 static arcs.
- Click and hover handlers that log the selected airport.
- A README with the run command.

No panel, no API, no data adapters. The first question is whether the globe feels right. Everything else is easier to change.
