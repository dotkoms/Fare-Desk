# Fare Desk data plane

Rust workspace for places, fares and reference data. The React app in [`web/`](web) is the interface; this service is the thing it asks.

```
app/
├─ Cargo.toml            workspace
├─ crates/
│  ├─ core/              contract types, pricing, fingerprint, ranking
│  ├─ store/             embedded reference data, TTL cache, SQLite ledger
│  ├─ sources/           fixtures, SerpAPI, FX and holiday clients
│  ├─ scheduler/         daily fare-history collector
│  └─ api/               Axum HTTP service
├─ data/
│  ├─ airports.json      3,177 airports with IANA timezones (generated)
│  └─ providers.json     airline and OTA deep-link registry
├─ scripts/
│  └─ build-airports.mjs regenerates airports.json
├─ web/                  React + react-globe.gl
└─ .env.example
```

## Run it

```bash
cd app
cp .env.example .env          # optional; every value has a default
cargo run -p faredesk-api
```

The service listens on `http://127.0.0.1:8080`. With no `SERPAPI_KEY` it answers from the bundled fixture set — a full BER–LIS search, correctly priced for the party and bags, labelled as a fixture.

The web app calls this service in development (`VITE_API_BASE`, default `http://127.0.0.1:8080`). If it is not running, the app falls back to its own bundled fixtures, says so above the results, and keeps working offline.

```bash
curl -s "http://127.0.0.1:8080/api/places?q=lis"
curl -s "http://127.0.0.1:8080/api/routes?origin=BER"
curl -s -X POST "http://127.0.0.1:8080/api/search" \
  -H 'content-type: application/json' \
  -d '{"origin":"BER","destination":"LIS","departDate":"2027-03-01","adults":1,"checkedBags":1}'
```

## Endpoints

| Method | Path | Returns |
|---|---|---|
| `GET` | `/healthz` | Version, uptime, reference size, cache size, source configuration and today's quota |
| `GET` | `/api/places?q=&limit=` | Airports ranked by IATA code, city, name, then country |
| `GET` | `/api/routes?origin=&limit=` | Destinations covered from an airport, with coverage state |
| `GET` | `/api/route-intel?from=&to=&date=` | Daily minima, p10/p50/p90, current percentile, sample and observation window |
| `GET` | `/api/seasonality?from=&to=` | Median, p10 and p90 per travel month |
| `GET` | `/api/fare-weather?from=&to=&date=` | Volatility: coefficient of variation, last-seven range, drop frequency, classification |
| `GET` | `/api/explore?home=&month=` | Cheapest destinations from a home airport for a month, with coverage |
| `POST` | `/api/search` | Priced, deduplicated, ranked itineraries with provenance |
| `POST` | `/api/price-check` | Storefront screen: eligibility, net totals with FX, status labels and a recommendation |
| `GET` | `/api/reference/fx?base=&quote=` | Latest ECB reference rate, cached 24 h, bundled fallback |
| `GET` | `/api/reference/holidays?country=&year=` | Public holidays, cached 24 h |
| `GET` | `/api/reference/providers?airport=` | Provider registry and deep-link templates |
| `GET` | `/api/reference/climate?place=` | Monthly temperature and rain normals with a comfort score |
| `GET` | `/api/airport-access?airport=` | City-centre transfers (time, cost), a live OSRM driving leg and minimum connection times |
| `GET` | `/api/door-to-door?from=&to=&date=` | Party fare plus the cheapest transfer at each end, as a door-to-door total |
| `GET` | `/api/data-card?dataset=` | The source, licence and method behind every dataset |

### Search body

```json
{
  "origin": "BER",
  "destination": "LIS",
  "departDate": "2027-03-01",
  "returnDate": null,
  "adults": 1,
  "children": 0,
  "checkedBags": 1,
  "carryOnBags": 1,
  "currency": "EUR",
  "cabin": "economy",
  "maxStops": null,
  "sort": "best",
  "source": "auto"
}
```

`source` is `auto` (live when `SERPAPI_KEY` is set, fixtures otherwise), `fixtures` or `serpapi`.

Every itinerary carries `totalPrice` for the party **including the requested checked bags**, a `priceBreakdown` that shows the fare and the bag component separately, the included bags, the fare rules, the redirect URL and `verifiedAt` (null for fixtures — a fixture is never a verification).

## Provenance

Every data response carries a `provenance` block:

```json
{
  "dataCardId": "fares-serpapi",
  "source": "serpapi",
  "observedAt": "2026-10-22T21:40:00Z",
  "freshness": "live",
  "ageSeconds": 0,
  "expiresAt": "2026-10-22T21:50:00Z",
  "degraded": false,
  "sampleSize": 12,
  "notes": []
}
```

`freshness` is one of `bundled` (reference data), `fixture`, `live`, `cached` or `stale`. A response without provenance is a bug: `GET /api/data-card` lists the id, source, licence and method for each dataset.

## Caching, quota and failure behaviour

| Data | TTL | On source failure |
|---|---|---|
| Search | 10 minutes | Serve the last cached answer with `freshness: "stale"` and `degraded: true` |
| FX | 24 hours | Serve the last good rate, then the bundled snapshot |
| Holidays | 24 hours | Serve the last good list, then an empty degraded list |
| Reference | none | Embedded in the binary; no network |

The live source has a daily quota (`SERPAPI_DAILY_LIMIT`, default 100) tracked in SQLite per source and day. An exhausted quota returns `429 quota_exhausted` with the reset date — or a stale cached answer if one exists. Fixtures are unmetered. A call is counted when it is attempted, including one the upstream rejects, so the cap errs toward protecting spend.

A live search also records the daily minimum per route, travel date and physical service in `fare_history`, which is the table Step 4 turns into percentiles.

## Reference data

`data/airports.json` is generated, not hand-edited:

```bash
cd app
node scripts/build-airports.mjs
```

It joins [OurAirports](https://ourairports.com/data/) (public domain: airports, coordinates, scheduled service) with IANA timezones from [mwgg/Airports](https://github.com/mwgg/Airports) (MIT), keeping large and medium airports with scheduled service. Airports with no known timezone are omitted rather than guessed — 67 today. The generated file is committed so the binary builds and starts offline.

## History collection

`faredesk-collector` is the Step 4 collector. Once a day it runs one paid search per watched route, picks one travel date per route (rotating through that route's window), and keeps the cheapest offer priced for the party and bags in `data/watchlist.yaml`.

```bash
cd app
cp ../data/watchlist.example.yaml ../data/watchlist.yaml   # then edit
cp .env.example .env                                       # add SERPAPI_KEY
cargo run -p faredesk-scheduler --bin faredesk-collector -- \
  --watchlist ../data/watchlist.yaml --dry-run
```

Drop `--dry-run` for a real pass. `--limit N` caps the number of source searches.

- One row per route, travel date and observation day: the daily minimum and the offer that produced it.
- Re-running the same day skips routes already observed, so cron retries are safe.
- The collector refuses to start without `SERPAPI_KEY`; fixture prices are never written into history.
- It respects `SERPAPI_DAILY_LIMIT`, stops when the quota is spent (exit code 2), and logs a JSON report.

In Docker, copy the example to `data/watchlist.yaml` and run the collection profile:

```bash
docker compose --profile collect run --rm collector
```

Daily cron, 06:07:

```cron
7 6 * * * cd /srv/faredesk/app && /usr/local/bin/faredesk-collector --watchlist /srv/faredesk/data/watchlist.yaml >> /var/log/faredesk-collector.log 2>&1
```

`GET /api/route-intel` reads that history and runs the buy/wait engine: below p10 books now; p10–p50 watches; above the median with more than 45 days to go waits; inside 14 days never promises a late drop; under 14 observations says `insufficient`. It returns the expected 7- and 21-day ranges from observed moves, the downside chance, the lead-time curve, a backtest at 60/30/14 days out when enough completed history exists, plain-language reasons and one line on what would change the call. The track record stays null until the engine has made calls it can score. Every response carries the `fare-history` data card, sample size and observation window.

## Explore, seasonality and fare weather

- **`GET /api/seasonality`** returns the median, p10 and p90 per travel month from the collected daily minima. A month with fewer than five observations is omitted rather than presented as a seasonal fact.
- **`GET /api/fare-weather`** returns the coefficient of variation, the last-seven-observation range, the share of day-to-day moves that fell by 5% or more, and a classification: `stable`, `erratic`, `rising` (up 2% inside 45 days) or `insufficient` under five observations.
- **`GET /api/explore`** prices each destination from the home airport on the 15th of the month, one search per destination, one adult and carry-on only. Rows are sorted by total and capped at twelve; `coverage` shows priced vs attempted. The feed is cached for six hours. A price percentile appears when that route has five or more observations. Destinations come from the demo route graph; a schedule feed is required for wider coverage.
- **Climate overlay is not built yet.** It needs Open-Meteo normals and a comfort score; Explore currently answers “cheap”, not “cheap and dry”.

## Climate normals

`GET /api/reference/climate?place=BER` fetches the last complete calendar year of daily temperatures and precipitation from Open-Meteo, aggregates monthly mean high, mean low and rain days, and scores comfort 0–100 from a mild-temperature target and a rain-day penalty. Normals are cached for 30 days and labelled historical; they are not a forecast. A failure returns an empty month list with a degraded reason rather than an error page. Open-Meteo is CC BY 4.0.

## Airport access and connection times

`GET /api/airport-access?airport=BER` returns the airport's city-centre transfers (mode, typical minutes, one-way cost in local currency), a live driving leg from OSRM (minutes, km), a live public-transit leg from Transitous (minutes, transfers) and minimum connection times for domestic and international connections. The OSRM leg is cached for 30 days, the transit leg for 24 hours at a representative 09:00 weekday departure. `GROUND_ROUTING=off` and `TRANSIT_ROUTING=off` disable the live legs.

The table in [`data/access.json`](data/access.json) covers thirteen launch airports and is **curated**, not live routing: times are typical, costs are one adult one way, and MCTs are guidance for a single ticket. Every response says so and carries the `airport-access` data card. Airports without an entry return an empty list with a note, not a 404.

`GET /api/door-to-door?from=BER&to=LIS&date=2027-03-01` returns the cheapest fare for one adult plus the cheapest curated transfer at each end, converted to the fare currency: `fare`, `transferCost`, `total`, `flightMinutes`, `transferMinutes`, `totalMinutes`, and each leg. It is the honest “what the trip actually costs” number the route page needs.

Search results also carry `connectionWarnings`: any connection whose layover is below the airport's minimum connection time is called out by name and minutes.

Still missing from this step: true 30/60/90-minute reach isochrones (we have one representative route, not an isochrone) and US BTS reliability data with a reliability-adjusted price.

## Point of sale

`POST /api/price-check` screens a shortlisted itinerary in up to five markets: departure country, destination country, US, GB and IE, deduplicated. Each market is one source search, so the endpoint is metered.

```json
{
  "origin": "BER",
  "destination": "LIS",
  "departDate": "2027-03-01",
  "itineraryId": "TAPAirPortugal-TP1041-2027-03-01",
  "adults": 1,
  "checkedBags": 1,
  "homeCurrency": "EUR",
  "fxFeePercent": 3,
  "profile": {
    "residency": "DE",
    "nationality": "DE",
    "cardCountries": ["DE"],
    "paymentMethods": ["card"]
  }
}
```

- **Net total:** fare and bags converted at the card rate, plus the card FX fee when the quote is in another currency. Same-currency rows carry no fee.
- **Eligible:** the provider accepts a payment method the traveller has and the market is one of their card countries.
- **Listed:** the price is visible but a local card or billing address may be required. **Ineligible:** the provider's payment methods do not match the profile.
- **Recommendation:** a foreign storefront only when it is eligible, the net gap is above €20 or 5%, and the same gap was already present on an earlier day in the last seven. Otherwise the home storefront is recommended, with the reason.
- **Persistence:** every quote is written to `pos_quotes`, so the seven-day check has data to read.
- **Quota:** one paid search per market. If the daily quota runs out partway through, the response contains the markets screened so far and a degraded reason.
- **Fixtures:** the fixture source is market-agnostic, so every storefront shows the same total and the endpoint says the home storefront is best. That demonstrates the contract, not a market gap.

Provider links currently point at the source's redirect surface (Google Flights), not the airline's local storefront. Re-pricing at the airline's own market is the remaining piece.

## Errors

Every failure is a JSON envelope, never an HTML error page or an empty body:

```json
{ "error": { "code": "quota_exhausted", "message": "…", "details": { "resetsOn": "2026-10-23" } } }
```

| Status | Codes |
|---|---|
| 400 | `missing_query`, `missing_origin`, `unknown_source`, `source_not_configured`, `source_rejected`, `invalid_party`, `invalid_bags`, `invalid_stops`, `invalid_currency`, `invalid_dates`, `date_in_past`, `date_too_far`, `same_place`, `return_trip_not_supported`, `unknown_dataset` |
| 404 | `unknown_place`, `not_found` |
| 429 | `rate_limited` (per-client window, `Retry-After`), `quota_exhausted` (`details.resetsOn`) |
| 502 | `source_unavailable`, `source_bad_response` |
| 500 | `internal` (a panic is caught and returned as this) |

A degraded answer is still a 200: the response says `freshness: "stale"` or `degraded: true` with a reason, because the client can use it.

## Deploy

The service is one binary with the reference dataset compiled in and a SQLite ledger on disk.

```bash
# from the repository root
docker compose up --build          # API on http://127.0.0.1:8080
```

Or without Docker:

```bash
cd app
cargo build --release -p faredesk-api
PORT=8080 DATABASE_PATH=/data/faredesk.sqlite ./target/release/faredesk-api
```

- Mount `/data` (or set `DATABASE_PATH`) so the quota ledger and `fare_history` survive restarts.
- Set `CORS_ORIGINS` to the deployed web origin; the defaults only cover local development.
- The container runs as a non-root user and needs outbound HTTPS for SerpAPI, Frankfurter and Nager.Date.
- Rate limiting is on by default at 120 requests per client per minute (`RATE_LIMIT_PER_MINUTE`, `0` disables). Clients are keyed by `X-Forwarded-For`, then `X-Real-IP`, then the socket address; `/healthz` is exempt.
- There is no authentication in this release. Put the service behind a proxy or a platform gateway before exposing it publicly.

CI runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, the full test suite, the web lint and build, and a Docker image build on every push (`.github/workflows/ci.yml`).

## Tests

```bash
cd app
cargo test          # 124 tests
cargo clippy --all-targets
```

The integration tests in `crates/api/tests/api.rs` run the real router against an in-memory store: places, a BER–LIS fixture search with bag pricing, cache hits, a dead source served from cache, a dead source without cache, quota exhaustion, validation errors and the data cards.

## Known limits in this step

- **Live search is one direction.** A SerpAPI round trip returns only outbound segment detail while pricing the pair, so the API rejects `returnDate` on the live source instead of mislabelling it. Fixtures accept a return date and price one direction, with a note.
- **Bag fees can be estimates.** When the source says a bag costs extra without naming the amount, the curated table in `core::pricing` is used and `bagFeeEstimated` is true.
- **Children are priced at the adult fare.**
- **The web app prices from this service but still holds its own demo history.** The buy/wait panel is fixture intelligence until Step 5.
- **History needs a key and time.** The collector writes only from a live source; a route needs 14 daily observations before advice appears.
- **No point-of-sale screening, no alerts, no auth or rate limiting.** Those are Steps 6, 8 and a deployment concern.
