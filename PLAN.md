# Fleet Atlas — flight finder with buy/wait intelligence

A 3D globe of airports and routes, a search console on the left, and a route page that tells you whether to buy now or wait — using that route's own price history, not a generic “book 39 days out” rule.

**Scope:** flights only. Trains and coaches are future work, not v1.

**Status:** plan only. No implementation yet.

---

## 1. The product in one paragraph

Fleet Atlas is a discovery and redirection product: it finds the best flights and the best time to buy, then sends the traveller to the airline or OTA to complete the purchase. It does not sell tickets, take payment, or handle changes and refunds.

A traveller opens Fleet Atlas, types “Berlin” and “Lisbon,” picks a week in March, and sees the best flights ranked on total cost. The globe draws the candidate routes as arcs; hovering a result lifts its arc, clicking it frames the route and dims the rest. A second tab answers the harder question: *should I buy now?* It shows the route's price history, where today's fare sits in the distribution, what usually happens between now and departure, and a plain recommendation — buy, wait, or watch — with a confidence level. When a route looks good, it also checks whether another country storefront the traveller is eligible for beats the home price after FX and bags. A third mode, Explore, turns the globe into a map of cheap places from your home airport.

---

## 2. Goals and non-goals

### Goals

- Find the best flight for a specific trip, where “best” means total cost and total time, not headline fare.
- Tell the traveller when to buy, with route-level evidence and an honest confidence level.
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

### Commerce model

Fleet Atlas is not a travel agency, OTA or merchant of record. It does not sell, issue, change or refund tickets, and it never takes payment. Every result links to the provider — the airline, the OTA, or the operator — and the transaction happens there.

- The displayed price is verified at the time of display; the final price and availability are confirmed on the provider's site.
- The provider's fare rules, payment requirements and after-sales terms govern the purchase.
- Affiliate links, if used, are disclosed.
- The app never auto-fills residency or billing details, and never books on the traveller's behalf.

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
- **Provider options:** a short comparison of eligible storefronts with net totals and Eligible / Listed / Ineligible status. Hidden unless a material, persistent gap exists. Each row links out; the purchase happens on the provider's site.
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

#### Rules

- Below p10 → default BUY, unless the fare rules are bad for the traveller's flexibility.
- Between p10 and p50 → WATCH, with an alert at the p10.
- Above p50 and more than 45 days out → WAIT, with a re-check date.
- Inside 14 days → BUY or WATCH; do not promise a late drop.
- Less than 14 observations or no comparable lead-time history → no advice. Say “collecting baseline.”
- Refundability changes the call: a slightly higher flexible fare can be the better buy.
- Always show the total with bags and fees.

#### Confidence

Confidence reflects sample size, dispersion and how well the current price fits the historical pattern. It is not a probability that the advice is correct. State that in the UI.

#### Backtest

For every route with enough history, replay the advice at 60, 30 and 14 days out and compare it with buying at that moment. Publish:

- share of cases where waiting beat buying,
- average saving or loss,
- worst-case loss.

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

## 10. Search and ranking

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
  providerUrl: string;   // where the traveller completes the purchase
  source: string;
  verifiedAt: string | null;
  risk: Array<'self-transfer' | 'ota-only' | 'hidden-city'>;
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
  confidence: number | null;
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
   │  ├─ pos/           multi-storefront screening, eligibility, net total
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
| **5 — Point of sale** (4–6 days) | Eligibility profile, three-to-five storefront screening, net total, status labels, comparison table | A route shows an eligible storefront comparison, or explicitly says the home storefront is best |
| **6 — Explore** (3–4 days) | Home airport, cheap-destination feed, arc colouring, destination cards | The globe shows the cheapest destinations from one airport with percentile labels |
| **7 — Alerts and polish** (4–6 days) | Watchlist, alert thresholds, mobile bottom sheet, accessibility, reduced motion | A saved route notifies on a new low; mobile works one-handed |

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
| POS feature becomes residency spoofing | Compare only storefronts the traveller is eligible for; no VPN, fake billing address or borrowed card |
| FX fee erases the POS gap | Compute net total at the card rate; show only gaps above €20 or 5% |
| Listed POS price fails at payment | Label “listed, unconfirmed”; never promise the final price |
| Foreign storefront after-sales | Show the ticketing country and refund path; warn when support is harder |
| Obscura limits | Optional; does not beat active bot protection; keep a fallback |
| Data licensing | Attribute OurAirports, OpenFlights, OSM |

---

## 16. Decisions

### Decided

- **Flights only.**
- **Data layer:** Rust, one workspace.
- **Browser:** Obscura optional, embedded, pinned; not required for v1.
- **Frontend:** TypeScript, React, `react-globe.gl`.
- **Commerce:** redirect only. No checkout, payment, ticketing, changes or refunds.
- **Advice:** empirical distributions, no ML, no advice without enough history.
- **Point of sale:** licensed search plus official storefronts; eligibility-only, no spoofing.

### Still open

1. **Home market:** which airport or country is the first-class citizen? The advice engine needs a concentration of routes.
2. **Live or fixtures:** fixtures for the interaction, or live SerpAPI from the start?
3. **Visual direction:** realistic Blue Marble or a stylised dark globe.
4. **Advice policy:** how aggressive should BUY be below p10? How much weight on refundability?
5. **POS markets:** which three to five storefronts to screen by default, and whether the traveller can add their own.
6. **POS threshold:** €20 or 5% — or a different rule for premium cabins.
7. **Budget:** SerpAPI cap and hosting.
8. **Name:** “Fleet Atlas” is a working title.

---

## 17. The first commit

- Vite + React + TypeScript scaffold.
- `react-globe.gl` with a textured globe, 1,000 airports and 500 static arcs.
- Hover and click handlers that log the selected airport.
- A README with the run command.

No panel, no API, no advice engine. The first question is whether the globe feels right.
