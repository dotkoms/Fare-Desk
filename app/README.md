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
| `POST` | `/api/search` | Priced, deduplicated, ranked itineraries with provenance |
| `GET` | `/api/reference/fx?base=&quote=` | Latest ECB reference rate, cached 24 h, bundled fallback |
| `GET` | `/api/reference/holidays?country=&year=` | Public holidays, cached 24 h |
| `GET` | `/api/reference/providers?airport=` | Provider registry and deep-link templates |
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

`GET /api/route-intel` reads that history. It returns `recommendation: "insufficient"` under 14 observations; once the sample is large enough it returns the statistics with `recommendation: null` until the Step 5 buy/wait engine lands. It always carries the `fare-history` data card, sample size and observation window.

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
cargo test          # 80 tests
cargo clippy --all-targets
```

The integration tests in `crates/api/tests/api.rs` run the real router against an in-memory store: places, a BER–LIS fixture search with bag pricing, cache hits, a dead source served from cache, a dead source without cache, quota exhaustion, validation errors and the data cards.

## Known limits in this step

- **Live search is one direction.** A SerpAPI round trip returns only outbound segment detail while pricing the pair, so the API rejects `returnDate` on the live source instead of mislabelling it. Fixtures accept a return date and price one direction, with a note.
- **Bag fees can be estimates.** When the source says a bag costs extra without naming the amount, the curated table in `core::pricing` is used and `bagFeeEstimated` is true.
- **Children are priced at the adult fare.**
- **The web app prices from this service but still holds its own demo history.** The buy/wait panel is fixture intelligence until Step 5.
- **No scheduler yet.** History is written by live searches; Step 4 adds the daily collector.
- **No point-of-sale screening, no alerts, no auth or rate limiting.** Those are Steps 6, 8 and a deployment concern.
