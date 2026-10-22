# Fare Desk data plane

Rust workspace for places, fares and reference data. The React app in [`web/`](web) is the interface; this service is the thing it asks.

```
app/
├─ Cargo.toml            workspace
├─ crates/
│  ├─ core/              contract types, pricing, fingerprint, ranking
│  ├─ store/             embedded reference data, TTL cache, SQLite ledger
│  ├─ sources/           fixtures, SerpAPI, FX and holiday clients
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

The live source has a daily quota (`SERPAPI_DAILY_LIMIT`, default 100) tracked in SQLite per source and day. An exhausted quota returns `429 quota_exhausted` with the reset date — or a stale cached answer if one exists. Fixtures are unmetered.

A live search also records the daily minimum per route, travel date and physical service in `fare_history`, which is the table Step 4 turns into percentiles.

## Reference data

`data/airports.json` is generated, not hand-edited:

```bash
cd app
node scripts/build-airports.mjs
```

It joins [OurAirports](https://ourairports.com/data/) (public domain: airports, coordinates, scheduled service) with IANA timezones from [mwgg/Airports](https://github.com/mwgg/Airports) (MIT), keeping large and medium airports with scheduled service. Airports with no known timezone are omitted rather than guessed — 67 today. The generated file is committed so the binary builds and starts offline.

## Tests

```bash
cd app
cargo test          # 62 tests
cargo clippy --all-targets
```

The integration tests in `crates/api/tests/api.rs` run the real router against an in-memory store: places, a BER–LIS fixture search with bag pricing, cache hits, a dead source served from cache, a dead source without cache, quota exhaustion, validation errors and the data cards.

## Known limits in this step

- **Live search is one direction.** A SerpAPI round trip returns only outbound segment detail while pricing the pair, so the API rejects `returnDate` on the live source instead of mislabelling it. Fixtures accept a return date and price one direction, with a note.
- **Bag fees can be estimates.** When the source says a bag costs extra without naming the amount, the curated table in `core::pricing` is used and `bagFeeEstimated` is true.
- **Children are priced at the adult fare.**
- **No scheduler yet.** History is written by live searches; Step 4 adds the daily collector.
- **No point-of-sale screening, no alerts, no auth.** Those are Steps 6 and 8.
