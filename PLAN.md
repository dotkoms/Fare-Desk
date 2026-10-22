# Fleet Atlas — flight finder with buy/wait intelligence

A 3D globe of airports and routes, a search console on the left, and a route page that tells you whether to buy now or wait — using that route's own price history, not a generic “book 39 days out” rule.

**Scope:** flights only. Trains and coaches are future work, not v1.

**Status:** plan only. No implementation yet.

---

## 1. The product in one paragraph

A traveller opens Fleet Atlas, types “Berlin” and “Lisbon,” picks a week in March, and sees the best flights ranked on total cost. The globe draws the candidate routes as arcs; hovering a result lifts its arc, clicking it frames the route and dims the rest. A second tab answers the harder question: *should I buy now?* It shows the route's price history, where today's fare sits in the distribution, what usually happens between now and departure, and a plain recommendation — buy, wait, or watch — with a confidence level. A third mode, Explore, turns the globe into a map of cheap places from your home airport.

---

## 2. Goals and non-goals

### Goals

- Find the best flight for a specific trip, where “best” means total cost and total time, not headline fare.
- Tell the traveller when to buy, with route-level evidence and an honest confidence level.
- Make cheap destinations visible from one home airport, using the globe for discovery.
- Show the route structure — airports, stops, layovers — so the price makes sense.
- Feel fast: cached results visible immediately; live verification in the background.
- Work on a phone, where the panel becomes a bottom sheet.

### Non-goals for v1

- Trains, coaches or ferries.
- Booking inside the app. Every result deeplinks to the airline or a licensed OTA.
- Multi-city itineraries. One-way and return only.
- A machine-learning price predictor. v1 uses empirical distributions.
- User accounts, saved trips, social features.
- Full global live search on every query. Cached routes and background refresh are part of the design.

---

## 3. The experience

### Wireframe — search

```
┌──────────────────────────┬─────────────────────────────────────────────┐
│ FLEET ATLAS              │                                             │
│                          │                                             │
│  [One way] [Return]      │              3D GLOBE                       │
│                          │                                             │
│  From   BER  Berlin      │        ●───────╮                            │
│  To     LIS  Lisbon      │       /         ╲                           │
│  ⇄                       │      ●           ●   arcs                   │
│                          │                                             │
│  Depart  Mar 1           │                                             │
│  Return  Mar 8           │                                             │
│  1 adult  Economy        │                                             │
│  Bags: 1 checked         │                                             │
│                          │                                             │
│  [        Search       ] │                                             │
│                          │                                             │
│  Best ▾   12 results     │                                             │
│  ┌────────────────────┐  │                                             │
│  │ BER → LIS   6h 20  │  │                                             │
│  │ €89 · 1 stop · bag │  │                                             │
│  │ verified 4 min ago │  │                                             │
│  └────────────────────┘  │                                             │
│  ┌────────────────────┐  │                                             │
│  │ BER → LIS   7h 05  │  │                                             │
│  │ €104 · nonstop    │  │                                             │
│  └────────────────────┘  │                                             │
└──────────────────────────┴─────────────────────────────────────────────┘
```

### Wireframe — route intelligence

```
┌──────────────────────────┬─────────────────────────────────────────────┐
│ BER → LIS   Mar 1–8      │                                             │
│                          │   the globe shows the route,               │
│  €89                     │   with the price history in the panel       │
│  ┌────────────────────┐  │                                             │
│  │ 12th percentile    │  │                                             │
│  │ 90-day median €151 │  │                                             │
│  └────────────────────┘  │                                             │
│                          │                                             │
│  [ price history chart ] │                                             │
│                          │                                             │
│  ┌────────────────────┐  │                                             │
│  │ BUY                │  │                                             │
│  │ 78% confidence     │  │                                             │
│  │ Expected +€35 if   │  │                                             │
│  │ you wait 3 weeks   │  │                                             │
│  └────────────────────┘  │                                             │
│                          │                                             │
│  [Set alert at €80]      │                                             │
└──────────────────────────┴─────────────────────────────────────────────┘
```

### Primary flows

**A. Search a known trip**

1. Pick origin, destination, dates, passengers and bags.
2. Results appear as cards; the globe draws the top candidate routes.
3. Hover a card: its arc lifts and brightens; a tooltip shows airports and local times.
4. Click a card: camera frames the route, other arcs dim, the detail drawer opens.
5. “When to buy” opens the route-intelligence tab for the same route and dates.

**B. Ask when to buy**

1. From a result, click “When to buy,” or start with a route and date.
2. The panel shows current price, percentile, median, the days-to-departure curve, and a recommendation.
3. If there is not enough history, it says so and starts a watch instead of inventing advice.
4. “Alert me” stores the route, date window and threshold.

**C. Explore from home**

1. Choose a home airport.
2. The globe shows the cheapest destinations from there, scored against each route's baseline.
3. Arcs are coloured by discount; the panel lists destinations with price and percentile.
4. Clicking a destination opens its route page.

### Empty, loading and error states

- **No origin:** globe idle, panel shows recent searches.
- **Loading:** cached results appear immediately; a “verifying prices” line updates in place.
- **No results:** explain which filter removed the last result and offer to relax it.
- **Price changed:** mark the card and re-price; never show the old number as live.
- **Insufficient history:** “Collecting baseline — 12 of 14 days.” No advice.

---

## 4. The 3D scene

### Rendering

**Recommended: `react-globe.gl` (Three.js).** Arcs, points, labels, rings, hover and click, camera flight and animated dashes are built in. The globe is the product's face; use a library that already solved great-circle arcs.

**Alternative:** MapLibre globe projection plus deck.gl, if basemap fidelity and terrain become more important than the cinematic globe.

### Layers

| Layer | Content | Notes |
|---|---|---|
| Globe | NASA Blue Marble texture, bump map, night lights | 4k desktop, 2k mobile |
| Atmosphere | Rim glow, thin haze | Subtle |
| Airports | Points sized by scheduled departures; IATA labels at high zoom | Cap labels to top 40 in view |
| Flight arcs | Great-circle arcs, altitude scaled by distance | Cap 300 visible |
| Selection rings | Pulsing rings at origin and destination | Stop after 3 cycles |
| Tooltip | HTML overlay with code, city, local time, price | Pointer-following |

### Arc behaviour

- **Altitude:** `0.05 + 0.25 × (distance / maxDistance)`, clamped.
- **Resolution:** 64 points normally, 128 for the selected itinerary.
- **Base colour:** cyan `#38BDF8`; selected white `#F8FAFC`; dimmed 8% opacity.
- **Explore mode:** colour by discount versus baseline — teal below p10, neutral between p10 and p50, amber above p50 — with a legend and price labels, not colour alone.
- **Animation:** 600ms draw-on when results arrive; slow dash along the selected route. Respect `prefers-reduced-motion`.

### Camera

- Origin selected: fly to the airport at altitude 1.8 over 900ms.
- Search complete: frame the bounding box of the top results, with padding for the panel.
- Selection: frame the itinerary; for long routes, centre the midpoint and pull back.
- Anti-meridian: split geometries at ±180°; test Auckland–Santiago and Tokyo–Honolulu.
- Idle: auto-rotate at 0.3 rpm until first interaction; disabled by reduced motion.

### Performance

- 300 arcs and 1,500 points at 60fps on an M1 Air and a mid-range Android.
- Initial JS under 1.5 MB gzipped; texture lazy-loaded.
- Pause the render loop when the tab is hidden.

---

## 5. The left panel

### Search

- One-way / return.
- Origin and destination with city/airport search and swap.
- Date range with ±1/±3 day flexibility.
- Adults, children, cabin, bags.
- Max stops, departure window, max duration.
- Search.

### Results

- Sort: Best, Cheapest, Fastest, Earliest.
- Filter: nonstop, bags included, verified only, airline.
- Card: airports, times in local time, duration, stops, total price with bags, verification age, risk badge.
- Virtualised list.

### Route intelligence

- Current price and percentile against the route's 90-day distribution.
- 90-day median, p10, p90.
- Price-history chart with the travel date marked.
- Days-to-departure curve: what the fare usually does from here.
- Recommendation card: BUY / WAIT / WATCH, confidence, expected change, downside.
- Fare rules: refundable, changeable, bag inclusion.
- Alert threshold and “watch this route.”
- Backtest note: “On this route, waiting beat buying in 6 of 10 similar cases.”

### Explore

- Home airport selector.
- Destination list with current price, percentile and nonstop/stops.
- Filter: nonstop, under €X, weekend, month.
- Click a destination to open its route page.

### Mobile

Bottom sheet with collapsed, half and full states. The globe stays full-bleed. Search, result selection and the recommendation must work one-handed.

---

## 6. Buy/wait engine

This is the product's differentiator. It must be empirical and auditable, not a black box.

### Data it needs

- Daily minimum price for the route and travel date, collected over time.
- Historical prices for the same route and comparable lead times.
- The route's days-to-departure curve.
- Seasonality and event calendar.
- Current offer details: fare rules, bags, seats left at this price.
- Volatility: how much the route's price moves day to day.

### Output

```
Recommendation: BUY
Confidence: 78%
Current: €89 (12th percentile of 90 days)
Median: €151   p10: €84   p90: €220
Expected in 7 days:  €92–€110
Expected in 21 days: €120–€170
Downside: 18% chance the fare falls below €80 if you wait
Fare rules: non-refundable, 1 checked bag included
Why: below the 10th percentile, fewer than half the seats left
     at this fare, and this route rises sharply inside 21 days.
```

### Rules

- Below p10 → default BUY, unless the fare rules are bad for the traveller's flexibility.
- Between p10 and p50 → WATCH, with an alert at the p10.
- Above p50 and more than 45 days out → WAIT, with a re-check date.
- Inside 14 days → BUY or WATCH; do not promise a late drop.
- Less than 14 observations or no comparable lead-time history → no advice. Say “collecting baseline.”
- Refundability changes the call: a slightly higher flexible fare can be the better buy.
- Always show the total with bags and fees.

### Confidence

Confidence reflects sample size, dispersion and how well the current price fits the historical pattern. It is not a probability that the advice is correct. State that in the UI.

### Backtest

For every route with enough history, replay the advice at 60, 30 and 14 days out and compare it with buying at that moment. Publish:

- share of cases where waiting beat buying,
- average saving or loss,
- worst-case loss.

If the advice does not beat “buy now” after costs, the product should say so and position itself as search plus alerts.

### Cold start

Historical prices are not freely available at route level for arbitrary past dates. Seed with Travelpayouts cached prices and any history the licensed source exposes, then collect daily. Do not issue advice until the route has enough observations. The first users will see watch pages, not recommendations — design that as a legitimate state, not an error.

---

## 7. Interaction state

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
    maxStops: number | null;
  };
  results: {
    status: 'idle' | 'loading' | 'ready' | 'error';
    itineraries: Itinerary[];
    verifying: boolean;
  };
  routeIntel: {
    status: 'idle' | 'loading' | 'ready' | 'insufficient';
    snapshot: RouteSnapshot | null;
  };
  explore: {
    home: Place | null;
    destinations: DestinationDeal[];
    status: 'idle' | 'loading' | 'ready';
  };
  selection: {
    hoveredId: string | null;
    selectedId: string | null;
  };
};
```

Rules:

- `hoveredId` never moves the camera.
- `selectedId` frames the route and dims the rest.
- Changing a query invalidates results but keeps the old selection at 30% until the new search resolves.
- URL state: `?from=BER&to=LIS&depart=2027-03-01&return=2027-03-08`. Shareable and reload-safe.

---

## 8. Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ web/ — React + TypeScript + Vite                             │
│  SearchPanel · GlobeScene · ResultList · RouteIntel · Explore │
│  Zustand store · TanStack Query                              │
└───────────────────────────┬──────────────────────────────────┘
                            │ HTTPS / JSON
┌───────────────────────────▼──────────────────────────────────┐
│ crates/api — Rust (Axum + Tokio)                             │
│  /places /routes /search /route-intel /explore /alerts        │
│  search → sources → normalizer → ranker → verifier            │
│       ├─ SerpAPI / SearchApi (Google Flights)                 │
│       ├─ Travelpayouts (cached baselines)                     │
│       ├─ Duffel (booking links)                               │
│       └─ Obscura (optional, browser-bound sources)            │
│  SQLite / Postgres: offers, history, ledger, quota            │
└──────────────────────────────────────────────────────────────┘
```

### Stack

- **Web:** React 19, TypeScript, Vite, `react-globe.gl`, Three.js, Zustand, TanStack Query, Tailwind, Radix.
- **API:** Rust, Axum, Tokio, Serde, SQLx.
- **Browser:** Obscura only if a source requires it; not needed for the licensed air path.
- **Shared:** `crates/core` for types, geo, fingerprint, ranking, percentiles.
- **Testing:** `cargo test`, Vitest, Playwright.
- **Deploy:** static web; Rust API and SQLite/Postgres in one container.

---

## 9. Data and sources

### Places

| Dataset | Use | Licence |
|---|---|---|
| [OurAirports](https://ourairports.com/data/) | Airports, coordinates, scheduled service | Public domain |
| [OpenFlights routes](https://openflights.org/data.html) | Demo route graph for idle arcs | ODbL |
| Nominatim / OSM | City search fallback | ODbL; rate-limited |

Filter to airports with scheduled service. Cap idle arcs to the top destinations by traffic.

### Fares

| Source | Use |
|---|---|
| SerpAPI Google Flights | Live search and verification |
| SearchApi | Cheaper at volume, same surface |
| Travelpayouts | Cached baselines and price calendar |
| Duffel | Booking deeplinks and offer detail |
| Skiplagged MCP | Flexible-date and hidden-city leads (flags required) |
| Kiwi MCP | Self-transfer leads (flags required) |
| Obscura | Optional, browser-bound sources only |

**Never scrape Google, Southwest or easyJet in production.** Use SerpAPI or a licensed source. See [docs/03-public-apis-vs-scraping.md](docs/03-public-apis-vs-scraping.md) and [docs/12-rust-data-layer.md](docs/12-rust-data-layer.md).

### History

Store one row per route, travel date and observation day:

```sql
CREATE TABLE fare_history (
  route        TEXT,
  travel_date  TEXT,
  observed_on  TEXT,
  price        REAL,
  currency     TEXT,
  source       TEXT,
  fingerprint  TEXT,
  PRIMARY KEY (route, travel_date, observed_on, fingerprint)
);
```

The daily minimum is the signal. Do not store every quote; store the minimum and the offer that produced it.

### Cache

| Data | TTL |
|---|---|
| Place search | 24 hours |
| Idle route arcs | 24 hours |
| Search results | 10 minutes |
| Offer detail | Until `expires_at`, max 30 minutes |
| Route history | Daily rollup, updated once a day |
| Explore feed | 6 hours |

---

## 10. Search and ranking

1. Resolve places; expand nearby airports (up to 3 each).
2. Query the air source for the date range and passenger/bag parameters.
3. Normalize every offer into one schema; fingerprint by physical service.
4. Drop duplicates; keep the lowest verified total.
5. Rank by generalized cost:

```
score = total price
      + (value_of_time × door_to_door_hours)
      + stop_penalty
      + risk_premium
```

6. Verify the top N at the source; mark the rest indicative.
7. Return itineraries plus the route snapshot for the intelligence tab.

Defaults: value of time €15/hour; stop penalty €25 per stop; risk premium €40 for a self-transfer and €25 for an OTA-only fare. All adjustable behind “advanced.”

**Best** is the weighted score. **Cheapest** and **Fastest** are separate sorts so the user can see the trade-off.

---

## 11. API contract

```ts
type Place = {
  id: string;              // 'airport:BER'
  code: string;            // IATA
  name: string;
  city: string;
  country: string;
  lat: number;
  lng: number;
};

type Itinerary = {
  id: string;
  segments: Segment[];
  totalPrice: number;
  currency: string;
  totalDurationMinutes: number;
  stops: number;
  bags: { carryOn: number; checked: number };
  fareRules: { refundable: boolean; changeable: boolean };
  bookingUrl: string;
  source: string;
  verifiedAt: string | null;
  risk: Array<'self-transfer' | 'ota-only' | 'hidden-city'>;
};

type RouteSnapshot = {
  route: string;
  travelDate: string;
  currentPrice: number;
  percentile: number;
  median90: number;
  p10: number;
  p90: number;
  history: Array<{ date: string; price: number }>;
  curve: Array<{ daysOut: number; median: number; p10: number; p90: number }>;
  recommendation: 'buy' | 'wait' | 'watch' | 'insufficient';
  confidence: number | null;
  expectedChange: { days: number; low: number; high: number } | null;
  backtest: { cases: number; waitWins: number; avgSaving: number; worstLoss: number } | null;
};

// GET  /api/places?q=lis
// GET  /api/routes?origin=airport:BER&limit=20
// POST /api/search
// GET  /api/route-intel?from=BER&to=LIS&date=2027-03-01
// GET  /api/explore?home=BER&month=2027-03
// POST /api/alerts
```

---

## 12. Repository layout

```
fleet/
├─ PLAN.md
├─ docs/
├─ agents/
└─ app/
   ├─ Cargo.toml
   ├─ crates/
   │  ├─ core/          types, geo, rank, percentiles, fingerprint
   │  ├─ sources/       serpapi, travelpayouts, duffel, mcp
   │  ├─ browser/       optional Obscura pool
   │  ├─ store/         SQLite/Postgres, history, quota
   │  ├─ api/           Axum routes
   │  └─ scheduler/     daily history collection, alerts
   ├─ web/              React + react-globe.gl
   └─ data/
      ├─ airports.json
      └─ routes.json
```

---

## 13. Phases and acceptance criteria

| Phase | Build | Done when |
|---|---|---|
| **0 — Globe spike** (2–3 days) | React app, `react-globe.gl`, 1,000 airports, 500 arcs, hover/click, camera flight | 60fps on desktop and a mid-range phone; arcs clickable; no label clutter |
| **1 — Search shell** (3–4 days) | Panel, combobox, date range, result cards, mock search, selection highlighting | A fixture route can be searched, hovered and selected |
| **2 — Air data** (5–7 days) | Rust API, places, route previews, SerpAPI adapter, cache, quota, verification | Real BER–LIS results with total price, bags and a verified timestamp |
| **3 — History collection** (3–4 days) | Daily scheduler, `fare_history`, percentiles, route snapshot endpoint | A route accumulates daily minima and returns p10/p50/p90 |
| **4 — Buy/wait** (4–6 days) | Curve, recommendation rules, confidence, backtest, route-intelligence panel | A recommendation is shown only with enough data; the backtest is visible |
| **5 — Explore** (3–4 days) | Home airport, cheap-destination feed, arc colouring, destination cards | The globe shows the cheapest destinations from one airport with percentile labels |
| **6 — Alerts and polish** (4–6 days) | Watchlist, alert thresholds, mobile bottom sheet, accessibility, reduced motion | A saved route notifies on a new low; mobile works one-handed |

Ship phase 1 as an internal demo. Ship phase 2 before investing in the advice engine; without reliable prices the advice is noise.

---

## 14. Performance, accessibility and testing

### Performance

- First contentful paint under 1.5s; globe under 2.5s.
- Interaction under 100ms for hover and selection.
- 300 arcs / 1,500 points at 60fps.
- No layout shift when results load.
- Pause rendering when hidden.

### Accessibility

- The result list is the primary interface; the globe is an enhancement.
- Keyboard: arrow keys through results, Enter selects, Esc clears, `/` focuses search.
- `aria-live` for result counts and selection.
- Reduced motion disables rotation, draw-on and pulsing.
- Colour is never the only signal: selected arcs get a thicker stroke and a label.
- A “list only” toggle for low-power devices.

### Testing

- **Unit:** great-circle, anti-meridian, fingerprinting, percentiles, ranking, total cost.
- **Component:** combobox, date range, result card, recommendation card, empty/insufficient states.
- **E2E:** search → hover → select → route intel; explore → destination → route intel; mobile sheet.
- **Data:** fixture responses for air; contract test per source.
- **Backtest:** a deterministic fixture history with a known answer.
- **Accessibility:** axe and a keyboard-only pass.

---

## 15. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Advice is wrong or generic | Route-level history, confidence, backtest, and an explicit “insufficient data” state |
| Cold start | Start collecting before launch; seed from cached sources; show watch pages |
| Search cost and latency | Cached results first, verify in background; paid calls only for the shortlist |
| Price changes between display and click | Re-price, timestamp, and show “price changed” |
| Google/Kayak/Hopper already do this | Differentiate on auditability, total cost and no dark patterns, not on a magic score |
| Globe is a demo, not a tool | List-first results; globe supports discovery and route context; 2D toggle later |
| Airport/label clutter | Cap labels and arcs by traffic; show more on zoom |
| Anti-meridian arcs | Split at ±180°; test long routes |
| Scraping/ToS | Licensed sources only for displayed prices; no Google/Southwest/easyJet scraping |
| Obscura limits | Optional; does not beat active bot protection; keep a fallback |
| Data licensing | Attribute OurAirports, OpenFlights, OSM |

---

## 16. Decisions

### Decided

- **Flights only.**
- **Data layer:** Rust, one workspace.
- **Browser:** Obscura optional, embedded, pinned; not required for v1.
- **Frontend:** TypeScript, React, `react-globe.gl`.
- **Booking:** deeplink.
- **Advice:** empirical distributions, no ML, no advice without enough history.

### Still open

1. **Home market:** which airport or country is the first-class citizen? The advice engine needs a concentration of routes.
2. **Live or fixtures:** fixtures for the interaction, or live SerpAPI from the start?
3. **Visual direction:** realistic Blue Marble or a stylised dark globe.
4. **Advice policy:** how aggressive should BUY be below p10? How much weight on refundability?
5. **Budget:** SerpAPI cap and hosting.
6. **Name:** “Fleet Atlas” is a working title.

---

## 17. The first commit

- Vite + React + TypeScript scaffold.
- `react-globe.gl` with a textured globe, 1,000 airports and 500 static arcs.
- Hover and click handlers that log the selected airport.
- A README with the run command.

No panel, no API, no advice engine. The first question is whether the globe feels right.
