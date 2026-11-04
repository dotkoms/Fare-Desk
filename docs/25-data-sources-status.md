# Data sources: what is wired, what works, what comes next

**As of 24 October 2026**, checked against the running data plane with no API keys. The rule since this date: **Fare Desk must work on first local run with no keys or signups.** Keyed sources may exist only as opt-in.

---

## 1. Every source we have, and its state

### Fares

| Source | Key? | State | Notes |
|---|---|---|---|
| **Kiwi.com MCP** (`mcp.kiwi.com`) | No | **Working, default** | Full itineraries incl. connections, party + bag pricing, Kiwi.com booking links. Protected connections only (Kiwi does not flag self-transfers). No fare rules. |
| **Skiplagged MCP** (`mcp.skiplagged.com/mcp`) | No | **Working, merged with Kiwi** | Standard nonstop fares beside Kiwi's; the cheaper seller wins per flight. Hidden-city and self-transfer fares excluded; connections skipped (no connection airport in the data). Also powers the flex calendar. |
| SerpAPI (Google Flights) | **Yes, paid** | Off by choice | Code works and is tested; used only when a request names `source=serpapi` and `SERPAPI_KEY` is set. Rejected as default because it needs a key. |
| Demo fixtures | No | **Removed** | Compile only for tests; the server never serves them. |

### Reference and context

| Source | Key? | State | Why it looked broken, and what changed |
|---|---|---|---|
| OurAirports + mwgg timezones | No | Working (bundled) | Now carries airport size for search ranking. |
| ECB rates via Frankfurter | No | Working | — |
| Nager.Date holidays | No | Working | — |
| Open-Meteo climate | No | Working | — |
| OSRM driving legs | No | Working | e.g. Manhattan → JFK 34 min. |
| Transitous public transit | No | Working in Europe; **no New York coverage** | Berlin → BER returns routes; Manhattan → JFK returns none because Transitous has no NYC feeds. The app said "could not parse"; it now says "Transitous has no public-transit route here". |
| Curated airport access + MCT (`data/access.json`) | No | Working, **19 airports** | Expanded with LGA, EWR, LAX, ORD, SFO, BOS (NY corridor + major US hubs). Still curated estimates. |
| BTS on-time (reliability) | No | **Working once imported** | It was empty because nobody had run the monthly import, and the first download hit a transient network error with no retry. Downloads now retry three times; August and July 2026 are loaded (6,134 routes). JFK–LAX: AA 75%, DL 66%, B6 60%. |
| `fare_history` → When to buy, seasonality, fare weather | No | Working, **empty** | Needs the daily collector: 14 observations per route for a verdict, 5 per month for seasonality. Day 1 of the New York corridor was recorded (29/30 routes). |
| Point-of-sale storefront check | — | **Not meaningful now** | Built to price SerpAPI in several country storefronts. Kiwi and Skiplagged each sell from one storefront, so there is nothing to compare. |
| Explore | No | Working, thin | Prices destinations from routes already in history. |

### The biggest reason things "don't show up"

Most of the backend features above work but **the web app never calls them**. Explore, seasonality, fare weather, door-to-door totals, reliability, climate, alerts and the storefront check have endpoints and tests but no screen. The app shows search, When to buy and, as of today, the cheaper-days strip.

---

## 2. Game plan for more sources

Ordered by value per effort, keyless first.

| # | Source | Key? | What it adds | Effort | Notes |
|---|---|---|---|---|---|
| 1 | **Daily collector schedule** | No | Fills When to buy, seasonality and fare weather | done | Example crontab in `app/scripts/crontab-example.txt`. Docker collect profile already exists. |
| 2 | **US DOT Consumer Airfare Report** | No | Quarterly average fare per city pair: a labelled baseline for When to buy before our own 14 days exist | done | Download script + `faredesk-airfare-ingest` binary + `GET /api/airfare-baseline`. |
| 3 | **BTS T-100 segment data** | No | Which US routes fly and how busy they are: a real route list for Explore and coverage | done | `faredesk-t100-ingest` binary + routes endpoint supplements with T-100 destinations. |
| 4 | **Kiwi native round trips** | No | True return prices (can beat two one-ways) in one call | done | `search_round_trip` maps inbound leg; full RT price on outbound. Falls back to two one-ways. |
| 5 | **Skiplagged connections** | No | One-stop fares from Skiplagged | done | One-stop results are now included when maxStops ≥ 1, with an explicit warning that the connection airport is not named in the Skiplagged card. |
| 6 | **Skiplagged "anywhere"** | No | Explore candidates beyond our history | done | `SkiplaggedSource::anywhere` + `GET /api/anywhere`. May return empty; provenance notes when it does. |
| 7 | **Wider airport access table** | No | Transfers and MCTs for the New York corridor airports | 1 day | Curated; label it so. |
| 8 | **OpenSky** (anonymous) | No | Observed flights to sanity-check that a route is flying | notes | Brief notes in `app/scripts/opensky-notes.md`. Rate-limited free tier. |
| 9 | **BTS DB1B** | No | 10% sample of real US ticket prices per quarter | 2 days | Large files; good priors and a check on our verdicts. |

**Opt-in, keyed (never default):** SerpAPI or SearchApi (Google Flights), Travelpayouts (cached cheap fares), Duffel (exact bags and fare rules per offer).

**Not planned:** fast-flights and the unofficial Ryanair API. They need no key but break with every site change and go against the sites' terms (see docs/03).

---

## 3. Surfacing what already works

Before more sources, the app should show what the backend already knows:

1. Reliability on the itinerary (on-time rate and the adjusted price). **Surfaced** in the intel tab via ContextPanel.
2. Seasonality and fare weather in When to buy. (API ready; UI still basic)
3. Door-to-door total on the itinerary. **Surfaced** in the intel tab via ContextPanel.
4. Explore as a mode on the globe. (API ready)
5. DOT airfare baseline. **Surfaced** in the intel tab via ContextPanel.
