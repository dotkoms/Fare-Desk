# Build planner

A step-by-step build order with effort, gates and the exact docs and agents to consult at each step.

**Assumptions:** one focused developer, flights-only v1, the architecture in [PLAN.md](../PLAN.md). Effort is **person-days**, not calendar days. Agents compress research and review; they do not write the integration.

**Rule:** do not start a step until the previous gate is met. If a gate fails, cut scope, not quality.

---

## 1. Effort at a glance

| Step | Effort | Share of build | Gate |
|---|---:|---:|---|
| 0 — Decisions and corridor | 3–5 days | 5% | Corridor, budget, licence and success metric written down |
| 1 — List-first shell | 5–7 days | 8% | Search-to-route-summary works with WebGL off |
| 2 — Globe v0 + sync | 10–15 days | 16% | 60fps desktop / 30fps mobile; hover never moves camera |
| 3 — Air data + reference data | 10–14 days | 15% | Real route returns a verified total |
| 4 — History collection | 3–5 days | 5% | A route has 14 daily observations and percentiles |
| 5 — Buy/wait + fare weather | 7–10 days | 11% | Advice only with enough data; backtest visible |
| 6 — Point of sale | 5–7 days | 8% | Eligible storefront comparison, or “home is best” |
| 7 — Explore + climate | 5–7 days | 8% | Validated destinations with coverage count |
| 8 — Airport access + reliability | 7–10 days | 11% | Door-to-door price and on-time badge |
| 9 — Alerts + mobile + accessibility | 5–7 days | 8% | Alert caps, one-handed mobile, axe clean |
| 10 — Launch + OSS | 3–5 days | 5% | One-command demo and launch checklist |
| **Total** | **63–92 days** | **100%** | **13–18 focused weeks** |

**Do not spend more than 20% of the build on the globe.** It is the hook; the list, the data and the advice are the product.

### Where the effort should go

| Area | Share | Why |
|---|---:|---|
| Data, adapters, storage | 35% | The product dies on stale or incomplete prices |
| UI shell, globe, route board | 25% | The interaction is the demo; cap the globe at 15–20% |
| Advice, history, backtest | 20% | The defensible part |
| Accessibility, mobile, polish | 10% | Trust and reach |
| Launch, docs, community | 10% | An open-source project needs a front door |

---

## 2. The steps

### Step 0 — Decisions and corridor (3–5 days)

**Build:** choose the launch corridor (30 routes from one home airport), the visual direction, the monthly API budget, the licence, and the one success metric. Write them in `decisions.md`.

**Read first:**

- [PLAN.md](../PLAN.md) §15–§16 (decisions, first commit)
- [00-short-version.md](00-short-version.md)
- [05-when-to-buy.md](05-when-to-buy.md)
- [20-users-and-competitors.md](20-users-and-competitors.md)
- [03-public-apis-vs-scraping.md](03-public-apis-vs-scraping.md)

**Agents:** [desk-ops](../agents/desk-ops.md) for a sample trip brief; [air-scout](../agents/air-scout.md) for the source shortlist. Do not use the rail, coach, arbitrage or points agents in v1.

**Gate:** a one-page decision record with the corridor, budget, licence, visual direction and the metric you will judge the launch by. Start the shadow collector now.

---

### Step 1 — List-first shell (5–7 days)

**Build:** search console, place combobox, date range, passenger/bag selector, result cards, route strip, URL state, design tokens, keyboard path and price pills. **No globe.**

**Read first:**

- [16-ui-ux.md](16-ui-ux.md)
- [17-design-system.md](17-design-system.md)
- [18-color.md](18-color.md)

**Agents:** none. This is interface craft; do not delegate it.

**Gate:** the full search-to-route-summary flow works with WebGL disabled; axe has zero critical issues; 20 cards scroll at 60fps; a user can read the total with one checked bag without opening anything.

---

### Step 2 — Globe v0 + sync (10–15 days)

> If the 2D poster map in [23-2d-poster-map.md](23-2d-poster-map.md) is approved, this step becomes its Step B–E and the globe references below are superseded by that document.

**Build:** `react-globe.gl`, Natural Earth land polygons, top 100 airports, direct arcs, one-stop paths, HTML labels, camera fit with a safe rect, poster fallback, hover preview, click selection, `ensureVisible`, keyboard focus and `aria-live`.

**Read first:**

- [21-globe-implementation.md](21-globe-implementation.md)
- [17-design-system.md](17-design-system.md) (globe vs flat, labels, motion)
- [18-color.md](18-color.md) (paper/night globe, selection)
- [16-ui-ux.md](16-ui-ux.md) (interaction model, accessibility)
- [PLAN.md](../PLAN.md) §4 and §7

**Agents:** none for the scene. If you need a design review, use the `ui-ux-designer` system agent only after the first build; it is not part of this repo.

**Gate:** 60fps on an M1 Air, 30fps on a mid-range Android; no label overlap at 12 labels; hover never moves the camera; WebGL failure shows the poster and list; a one-stop itinerary renders as a path, not a direct arc.

---

### Step 3 — Air data + reference data (10–14 days)

**Build:** Rust Axum API, `/places`, `/routes`, `/search`; SerpAPI adapter; cache; quota broker; verification; time zones; FX; provider registry; public holidays; error and freshness states.

**Read first:**

- [02-data-sources.md](02-data-sources.md)
- [03-public-apis-vs-scraping.md](03-public-apis-vs-scraping.md)
- [12-rust-data-layer.md](12-rust-data-layer.md)
- [15-hosting-on-vercel.md](15-hosting-on-vercel.md)
- [PLAN.md](../PLAN.md) §9 and §11

**Agents:** [air-scout](../agents/air-scout.md) for the source contract and candidate shapes; [fare-verifier](../agents/fare-verifier.md) for verification rules. Do not use Obscura unless a source genuinely needs a browser.

**Gate:** a real BER–LIS search returns a total with bags and a verified timestamp; the API returns a data-card id and freshness; a dead source degrades to cached results, not a blank screen.

---

### Step 4 — History collection (3–5 days)

**Build:** scheduler, `fare_history`, daily minima, p10/p50/p90, route snapshot endpoint, observation counts, “collecting baseline” state.

**Read first:**

- [14-cold-start-and-coverage.md](14-cold-start-and-coverage.md)
- [09-build-order.md](09-build-order.md)
- [PLAN.md](../PLAN.md) §9 and §10

**Agents:** [watchkeeper](../agents/watchkeeper.md) for cadence, dedupe and alert budget.

**Gate:** a route has at least 14 daily observations; the snapshot returns percentiles and sample size; re-running the collector does not duplicate rows.

---

### Step 5 — Buy/wait + fare weather (7–10 days)

**Build:** days-to-departure curve, recommendation states (Book now / Wait / Watch / Not enough data), range, sample, track record, backtest, volatility band, route-intelligence panel, “what would change our mind”.

**Read first:**

- [05-when-to-buy.md](05-when-to-buy.md)
- [19-gamification.md](19-gamification.md)
- [20-users-and-competitors.md](20-users-and-competitors.md)
- [PLAN.md](../PLAN.md) §6.2

**Agents:** [watchkeeper](../agents/watchkeeper.md), [fare-verifier](../agents/fare-verifier.md), [desk-ops](../agents/desk-ops.md) for the ranking and explanation.

**Gate:** no advice under 14 observations; the track record includes misses; no raw “78% confidence”; a user can explain why the app said book or wait; the backtest is visible.

---

### Step 6 — Point of sale (5–7 days)

**Build:** eligibility profile, three-to-five storefront screening, net total, Eligible / Listed / Ineligible labels, provider comparison table. No VPN or fake residency.

**Read first:**

- [13-point-of-sale.md](13-point-of-sale.md)
- [03-public-apis-vs-scraping.md](03-public-apis-vs-scraping.md)
- [PLAN.md](../PLAN.md) §6.3

**Agents:** [fare-verifier](../agents/fare-verifier.md) and [desk-ops](../agents/desk-ops.md).

**Gate:** a route shows an eligible storefront comparison, or explicitly says the home storefront is best; no “listed” price is presented as guaranteed; the FX fee is in the total.

---

### Step 7 — Explore + climate (5–7 days)

**Build:** home airport, validated destination set, top 12 arcs, top 8 price chips, month scrubber, climate overlay, coverage count, last-checked time. No stale route graph.

**Read first:**

- [PLAN.md](../PLAN.md) §10 (unique data)
- [14-cold-start-and-coverage.md](14-cold-start-and-coverage.md)
- [18-color.md](18-color.md) (price classes on the globe)

**Agents:** [air-scout](../agents/air-scout.md) for candidates; [watchkeeper](../agents/watchkeeper.md) for freshness.

**Gate:** every drawn route is validated by a live search; the page shows coverage and freshness; a destination opens its route page and Back restores the camera.

---

### Step 8 — Airport access + reliability (7–10 days)

**Build:** 30/60/90-minute isochrones for the top 50 airports, transfer cost, door-to-door price, minimum connection times, US BTS on-time data, reliability-adjusted price.

**Read first:**

- [PLAN.md](../PLAN.md) §10.4 and §10.6
- [14-cold-start-and-coverage.md](14-cold-start-and-coverage.md)
- [05-when-to-buy.md](05-when-to-buy.md) (fixed-date risk)

**Agents:** [air-scout](../agents/air-scout.md) for access and schedule data; [fare-verifier](../agents/fare-verifier.md) for total-cost checks.

**Gate:** a secondary airport is shown with its real door-to-door price and time; the reliability badge is sourced and dated; MCT warnings appear on self-transfers.

---

### Step 9 — Alerts + mobile + accessibility (5–7 days)

**Build:** watchlist, threshold alerts, caps (one per route/week, three per user/week), quiet hours, email digest, mobile three-detent sheet, keyboard path, list-only mode, reduced motion.

**Read first:**

- [16-ui-ux.md](16-ui-ux.md)
- [17-design-system.md](17-design-system.md)
- [19-gamification.md](19-gamification.md)
- [PLAN.md](../PLAN.md) §15

**Agents:** [watchkeeper](../agents/watchkeeper.md) for alert logic and dedupe.

**Gate:** an alert is verified at the source, capped and quiet-hours aware; mobile works one-handed; axe is clean; a keyboard-only user can search, select, read the recommendation and open the provider.

---

### Step 10 — Launch + open source (3–5 days)

**Build:** demo mode with fixtures, hosted demo, `LICENSE`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `ROADMAP.md`, data cards, product-first README, 10 good first issues, launch post with a finding.

**Read first:**

- [PLAN.md](../PLAN.md) §18 (standout open-source playbook)
- [20-users-and-competitors.md](20-users-and-competitors.md)
- [11-sources.md](11-sources.md)
- [README.md](../README.md)

**Agents:** [desk-ops](../agents/desk-ops.md) for a final trip-brief demo; [air-scout](../agents/air-scout.md) for a data sanity pass.

**Gate:** `docker compose up` shows the globe and a fixture search without keys; the hosted demo works; the launch post links the data and method; no secret is in the repo.

---

### Optional — 2b Obscura collector (3–5 days)

Only if a required source has no API and needs JavaScript.

**Build:** embedded Obscura pool, JSON-response capture, sandbox, egress allowlist, fallback when the browser fails.

**Read first:**

- [12-rust-data-layer.md](12-rust-data-layer.md)
- [15-hosting-on-vercel.md](15-hosting-on-vercel.md)
- [PLAN.md](../PLAN.md) §4 and §15

**Agents:** [air-scout](../agents/air-scout.md) for source selection.

**Gate:** one browser-bound source produces structured offers without DOM parsing; the API still serves cached data if the browser fails; no credentials or general internet access in the sandbox.

---

## 3. Parallel workstreams

| Workstream | When | Owner | Why |
|---|---|---|---|
| **Shadow collection** | Start at step 0; run 12 weeks | `watchkeeper` + cron | Without history there is no advice |
| **Data cards and methodology** | From step 3 | You | Trust and open-source credibility |
| **Accessibility pass** | Step 1, then every step | You | Retrofitting is expensive |
| **OSS front door** | Step 10, or earlier | You | The repo is the distribution |
| **Backtest** | Step 5 and after | `fare-verifier` + you | The only proof the advice works |

---

## 4. Agent map

| Agent | Use it for | Do not use it for |
|---|---|---|
| [desk-ops](../agents/desk-ops.md) | Trip briefs, ranking, final demo, orchestration | Writing code or inventing prices |
| [air-scout](../agents/air-scout.md) | Source contracts, candidate offers, Explore candidates | Final quotes |
| [fare-verifier](../agents/fare-verifier.md) | Re-pricing rules, total cost, risk flags | Discovery |
| [watchkeeper](../agents/watchkeeper.md) | History cadence, dedupe, alert budget | UI layout |
| [rail-scout](../agents/rail-scout.md) / [coach-scout](../agents/coach-scout.md) | Later expansion only | v1 |
| [arbitrage-analyst](../agents/arbitrage-analyst.md) | Later itinerary structures | v1 |
| [points-strategist](../agents/points-strategist.md) | Later points work | v1 |

A useful rule: the agents write **research and checks**, not the price path. Deterministic code extracts and verifies.

---

## 5. Calendar

| Week | Work |
|---|---|
| **-12** | Decisions, corridor, shadow collector starts |
| **-11 to -10** | List-first shell |
| **-10 to -7** | Globe v0 + sync |
| **-7 to -5** | Air data + reference data |
| **-5** | History collection |
| **-5 to -4** | Buy/wait + fare weather |
| **-4 to -3** | Point of sale |
| **-3 to -2** | Explore + climate |
| **-2 to -1** | Airport access + reliability |
| **-1** | Alerts, mobile, accessibility |
| **0** | Launch and open-source front door |

This is a 12-week target with limited parallel work (design, data and docs). Strictly solo and sequential, use the 13–18 week range. Add 20–30% for integration surprises, platform review and the first real user feedback.

---

## 6. What not to do

- Do not build the globe before the list shell.
- Do not start buy/wait before the route has 14 observations.
- Do not add trains, coaches, points or packages to v1.
- Do not add booking or payment.
- Do not spend more than 20% on the globe.
- Do not launch with a global map built from a 2014 route file.
- Do not let an agent invent a fare.
- Do not treat “no clear signal” as a bug; it is a product state.

---

## 7. Definition of done

- A traveller can search a route, read the total with bags, understand whether to book or wait, and open the provider.
- Every displayed price is verified and timestamped.
- No advice appears without enough history.
- The globe is fast, optional and accessible.
- The app does not sell, book or take payment.
- `docker compose up` shows the product without keys.
- The data cards, methodology and backtest are public.
