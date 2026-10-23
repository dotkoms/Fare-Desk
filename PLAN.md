# Fare Desk — flight finder with buy/wait intelligence

A 3D globe of airports and routes, a search console on the left, and a route page that tells you whether to buy now or wait — using that route's own price history, not a generic “book 39 days out” rule.

**Scope:** flights only. Trains and coaches are future work, not v1.

**Status:** Steps 1–8 are built except BTS reliability: Explore has the climate overlay; airport access has curated transfers, live OSRM driving and Transitous transit legs and MCTs; door-to-door combines fare and transfers; search results carry MCT warnings. True reach isochrones and reliability-adjusted prices remain. `app/web` holds the list-first shell and globe; `app/crates` holds the data plane (places, search, reference data, cache, quota, provenance) and the daily history collector. `GET /api/route-intel` returns daily minima, percentiles, sample and window. The Step 5 buy/wait engine is built behind `/api/route-intel` (rules, expected ranges, downside, curve, backtest, reasons). `POST /api/price-check` screens storefronts with eligibility, FX-adjusted net totals, status labels and the persistence threshold. `/api/seasonality`, `/api/fare-weather` and `/api/explore` are live over the collected history. Next: BTS reliability and transit isochrones, then alerts and the frontend wiring. Live fares need `SERPAPI_KEY`. [docs/23-2d-poster-map.md](docs/23-2d-poster-map.md) proposes replacing the 3D globe with a flat poster map; the decision is open.

---

## 1. The product in one paragraph

Fare Desk is a discovery and redirection product: it finds the best flights and the best time to buy, then sends the traveller to the airline or OTA to complete the purchase. It does not sell tickets, take payment, or handle changes and refunds.

A traveller opens Fare Desk, types “Berlin” and “Lisbon,” picks a week in March, and sees the best flights ranked on total cost. The globe draws the candidate routes as arcs; hovering a result lifts its arc; clicking selects it, dims the rest to 15%, and moves the camera only if the route is outside the safe area. A second tab answers the harder question: *should I buy now?* It shows the route's price history, where today's fare sits in the distribution, what usually happens between now and departure, and a plain recommendation — book now, wait, or watch — with the range, sample size and track record. When a route looks good, it also checks whether another country storefront the traveller is eligible for beats the home price after FX and bags. A third mode, Explore, turns the globe into a map of cheap places from your home airport.

---

## 2. Goals and non-goals

### Goals

- Find the best flight for a specific trip, where “best” means total cost and total time, not headline fare.
- Tell the traveller when to buy, with route-level evidence, an honest range, the sample size and a track record.
- Surface legitimate point-of-sale savings, net of FX and eligibility, without faking residency.
- Make cheap destinations visible from one home airport, using the globe for discovery.
- Show the route structure — airports, stops, layovers — so the price makes sense.
- Feel fast: cached results visible immediately; live verification in the background.
- Work on a phone, where the panel becomes a bottom sheet.

### Non-goals for v1

- Trains, coaches or ferries.
- Selling tickets, processing payments, issuing tickets, or handling changes and refunds.
- Booking inside the app. Every result redirects to the airline or a licensed OTA, where the purchase is completed.
- Multi-city itineraries. One-way and return only.
- A machine-learning price predictor. v1 uses empirical distributions.
- User accounts, saved trips, social features.
- Full global live search on every query. Cached routes and background refresh are part of the design.
- A mandatory 3D globe or 3D-only navigation. The list and route board must work without WebGL.

### Commerce model

Fare Desk is not a travel agency, OTA or merchant of record. It does not sell, issue, change or refund tickets, and it never takes payment. Every result links to the provider — the airline, the OTA, or the operator — and the transaction happens there.

- The displayed price is verified at the time of display; the final price and availability are confirmed on the provider's site.
- The provider's fare rules, payment requirements and after-sales terms govern the purchase.
- Affiliate links, if used, are disclosed.
- The app never auto-fills residency or billing details, and never books on the traveller's behalf.

---

## 3. The experience

### Wireframe — search

```
┌──────────────────────────┬─────────────────────────────────────────────┐
│ FARE DESK              │                                             │
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
│  │ 7 in 10 calls      │  │                                             │
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
4. Click a card: select it; `ensureVisible` only if needed; other arcs dim to 15%; the detail drawer opens.
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

The interface specification, component system and palette are in [docs/16-ui-ux.md](docs/16-ui-ux.md), [docs/17-design-system.md](docs/17-design-system.md) and [docs/18-color.md](docs/18-color.md). Gamification is in [docs/19-gamification.md](docs/19-gamification.md). User and competitor research is in [docs/20-users-and-competitors.md](docs/20-users-and-competitors.md).

### Empty, loading and error states

- **No origin:** globe idle, panel shows recent searches.
- **Loading:** cached results appear immediately; a “verifying prices” line updates in place.
- **No results:** explain which filter removed the last result and offer to relax it.
- **Price changed:** mark the card and re-price; never show the old number as live.
- **Insufficient history:** “Collecting baseline — 12 of 14 days.” No advice.

---

## 4. The globe and route board

The globe is the lobby, not the desk. It carries orientation, Explore and route context. The list and the route board carry the decision.

### Rendering

**v1: `react-globe.gl` for the globe, plus a flat route board for route scale.** The globe owns world and continent views; below a camera altitude of 1.35 R, crossfade to a flat route board. A globe shows one hemisphere, and at route scale the destination can sit behind the horizon. Prop map, data shapes and budgets: [docs/21-globe-implementation.md](docs/21-globe-implementation.md).

**Alternative if one engine matters more than speed of build:** MapLibre GL JS globe + deck.gl, which gives globe/Mercator switching, DOM labels, collision and `fitBounds` in one stack. More integration work, fewer hand-rolled systems.

### Layers

| Layer | Content | Notes |
|---|---|---|
| Globe | Low-chroma paper globe (light) or night globe (dark); no texture, no stars, no city lights | 2k is enough; no NASA imagery |
| Atmosphere | 2px rim at 12% opacity | No glow, no bloom |
| Airports | Points sized by scheduled departures; HTML labels in screen space | Budgeted, collision-tested |
| Flight arcs | Great-circle arcs, altitude by distance | 40 / 120 / 300 by zoom |
| Selection | 2.5px arc + 2px casing, endpoint rings, one 600ms ring | No pulse loop |
| Tooltip | HTML overlay with code, city, local time, price | Pointer-following |

### Labels

- Render labels as **HTML in screen space**, not 3D text meshes. Position them from `getScreenCoords`, rank them, and collision-test them.
- Pinned labels (origin, destination, selection, hover) are never hidden. Lower-priority labels hide when they collide.
- Budgets: 12 labels / 40 arcs at world view; 24 / 120 at continent; 40 / 300 at region; endpoints plus three connections at route view. Halve on mobile.
- On a camera flight, hide non-pinned labels and fade them in over 120ms after the camera settles.

### Arc behaviour

- **Altitude:** `clamp(0.02 + 0.18 × distanceKm / 20000, 0.02, 0.20)`; selected +0.02.
- **Width encodes state, not price:** 1px base, 1.5px hover, 2.5px selected plus a 2px casing.
- **Colour:** base `--arc-base` at 60%; selected `--ink` on light and `#F8FAFC` on dark; dimmed 35% on hover, 15% on selection. Never 8%.
- **Deduplicate by airport pair:** twelve BER–LIS itineraries are one arc with a “12 options” badge, not twelve arcs.
- **One-stop itineraries are polylines with a connection dot**, not direct arcs.
- **No draw-on, no dash, no plane animation.** Fade new arcs in over 200ms after the camera settles. A fare search is not live tracking.
- **Explore arcs are destination rays**, not flight paths: one direct line from the home airport to each of the top 12 deals, with the true stop count in the list.

### Camera

- Hover and keyboard focus never move the camera.
- Prefer `ensureVisible` over a flight: if both endpoints are already inside the safe rectangle with 24px margin, do not move.
- Flight duration: `clamp(350 + 450 × angularDistance/π, 350, 800)` ms; interruptible by any pointer or key input.
- Suppress auto-fit for 8 seconds after the user moves the camera; show a “Fit route” chip instead.
- Frame the route between the panel and the safe area; on mobile, pad above the sheet.
- Anti-meridian: normalize the longitude delta to the short way; split geometry at ±180°; test Auckland–Santiago, Tokyo–Honolulu and Los Angeles–Sydney.
- **No auto-rotation.** A rotating globe moves targets under the cursor, makes labels swim and triggers vestibular symptoms.

### Performance

- Tier A desktop: 300 arcs, 1,500 points, 40 labels, DPR 2.
- Tier B laptop: 180 arcs, 900 points, 32 labels, DPR 1.5.
- Tier C mobile: 100 arcs, 500 points, 16 labels, DPR 1.5, no atmosphere.
- Tier D fallback: static poster plus the list; no WebGL.
- Show a static poster immediately and crossfade to WebGL when the first frame is ready.
- Handle `webglcontextlost` with the poster and a reload action.
- Pause the render loop when the tab is hidden or the sheet is full.

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

- **Route strip first:** `BER 10:40 ── 1h 50m ── LIS 12:30 / nonstop`. A one-stop itinerary shows the connection and layover. This is more legible than the globe for “what am I buying?”
- Current total and percentile against the route's 90-day distribution.
- 90-day median, p10, p90.
- Recommendation card **above** the chart: BOOK NOW / WAIT / WATCH / NOT ENOUGH DATA, with range, sample, track record, expected change and downside.
- Two small multiples: 90-day price history and days-to-departure curve. Never a dual-axis chart.
- Fare rules: refundable, changeable, bag inclusion.
- **Provider options:** a short comparison of eligible storefronts with net totals and Eligible / Listed / Ineligible status. Hidden unless a material, persistent gap exists. Each row links out; the purchase happens on the provider's site.
- Alert threshold and “watch this route.”
- Backtest note: “On this route, waiting beat buying in 6 of 10 similar cases.”
- “What would change our mind”: one sentence.
- Fare rules: refundable, changeable, bag inclusion.
- **Provider options:** a short comparison of eligible storefronts with net totals and Eligible / Listed / Ineligible status. Hidden unless a material, persistent gap exists. Each row links out; the purchase happens on the provider's site.
- Alert threshold and “watch this route.”
- Backtest note: “On this route, waiting beat buying in 6 of 10 similar cases.”

### Explore

- Home airport selector.
- Destination list with current price, percentile and nonstop/stops.
- Filter: nonstop, under €X, weekend, month.
- Click a destination to open its route page.

### Mobile

- Three detents: peek 88px, half 48dvh, full 92dvh.
- The globe is fixed behind the sheet; do not put it inside a scrolling page.
- The sheet header and drag handle move the sheet; the content scrolls.
- Primary actions sit in the bottom 60% of the screen, sticky above the safe area.
- 48×48dp targets; cards 72–88px tall; chips 40px with 8px gaps.
- Do not use horizontal swipe between Results and When to buy; use tabs in the sheet header. A horizontal swipe on the globe is a rotate gesture.
- Search on mobile is a summary chip (`BER → LIS · Mar 1–8 · 1 adult`) that opens a full-height search sheet.
- The globe is not interactive when the sheet is full; pause rendering.

---

## 6. Finding the best price

Four levers, ordered by expected value. The first two ship in v1; the third ships once search and history are reliable; the fourth is a later phase.

| Lever | What it does | V1? |
|---|---|---|
| **Dates and airports** | Flexible dates, nearby airports, total cost | Yes |
| **Timing** | Buy/wait on the route's own history | Yes |
| **Point of sale** | Eligible storefronts, net of FX and payment rules | After search works |
| **Itinerary structure** | Split tickets, positioning, open-jaw | Later |

### 6.1 Dates and airports

- Search the full date window, not one date.
- Expand to up to three origin and three destination airports.
- Price total cost with bags, seats and ground transfer.
- Do not spend paid calls on every date; scan the calendar, then verify the shortlist.

This is the cheapest lever and it needs no new data source.

### 6.2 Timing and the buy/wait engine

Timing is the product's main differentiator. It must be empirical and auditable, not a black box.

#### Data it needs

- Daily minimum price for the route and travel date, collected over time.
- Historical prices for the same route and comparable lead times.
- The route's days-to-departure curve.
- Seasonality and event calendar.
- Current offer details: fare rules, bags, seats left at this price.
- Volatility: how much the route's price moves day to day.

#### Output

```
Recommendation: BOOK NOW
Current total: €119 (1 checked bag)
Typical range: €110–€151 (middle 80% of fares paid, N=84)
Position: below the 10th percentile
Expected in 7 days:  €122–€140
Expected in 21 days: €150–€210
Downside: 18% chance it falls below €110 if you wait
Fare rules: non-refundable; change €75 + fare difference
Track record: when we said this on similar routes, it was right
              7 times in 10 (n=214)
Why: below the typical range, fewer than half the seats left at this
     fare, and this route rises sharply inside 21 days.
What would change our mind: a new sale bucket, a schedule change, or
     a drop below €105 before Tuesday.
```

#### Rules

- Below p10 → **Book now**, unless the fare rules are bad for the traveller's flexibility.
- Between p10 and p50 → **Watch**, with an alert at the p10. Do not manufacture a verdict.
- Above p50 and more than 45 days out → **Wait**, with a re-check date.
- Inside 14 days → **Book now** or **Watch**; never promise a late drop.
- Less than 14 observations or no comparable lead-time history → **Not enough data**. Say “collecting baseline.”
- Enough data but a mixed signal → **No clear signal** and a watch. “Your call” is a valid answer.
- Refundability changes the call: a slightly higher flexible fare can be the better buy.
- Always show the total with bags and fees.
- Never show a raw “confidence: 78%”. Show the range, the sample and the track record.

#### Track record and uncertainty

A confidence number is easy to misread as a probability that the advice is right. Do not show one. Show:

- the outcome in plain frequencies (“6 in 10 similar trips fell”),
- the sample size and window,
- the track record on this route or route class, including misses,
- what would change the recommendation.

If the track record is not available, say so. A product that only surfaces successes will be judged on its first visible failure.

#### Backtest

For every route with enough history, replay the advice at 60, 30 and 14 days out and compare it with buying at that moment. Publish:

- share of cases where waiting beat buying,
- average saving or loss,
- worst-case loss,
- calibration: Brier score or a reliability diagram, with the misses shown.

If the advice does not beat “buy now” after costs, the product should say so and position itself as search plus alerts.

#### Cold start

Historical prices are not freely available at route level for arbitrary past dates. Seed with Travelpayouts cached prices and any history the licensed source exposes, then collect daily. Do not issue advice until the route has enough observations. The first users will see watch pages, not recommendations — design that as a legitimate state, not an error.

### 6.3 Point of sale

The same physical flight can be cheaper on another country's storefront. The app should check the ones the traveller can actually use.

**Screening.** For a shortlisted itinerary, run the same search across three to five markets: departure country, destination country, US, UK, and one competitive low-fare market. SerpAPI's Google Flights endpoint accepts `gl` (country), `hl` (language) and `currency`; deep-link to the airline's local storefront to confirm.

**Eligibility.** Store residency, nationality, the countries where the traveller holds a card or account, and the payment methods they can use. No card numbers, no addresses. Label every row:

- **Eligible** — fare rules and payment requirements match the traveller; the purchase can be completed on the provider's site.
- **Listed, unconfirmed** — visible price, but a local card or billing address may be required.
- **Ineligible** — residency-restricted or local-payment-only.

**Net total.** Fare + bags + seats + provider booking fee + card FX fee + forced conversion + ground transfer. Convert at the card's rate, not the airline's marketing rate.

**Threshold.** Recommend a foreign storefront only when the net gap is above €20 or 5%, whichever is higher, and has persisted for several days, and the fare rules are equivalent.

**Guardrails.** No VPN or proxy to fake residency. No fabricated billing addresses. No borrowed cards. No promise of the final price until the payment step. The full mechanics, evidence and rollout are in [docs/13-point-of-sale.md](docs/13-point-of-sale.md).

### 6.4 Itinerary structure (later)

Split tickets, positioning flights and open-jaws can beat a through fare, but they create separate contracts and connection risk. They belong in a later phase, behind clear risk badges and a buffer rule. The tactics and thresholds are in [docs/04-playbook.md](docs/04-playbook.md).

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
    focusedId: string | null;   // keyboard focus, ephemeral
    selectedId: string | null;  // persistent, in URL
  };
  mode: 'search' | 'intel' | 'explore';
  camera: {
    lat: number;
    lng: number;
    altitude: number;
    manualAt: number | null;    // last user-initiated pan/zoom
  };
};
```

Rules:

- `previewId = hoveredId ?? focusedId`. It styles the card and arc, but never changes `selectedId` or the camera.
- `selectedId` calls `ensureVisible` only if the route is outside the safe area; it dims other arcs to 15%.
- `mode: 'intel'` does not move the camera. The route is already framed.
- Auto-fit is suppressed for 8 seconds after `camera.manualAt`; show a “Fit route” chip instead.
- Changing a query invalidates results but keeps the old selection at 30% until the new search resolves.
- URL state: `?from=BER&to=LIS&depart=2027-03-01&return=2027-03-08&selected=...`. Shareable and reload-safe.

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
│  /places /routes /search /route-intel /price-check            │
│  /explore /alerts                                             │
│  search → sources → normalizer → ranker → verifier            │
│       ├─ SerpAPI / SearchApi (Google Flights)                 │
│       ├─ Travelpayouts (cached baselines)                     │
│       ├─ Duffel (provider links)                               │
│       ├─ multi-storefront POS screening (gl/hl/currency)      │
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
- **Deploy:** Vercel for the web app and static data dashboard; Rust API, scheduler and optional Obscura in a container (Fly.io/Render/Railway) with SQLite or Postgres. Full split, costs and checklist: [docs/15-hosting-on-vercel.md](docs/15-hosting-on-vercel.md).

---

## 9. Data and sources

### Places

| Dataset | Use | Licence |
|---|---|---|
| [OurAirports](https://ourairports.com/data/) | Airports, coordinates, scheduled service | Public domain |
| [OpenFlights routes](https://openflights.org/data.html) | Demo route graph for idle arcs only; the dataset is from 2014 and is not production-current | ODbL |
| Nominatim / OSM | City search fallback | ODbL; rate-limited |

Filter to airports with scheduled service. Cap idle arcs to the top destinations by traffic.

### Reference data

These are not fare feeds, but the product is wrong without them.

| Dataset | Fields | Source | Licence / cost | When |
|---|---|---|---|---|
| **Time zones** | IANA zone per place | IANA tzdata | Public domain | Now |
| **FX rates** | daily reference rate, base/quote | ECB Data Portal, Frankfurter | Free | Now |
| **Provider registry** | airline/OTA, country, currencies, payment methods, deep-link template, affiliate flag | First-party | Internal | Now |
| **Public holidays** | country, date, name | Nager.Date | MIT | Now |
| **School holidays** | region, start, end | Government calendars, curated | Varies | Launch markets |
| **Fare rules and baggage** | fare family, carry-on, checked, change, cancel, seat, size/weight | Airline pages, community | Curated | Phase 5 |
| **Airport access** | city centre, mode, minutes, cost, frequency | OSM/GTFS + curated | ODbL / varies | Phase 7 |
| **Minimum connection times** | airport, domestic/international, source | Airport/airline MCT + conservative default | Curated | Phase 7 |
| **Availability** | seats left at price, fare class | Search response | Source | Optional |
| **Deal feeds** | error fares, promos | Public sites, RSS, Telegram | Link only | Later (optional) |

### Fares

| Source | Use |
|---|---|
| SerpAPI Google Flights | Live search and verification |
| SearchApi | Cheaper at volume, same surface |
| Travelpayouts | Cached baselines and price calendar |
| Duffel | Offer detail and provider links; the app never books |
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

### Point of sale

One row per storefront, itinerary and observation:

```sql
CREATE TABLE pos_quotes (
  route             TEXT,
  travel_date       TEXT,
  storefront_country TEXT,
  currency          TEXT,
  fare              REAL,
  bags              REAL,
  fees              REAL,
  fx_fee            REAL,
  net_total         REAL,
  status            TEXT,   -- eligible | listed | ineligible
  fare_rules        TEXT,
  observed_at       TEXT,
  expires_at        TEXT,
  PRIMARY KEY (route, travel_date, storefront_country, observed_at)
);
```

The eligibility profile stays on the client or in a local table — residency, nationality, card countries, payment methods. No card numbers, no billing addresses.

### Cache

| Data | TTL |
|---|---|
| Place search | 24 hours |
| Idle route arcs | 24 hours |
| Search results | 10 minutes |
| Offer detail | Until `expires_at`, max 30 minutes |
| Route history | Daily rollup, updated once a day |
| POS quotes | 6 hours |
| Explore feed | 6 hours |

---

## 10. Unique data and visual layers

Inventory is a commodity. Every OTA has the same flights. The edge is **longitudinal** data — how a route's price behaves — and **total-cost** data — what the fare hides. Both are hard for a transaction-monetised incumbent to publish, and both are natural for an open-source project.

Every dataset gets a **data card**: source, licence, coverage, freshness, methodology and known bias. No number appears without provenance.

### The layers

| Layer | Question it answers | Source | Effort | Visual | Order |
|---|---|---|---|---|---|
| **Open fare history** | Should I buy now? | Daily collection + Travelpayouts | M | percentile band, wait-win rate | Launch |
| **Fare weather** | How erratic is this route? | Derived from history | L | volatility colour, expected range | Launch |
| **Seasonality scrubber** | When is this route cheap? | History + cached calendars | M | month slider recolours the globe | Next (phase 6) |
| **Cheap airport trap** | Is the cheap airport actually cheap? | OSM + GTFS + curated transfers | H | isochrones, door-to-door price | Next (phase 7) |
| **True Fare** | What will bags and changes add? | Community-maintained fee data | M | total-cost breakdown | Next (phase 5) |
| **Reliability** | Is it usually on time? | US BTS on-time data | M | on-time badge, adjusted price | Next (phase 7) |
| **Climate overlay** | Is it cheap and pleasant? | Open-Meteo normals | L | “cheap and dry” month view | Next (phase 6) |
| **What people paid** | What did travellers actually pay? | Opt-in submissions | M | community median | Later |
| **Jet-lag cost** | What does the schedule do to me? | Time zones + flight times | L | circadian score | Later |
| **Deal graveyard** | Which error fares survived? | Public deal posts + checks | H | outcome timeline | Later |

### 10.1 Open fare history

The core dataset. One row per route, travel date and observation day; the daily minimum is the signal.

Publish the aggregate:

- p10 / p50 / p90 for every route and lead time.
- The days-to-departure curve.
- How often waiting beat buying.
- The average saving and the worst-case loss.

**FareBench.** A reproducible benchmark built from fixed route/date snapshots: given the history up to day T, recommend buy or wait, then score the result against what happened. Publish the fixtures, the scoring code and the leaderboard. This turns “trust our algorithm” into “here is the data; check it.” It also gives researchers and contributors something concrete to improve.

Data card: coverage, observation count per route, source mix, freshness, and the routes where the sample is too thin to advise.

### 10.2 Fare weather

A visual language for volatility, derived entirely from history:

- **Stable** — narrow band, little movement; buy when convenient.
- **Erratic** — wide band, frequent drops; watch and alert.
- **Rising** — inside the window; buy now.

Compute the coefficient of variation of the daily minimum, the frequency of meaningful drops, and the empirical 7-day range. Show the range, not a single prediction. The point is to make uncertainty legible.

On the globe, colour routes by volatility; on the route page, show the band, the sample size and the track record. No “price will be €X” claims.

### 10.3 Seasonality scrubber

For each route, compute the median price by month and by week. Add a month slider to the globe: drag from January to December and watch the cheap routes move. This is the signature screenshot — and it is genuinely useful for flexible travellers.

With the climate overlay, the query becomes “cheap and dry,” not just “cheap.” Combine a monthly fare percentile with a simple comfort score (temperature, rain) from Open-Meteo normals.

Cold start: seed with Travelpayouts calendars and label thin months as “limited data.” Never present a monthly median from three observations as a seasonal fact.

### 10.4 The cheap airport trap

The most distinctive visual. For each airport, compute 30/60/90-minute reach from the city centre, plus the transfer cost, using OSM and GTFS. Then show:

- the fare,
- the transfer cost,
- the door-to-door time,
- the real price.

A secondary airport that looks €40 cheaper but adds 90 minutes and €25 is not cheaper. Start with the top 50 airports by traffic and let the community add more. Cache the isochrones; they change slowly.

This also gives the globe a second layer: access rings around airports, not just arcs between them.

### 10.5 True Fare

A structured, open dataset of ancillary fees by airline and fare family: carry-on, checked bag, seat selection, change, cancellation, with a source URL and a checked date. It is tedious to build and easy to contribute to, and it directly powers total-cost ranking.

Ship it as YAML in the repo with a schema and CI validation. Treat stale entries as a bug and show “last checked.” Do not guess a fee; mark it unknown.

### 10.6 Reliability

Start with US BTS on-time and cancellation data, which is public. Show on-time rate by route, carrier and departure hour, and compute a **reliability-adjusted price**:

```
adjusted = price / on-time probability
```

or an expected cost that includes a delay allowance. A €70 flight with a 65% on-time rate can be worse than an €85 flight at 90%.

Other markets can follow when an open dataset exists; do not scrape proprietary reliability scores.

### 10.7 Climate overlay

Open-Meteo climate normals, free and global. Compute a simple comfort score from average temperature and rain days, then show the months that are both cheap and pleasant. This is an Explore feature, not a search filter.

### 10.8 What people actually paid

An opt-in, anonymised submission: route, travel date, cabin, bags, total paid, and whether it was a deal. No account, no PII.

This is the one dataset only an open-source community can build. It is also biased — self-reported, skewed toward deal hunters — so label it as a supplement, never as the verified price. Show sample size and the gap versus the API baseline.

### 10.9 Jet-lag cost

A heuristic score from time-zone difference, direction and arrival time. Eastbound red-eyes cost more than westbound day flights. Show it as a secondary badge, not a ranking input by default, and link to the method. It is shareable, human and cheap to compute.

### 10.10 Deal graveyard

Record public deal posts and what happened: price, route, expiry, whether the fare was honoured or cancelled, and how long it lived. Link to the original post; never republish fare content.

This is a unique research dataset and a strong weekly report. It also carries legal and data-quality risk: only public posts, no scraping behind logins, and clear “reported, unverified” labels.

### 10.11 Data completeness check

The product needs to answer ten questions. This is what each one requires, and what is still missing.

| Question | Data required | Status |
|---|---|---|
| **Which flights exist?** | Offers, schedules, airports | Have: SerpAPI, Duffel, OurAirports |
| **What is the total price?** | Fare, taxes, bags, seats, fees, FX | Partial: FX missing; fees in phase 5 |
| **What is the total time?** | Durations, layovers, airport access, MCT | Partial: access and MCT in phase 7 |
| **Should I buy now?** | History, lead-time curve, seasonality, holidays, volatility | Partial: holidays missing; rest planned |
| **Where should I buy?** | POS quotes, provider registry, FX, eligibility | Partial: registry and FX missing |
| **Is it reliable?** | On-time and cancellation rates | Planned: phase 7 |
| **Is it pleasant?** | Climate normals, jet-lag score | Planned: phases 6–7 |
| **Is it a deal?** | Route baseline, deal feeds | Partial: baseline planned; feeds later |
| **Is it eligible?** | Fare rules, baggage policy, payment methods | Partial: fare rules phase 5; registry now |
| **Is it available?** | Seats left at the price | Optional: source-dependent |
| **Which destinations are reachable?** | Current route network by month | Missing: OpenFlights is a 2014 demo; production needs live search or a current schedule source |

**Verdict: six pieces must be added before the core loop is complete.**

1. **Time zones** — local times, day boundaries and jet lag are wrong without them.
2. **FX rates** — the point-of-sale net total and currency comparison are wrong without a dated rate.
3. **Provider registry** — redirects, eligibility and affiliate disclosure need one structured list of airlines and OTAs.
4. **Public and school holidays** — seasonality and demand advice miss the largest price spikes without them.
5. **Fare rules and baggage schema** — “total cost” is a headline fare without carry-on, checked, change and cancel rules.
6. **Airport access and minimum connection times** — door-to-door time and self-transfer risk are guesses without them.

**Defer, but design the schema now:** deal feeds, seat availability, schedule/route seasonality, emissions, points, security wait times, overnight hotel costs. Each is useful; none is required to prove the core loop.

The first four are small additions. The last two are the expensive ones: start with the top 50 airports and a conservative MCT default, then let the community improve them.

**One more gap sits under Explore: route existence.** OpenFlights is a 2014 demo graph. For production, derive reachable destinations from live search history or a current schedule source; otherwise Explore shows routes that no longer operate. Until that source exists, limit Explore to the watchlist corridor rather than drawing a global map from stale data.

Both hard problems — cold-start history and current route coverage — have a build plan: [docs/14-cold-start-and-coverage.md](docs/14-cold-start-and-coverage.md).

---

## 11. Search and ranking

1. Resolve places; expand nearby airports (up to 3 each).
2. Query the air source for the date range and passenger/bag parameters.
3. Normalize every offer into one schema; fingerprint by physical service.
4. Drop duplicates; keep the lowest verified total.
5. For a point-of-sale check, screen three to five eligible storefronts and compute the net total for each, including FX and bags.
6. Rank by generalized cost, using the net total rather than the headline fare:

```
score = total price
      + (value_of_time × door_to_door_hours)
      + stop_penalty
      + risk_premium
```

7. Verify the top N at the source; mark the rest indicative.
8. Return itineraries plus the route snapshot and any eligible storefront comparison.

Defaults: value of time €15/hour; stop penalty €25 per stop; risk premium €40 for a self-transfer and €25 for an OTA-only fare. All adjustable behind “advanced.”

**Best** is the weighted score. **Cheapest** and **Fastest** are separate sorts so the user can see the trade-off.

---

## 12. API contract

```ts
type Place = {
  id: string;              // 'airport:BER'
  code: string;            // IATA
  name: string;
  city: string;
  country: string;
  lat: number;
  lng: number;
  timezone: string;        // IANA, e.g. 'Europe/Berlin'
  metro?: string;          // groups BER with nearby airports
  cityCenter?: { lat: number; lng: number };
};

type FxRate = {
  date: string;
  base: string;
  quote: string;
  rate: number;
  source: 'ECB' | 'other';
};

type Provider = {
  id: string;
  name: string;
  kind: 'airline' | 'ota';
  country: string;
  currencies: string[];
  paymentMethods: string[];
  deepLinkTemplate: string;
  affiliate: boolean;
};

type FareRule = {
  airline: string;
  fareFamily: string;
  carryOn: { included: boolean; dimensions?: string; weightKg?: number };
  checked: { included: number; dimensions?: string; weightKg?: number };
  changeFee: number | null;
  cancelFee: number | null;
  seatFee: number | null;
  source: string;
  checkedAt: string;
};

type Segment = {
  id: string;
  marketingCarrier: string;
  operatingCarrier: string;
  serviceNumber: string;
  from: Place;
  to: Place;
  departTerminal?: string;
  arriveTerminal?: string;
  departLocal: string;     // ISO with offset
  arriveLocal: string;     // ISO with offset
  durationMinutes: number;
  aircraft?: string;
  cabin: 'economy' | 'premium' | 'business' | 'first';
  fareClass?: string;
};

type Itinerary = {
  id: string;
  segments: Segment[];
  totalPrice: number;
  taxesAndFees?: number;
  currency: string;
  totalDurationMinutes: number;
  stops: number;
  fareFamily?: string;
  bagsIncluded: { carryOn: number; checked: number };
  fareRules: { refundable: boolean; changeable: boolean };
  providerUrl: string;   // where the traveller completes the purchase
  source: string;
  verifiedAt: string | null;
  risk: Array<'self-transfer' | 'ota-only' | 'hidden-city'>;
  score?: {
    total: number;
    price: number;
    timeCost: number;
    stopPenalty: number;
    riskPremium: number;
  };
};

type PosQuote = {
  storefrontCountry: string;
  currency: string;
  fare: number;
  bags: number;
  fees: number;
  fxFee: number;
  netTotal: number;
  status: 'eligible' | 'listed' | 'ineligible';
  fareRulesEquivalent: boolean;
  providerUrl: string;   // provider site; the app does not book
  observedAt: string;
};

type EligibilityProfile = {
  residency: string;
  nationality: string;
  cardCountries: string[];
  paymentMethods: string[];
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
  trackRecord: { correct: number; cases: number } | null;
  sampleSize: number;
  expectedChange: { days: number; low: number; high: number } | null;
  backtest: { cases: number; waitWins: number; avgSaving: number; worstLoss: number } | null;
};

// GET  /api/places?q=lis
// GET  /api/routes?origin=airport:BER&limit=20
// POST /api/search
// GET  /api/route-intel?from=BER&to=LIS&date=2027-03-01
// POST /api/price-check   { itineraryId, profile }
// GET  /api/explore?home=BER&month=2027-03
// POST /api/alerts
// GET  /api/fare-weather?from=BER&to=LIS&date=2027-03-01
// GET  /api/seasonality?from=BER&to=LIS
// GET  /api/airport-access?airport=BER
// GET  /api/true-fare?airline=LH&fareFamily=light
// GET  /api/reliability?from=BER&to=LIS
// GET  /api/data-card?dataset=fares
// GET  /api/reference/fx?base=EUR&date=2027-03-01
// GET  /api/reference/holidays?country=DE&year=2027
// GET  /api/reference/providers?airport=BER
// GET  /api/reference/fare-rules?airline=LH&fareFamily=light
// GET  /api/reference/access?airport=BER
```

Every data endpoint returns its sample size, observation window and data-card id. A response without provenance is a bug.

---

## 13. Repository layout

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
   │  ├─ pos/           multi-storefront screening, eligibility, net total
   │  ├─ layers/        fare weather, seasonality, access, fees, reliability
   │  ├─ browser/       optional Obscura pool
   │  ├─ store/         SQLite/Postgres, history, quota
   │  ├─ api/           Axum routes
   │  └─ scheduler/     daily history collection, alerts
   ├─ web/              React + react-globe.gl
   ├─ infra/            Dockerfile, fly.toml, GitHub Actions
   └─ data/
      ├─ airports.json
      ├─ routes.json
      └─ open/          versioned CSV/Parquet exports + data cards
```

---

## 14. Phases and acceptance criteria

The same steps with effort, gates, exact reading and agent assignments: [docs/22-build-planner.md](docs/22-build-planner.md).

| Phase | Build | Done when |
|---|---|---|
| **0 — List-first shell** (1 week) | Search, results, route strip, URL state, tokens, keyboard, price pills, status badges. No globe. | The full search-to-route-summary flow works with WebGL disabled; axe has zero critical issues; 20 cards scroll at 60fps |
| **1 — Globe v0 + sync** (2–3 weeks) | Top 100 airports, selected route, HTML labels, camera fit with a safe rect, poster fallback; hover preview, click selection, arcs deduped by airport pair | 60fps on an M1 Air and 30fps on a mid-range Android; no label overlap at 12 labels; hover never moves the camera; WebGL failure shows the poster and list |
| **2 — Air data** (5–7 days) | Rust API, places, route previews, SerpAPI adapter, cache, quota, verification | Real BER–LIS results with total price, bags and a verified timestamp |
| **3 — History collection** (3–4 days) | Daily scheduler, `fare_history`, percentiles, route snapshot endpoint | A route accumulates daily minima and returns p10/p50/p90 |
| **4 — Buy/wait + fare weather** (5–7 days) | Curve, recommendation rules, track record, backtest, volatility band, route-intelligence panel | A recommendation is shown only with enough data; the backtest, sample and uncertainty band are visible |
| **5 — Point of sale** (4–6 days) | Eligibility profile, three-to-five storefront screening, net total, status labels, comparison table | A route shows an eligible storefront comparison, or explicitly says the home storefront is best |
| **6 — Explore + climate** (4–5 days) | Home airport, cheap-destination feed, seasonality scrubber, climate overlay, destination cards | The globe shows cheap and pleasant destinations by month, with sample-size labels |
| **7 — Airport access + reliability** (5–7 days) | Isochrones for the top 50 airports, transfer cost, on-time data for US routes, adjusted price | A cheap secondary airport is shown with its real door-to-door price and time |
| **8 — Alerts and polish** (4–6 days) | Watchlist, alert thresholds, mobile bottom sheet, accessibility, reduced motion | A saved route notifies on a new low; mobile works one-handed |

Ship phase 1 as an internal demo. Ship phase 2 before investing in the advice engine; without reliable prices the advice is noise.

The unique data layers in section 10 attach to phases 3–7. Do not build a visual layer before its data has a baseline; an empty seasonality slider is worse than no slider.

**Reference data is part of phase 2, not a later cleanup:** time zones, daily FX, the provider registry and public holidays. Fare rules land with phase 5; airport access and MCT with phase 7.

**Start the shadow collector 12 weeks before launch.** Thirty routes, one search a day, daily minima in `fare_history`. The UI can ship while the history accumulates; the buy/wait tab stays in “collecting” until a route has enough observations. See [docs/14-cold-start-and-coverage.md](docs/14-cold-start-and-coverage.md).

---

## 15. Performance, accessibility and testing

### Performance

- First contentful paint under 1.5s; globe under 2.5s.
- Interaction under 100ms for hover and selection.
- 300 arcs / 1,500 points at 60fps.
- No layout shift when results load.
- Pause rendering when hidden.

### Accessibility

- The result list is the primary interface; the globe is `aria-hidden="true"` and not a control surface.
- Every airport, arc, price and status has a list or table equivalent. The route strip is a definition list, not an image.
- Keyboard: `/` focuses search; Up/Down moves focus between cards and previews the arc; Enter selects; Space opens detail; `i` opens “When to buy”; Esc clears.
- `aria-live="polite"` for result counts, selection and price changes. Do not announce hover.
- Charts: keyboard crosshair, `aria-live` value announcements, and a “View as table” toggle.
- Reduced motion disables auto-rotation, camera flight, arc animation and selection rings.
- Colour is never the only signal; price pills print the price and percentile; status badges carry an icon and text.
- Focus ring 2px with 2px offset; focused content is never obscured by the sheet.
- Targets: 48×48dp on mobile, 24×24px minimum on desktop.
- A “list only” toggle for low-power devices and WebGL failure.

### Testing

- **Unit:** great-circle, anti-meridian, fingerprinting, percentiles, ranking, total cost.
- **Component:** combobox, date range, result card, recommendation card, empty/insufficient states.
- **E2E:** search → hover → select → route intel; explore → destination → route intel; mobile sheet.
- **Data:** fixture responses for air; contract test per source.
- **Backtest:** a deterministic fixture history with a known answer.
- **Accessibility:** axe and a keyboard-only pass.

---

## 16. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Advice is wrong or generic | Route-level history, range, sample size, track record, backtest, and an explicit “not enough data” state |
| Cold start | Start 12 weeks early; seed priors from cached and quarterly sources; show watch pages; no advice under 14 observations |
| Explore shows dead routes | Validate every candidate with a live search; show coverage counts; keep Explore inside the corridor until a current route source exists |
| Vercel cannot host the data plane | Vercel for web; Rust API, scheduler and Obscura in a container. See [docs/15-hosting-on-vercel.md](docs/15-hosting-on-vercel.md) |
| Search cost and latency | Cached results first, verify in background; paid calls only for the shortlist |
| Price changes between display and click | Re-price, timestamp, and show “price changed” |
| Google/Kayak/Hopper already do this | Differentiate on auditability, total cost and no dark patterns, not on a magic score |
| Globe is a demo, not a tool | List-first results; globe is the lobby; flat route board below 1.35 R |
| Globe is wrong for short routes | Crossfade to a flat route board; a globe shows one hemisphere and can hide the destination |
| Auto-rotation causes motion sickness and label swim | No auto-rotation; user-controlled motion only; reduced-motion jump |
| WebGL fails or is blocked | Static poster plus the list; “Reload globe”; never a dead canvas |
| Colour-only price or status | Separate price, certainty and status channels; every value prints the price and percentile |
| Stale route graph | Use live search or a current schedule source; limit Explore to the corridor until then |
| Airport/label clutter | HTML labels with budgets, collision tests and priority ranking; hide lower-priority labels |
| Anti-meridian arcs | Split at ±180°; test long routes |
| Scraping/ToS | Licensed sources only for displayed prices; no Google/Southwest/easyJet scraping |
| POS feature becomes residency spoofing | Compare only storefronts the traveller is eligible for; no VPN, fake billing address or borrowed card |
| FX fee erases the POS gap | Compute net total at the card rate; show only gaps above €20 or 5% |
| Listed POS price fails at payment | Label “listed, unconfirmed”; never promise the final price |
| Foreign storefront after-sales | Show the ticketing country and refund path; warn when support is harder |
| Stale FX | Use the dated ECB rate; show the date; never use a marketing rate |
| Missing holiday or school calendar | Label seasonality as partial; add launch markets first; never imply a complete demand model |
| Provider link rot | Registry tests in CI; fail the build on a dead deep-link template |
| Fare rules change | Store `checked_at`; show “unknown” rather than guessing a fee |
| MCT too tight | Use a conservative default; warn on self-transfer; improve with community data |
| Obscura limits | Optional; does not beat active bot protection; keep a fallback |
| Data licensing | Attribute OurAirports, OpenFlights, OSM |

---

## 17. Decisions

### Decided

- **Flights only.**
- **Data layer:** Rust, one workspace.
- **Browser:** Obscura optional, embedded, pinned; not required for v1.
- **Frontend:** TypeScript, React, `react-globe.gl`.
- **Commerce:** redirect only. No checkout, payment, ticketing, changes or refunds.
- **Advice:** empirical distributions, no ML, no advice without enough history.
- **Point of sale:** licensed search plus official storefronts; eligibility-only, no spoofing.
- **Reference data:** IANA time zones, ECB daily FX, Nager.Date holidays, first-party provider registry.
- **Hosting:** Vercel for web and static dashboard; Rust API/scheduler/Obscura in a container with SQLite or Postgres.
- **Globe:** the lobby, not the desk. List-first; flat route board below 1.35 R; no auto-rotation; no textures, stars or city lights.
- **Name:** Fare Desk.
- **Visual direction:** low-chroma paper globe in light mode, night globe in dark; no satellite imagery, stars or city lights.
- **Licence:** GPL-3.0 with a CLA; Fare Desk trademark policy; fare data and the hosted service remain proprietary.

### Still open

1. **Home market:** which airport or country is the first-class citizen? The advice engine needs a concentration of routes.
2. **Live or fixtures:** fixtures for the interaction, or live SerpAPI from the start?
3. **Advice policy:** how aggressive should Book now be below p10? How much weight on refundability?
4. **POS markets:** which three to five storefronts to screen by default, and whether the traveller can add their own.
5. **POS threshold:** €20 or 5% — or a different rule for premium cabins.
6. **Explore route network:** derive destinations from live search history, or license a current schedule source?
7. **School holiday coverage:** which countries at launch; the data is manual and uneven.
8. **MCT source:** use a conservative default, or curate airport-level values from the start?
9. **Deal feeds:** include error fares and promos in v1, or keep the product purely search and timing?
10. **Budget:** SerpAPI cap and hosting.

---

## 18. Standout open-source playbook

If this were run like a standout open-source launch — the OpenClaw/Eyes of God instinct — the app would be the demo and the **data would be the contribution**. The goal is not another flight search box; it is the reference implementation for “when should I buy?” plus a public dataset nobody else publishes.

### Make the demo the distribution

- **One command, no keys.** `docker compose up` shows the globe, an Explore view and a fixture search. Live data is opt-in BYOK. If a contributor needs a SerpAPI key to see the product, most will never see it.
- **A hosted demo.** Rate-limited, clearly labelled, reset weekly. It is the link in every post.
- **One 15-second GIF.** The month scrubber or the cheap-airport trap. Not a feature list.
- **A “god view.”** One global screen that makes the whole system legible: fare weather, seasonality, anomalies, cheap routes from a home airport. The Eyes of God instinct is right — one view that explains a complex system is worth ten settings panels.

### Make the data the contribution

- Publish the fare history and the backtest, not just the app. CSV/Parquet, versioned, with a data card.
- Build **FareBench**: fixed snapshots, a buy/wait task, a scoring harness, a leaderboard. This is the thing researchers cite and contributors improve.
- Publish **True Fare** (ancillary fees) and **airport access** as community datasets with source and checked dates.
- Run a weekly **“State of Airfare”** report generated from the data: biggest drops, routes where waiting won, fee changes. Shareable charts, no app required.
- Every claim links to its evidence. “This route is 41% below median” should open the observations.
- Ship **primitives**, not just an app: a CLI (`fleet weather BER LIS`), an MCP server (`should_i_buy`), and an embeddable fare-weather widget for travel bloggers. Each is a distribution surface.

### Make the complex legible

- A **“why is this cheap?” receipt** for every itinerary: structure, point of sale, timing, fee breakdown, reliability, access time. Shareable as an image or a link.
- A **route anatomy** view: arcs, stops, layovers, transfer, door-to-door time, total cost.
- A **month scrubber** and a **volatility layer** that turn a table of fares into a picture of behaviour.
- A **list-only mode** for accessibility and low-power devices. The globe is the hook; the list is the product.

### Build in public with receipts

- A short devlog every week: what shipped, what broke, what the data showed. Link the commit.
- Public roadmap with votes, and a changelog that reads like a story.
- Publish the failures: “waiting advice lost money on 38% of routes” is more credible than a highlight reel.
- Launch with a **finding**, not a manifesto. “We analysed N fare observations and found X” travels further than “we built a flight app.”

### Security is the product

The OpenClaw pattern — a local-first tool that installs in one command — is the distribution model to copy. The cautionary half is that a viral tool which executes code needs sandboxing, scoped permissions and a visible security model from day one.

- Obscura runs untrusted page JavaScript in-process: container, non-root, read-only root, egress allowlist, no credentials.
- No API keys in the browser. No keys in logs. No keys in the repo.
- Signed releases, SBOM, `cargo audit`, pinned dependencies, a `SECURITY.md` with a disclosure path.
- Plugins are declarative source manifests, not arbitrary code. A community adapter should not be able to read the filesystem.
- A visible “what this app can and cannot do” page: no booking, no payment, no ticket issuance.

### Community loops

- A **source SDK** and fixture harness, so adding an adapter is a small PR, not a fork.
- **Good first issues** that are real: airport metadata, a fee entry, a translation, a test fixture.
- **Data bounties**: “add the 20 missing airport transfer costs for Italy,” credited in the release notes.
- Contributor credits and a `CONTRIBUTORS.md`; a monthly “data hero.”
- Discussions for ideas, issues for bugs. Do not let the issue tracker become a wish list.

### Sustainability without paid ranking

- BYOK for self-hosters; a rate-limited hosted demo; sponsorship or a paid hosted tier.
- Affiliate links are disclosed and **never** affect ranking. Publish the ranking formula and the backtest so anyone can check.
- Choose the licence deliberately: Apache-2.0 for adoption, AGPL-3.0 if a hosted competitor must share changes. Docs under CC BY 4.0; data under its original licences.
- DCO rather than a CLA. A CLA suppresses casual contributors.

### Scope discipline

- Flights only. One home airport or corridor first: roughly 30 routes and 60 days of real observations before going global. One command.
- No accounts, no checkout, no mobile app, no custom ML model in v1.
- Every new data layer must answer a question the user already has. “Cool” is not a question.

### Launch checklist

- [ ] `LICENSE`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `ROADMAP.md`
- [ ] Demo mode with fixtures; hosted demo
- [ ] 15-second GIF and a one-paragraph README pitch
- [ ] A finding from the first dataset, with a methodology link
- [ ] FareBench fixtures and a scoring command
- [ ] Data cards for every published layer
- [ ] CI: fmt, clippy, test, frontend build, Playwright, license check
- [ ] Ten good first issues
- [ ] No keys, no fare republishing, no paid placement

---

## 19. The first commit

- Vite + React + TypeScript scaffold.
- `react-globe.gl` with a textured globe, 1,000 airports and 500 static arcs.
- Hover and click handlers that log the selected airport.
- A README with the run command.

No panel, no API, no advice engine. The first question is whether the globe feels right.
