# Rust data layer and Obscura

**Decision:** build the data layer in Rust, and use [Obscura](https://github.com/h4ckf0r0day/obscura) as the headless browser for the sources that need one. The web frontend stays TypeScript.

This keeps the whole backend — API clients, GTFS ingest, scraping, normalization, ranking, caching and quota — in one Rust workspace, with no Node or Python sidecar for rendering.

---

## 1. What is an API and what needs scraping

Counted across the feeds this product would actually use:

| Access type | Share of sources | Examples |
|---|---:|---|
| **Self-serve API** | ~40–45% | SerpAPI, SearchApi, Travelpayouts, Duffel, DB Timetables, National Rail Darwin, Realtime Trains, Navitia, Transitous, GTFS, BlaBlaCar Bus, Seats.aero, Skiplagged MCP, Kiwi MCP, Reddit |
| **Scrape or unofficial endpoint** | ~25% | Azair, Ryanair, Wizz schedules, Trenitalia, Renfe, 12306, Megabus, National Express, consumer deal sites, `db-vendo-client` |
| **Partner/contract only** | ~25% | Trainline, Rail Europe, FlixBus prices, Busbud, Distribusion, Wanderu, Amtrak, IRCTC, JR East, Skyscanner, Sabre, Travelport, Southwest, easyJet |
| **Closed** | ~5–10% | Amadeus Self-Service (dead July 2026), Kiwi Tequila (invite-only), Google QPX |

The shares are by source count, not data volume. For the MVP — places, air prices, rail schedules, route previews — the clean API tier covers most of what the screen needs.

**The important nuance:** the best cash deals often sit on the scrape/unofficial side, because APIs miss LCC and OTA content. That is why the architecture separates **discovery** (may scrape) from **verification** (must use a licensed or operator source).

---

## 2. Feed-by-feed

### Air

| Feed | Access | Rust client | Browser? |
|---|---|---|---|
| SerpAPI (Google Flights) | Self-serve REST | `reqwest` + `serde` | No |
| SearchApi (Google Flights) | Self-serve REST | `reqwest` + `serde` | No |
| Travelpayouts Data API | Free affiliate REST | `reqwest` | No |
| Duffel | Self-serve REST | `reqwest` | No |
| Skiplagged MCP | Official MCP | `rmcp` | No |
| Kiwi MCP | Official MCP | `rmcp` | No |
| FlightAware / OpenSky | REST, OAuth2 | `reqwest` | No |
| fast-flights | Python scraper | Port or replace with SerpAPI | No |
| Azair | HTML scrape, expired TLS | `reqwest` + `scraper`, isolated TLS | No |
| Ryanair | Unofficial JSON | `reqwest` | No |
| Wizz schedules | Open JSON | `reqwest` | No |
| Wizz search | Kasada challenge | Not viable | Obscura likely blocked |
| Southwest / easyJet | Prohibited | Do not build | — |
| Amadeus / Tequila | Closed | Do not build | — |

### Rail

| Feed | Access | Rust client | Browser? |
|---|---|---|---|
| DB Timetables | Free official XML | `reqwest` + `quick-xml` | No |
| National Rail Darwin / LDBWS | Free official SOAP/JSON | `reqwest` + `quick-xml` | No |
| Realtime Trains | Paid self-serve REST | `reqwest` | No |
| Navitia (SNCF) | Free-tier REST | `reqwest` | No |
| Transitous | Free MOTIS REST | `reqwest` | No |
| GTFS / GTFS-RT | Files, CSV, protobuf | `gtfs-structures`, `gtfs-rt`, `prost` | No |
| db-vendo-client | Unofficial DB mobile JSON | Reimplement endpoints | No |
| Trenitalia | Unofficial JSON | `reqwest` | No |
| Renfe | JS-heavy site | `reqwest` first; Obscura if needed | Sometimes |
| 12306 | Active anti-bot | Not viable | Obscura unlikely |
| Ekispert / Jorudan / Navitime | REST/XML | `reqwest` | No |
| Trainline / Rail Europe | Partner contract | `reqwest` | No |
| Omio B2B | Affiliate/B2B REST | `reqwest` | No |
| Amtrak / IRCTC / JR East | Partner only | `reqwest` if contracted | No |

### Coach, awards and deals

| Feed | Access | Rust client | Browser? |
|---|---|---|---|
| FlixBus GTFS | Open files | `csv` + `zip` | No |
| FlixBus prices | Partner only | `reqwest` if contracted | No |
| BlaBlaCar Bus | Free key REST (stale docs) | `reqwest` | No |
| Omio | Affiliate/B2B | `reqwest` | No |
| Wanderu | Affiliate links, no API | `reqwest` + `scraper` | Sometimes |
| Megabus / National Express | HTML scrape | `reqwest` + `scraper` | Sometimes |
| Busbud / Distribusion | Partner only | `reqwest` if contracted | No |
| Mobility Database | REST | `reqwest` | No |
| Seats.aero | Paid API + MCP | `reqwest` / `rmcp` | No |
| point.me / AwardFares / ExpertFlyer | No public API | Do not scrape | — |
| Reddit | Public API | `reqwest` | No |
| Telegram | Bot API / MTProto | `teloxide` / `grammers` | No |
| X | Paid API | `reqwest` | No |
| RSS | Feeds | `feed-rs` | No |
| Secret Flying / Fly4Free / The Flight Deal | HTML/RSS | `reqwest` + `scraper` | Sometimes |
| Going / Jack's Flight Club | Login-gated email/site | Email or user session; avoid scraping | Obscura possible, ToS risk |

---

## 3. How much is Rust

**Essentially all of the data layer, with three exceptions.**

| Layer | Language | Share |
|---|---|---|
| API adapters (REST, XML, SOAP, GraphQL, MCP) | Rust | 100% |
| GTFS and GTFS-RT ingest | Rust | 100% |
| Static HTML scrapers | Rust (`scraper`) | ~70% of scrape targets |
| JS-heavy or login-gated scrapers | Rust + Obscura | ~30% of scrape targets |
| Browser pool, scheduling, retries, quotas | Rust | 100% |
| Normalization, fingerprinting, ranking, caching | Rust | 100% |
| Search API | Rust (Axum) | 100% |
| Web frontend | TypeScript / React | 0% |

**Practical estimate:** 85–90% of the source count runs natively in Rust. The rest runs through Obscura, which is also Rust. The only genuinely non-Rust piece is the React globe.

**What no language fixes:** active bot protection (Cloudflare interactive challenges, Akamai, Datadome, CAPTCHAs), closed partner APIs, and prohibited scraping. Obscura's own docs say stealth handles basic TLS/UA checks, not active challenges.

---

## 4. Workspace

```
app/
├─ Cargo.toml                 # workspace
├─ crates/
│  ├─ core/                   # types, geo, fingerprint, rank, total cost
│  ├─ sources/                # one adapter per feed
│  │  ├─ serpapi.rs
│  │  ├─ travelpayouts.rs
│  │  ├─ db_timetables.rs
│  │  ├─ darwin.rs
│  │  ├─ transitous.rs
│  │  ├─ gtfs.rs
│  │  ├─ blablacar.rs
│  │  └─ deal_feeds.rs
│  ├─ browser/                # Obscura pool + page helpers
│  ├─ store/                  # SQLite/Postgres, ledger, quota
│  ├─ api/                    # Axum routes
│  └─ scheduler/              # watchlist, refreshes, alerts
├─ sidecar/                   # optional: Playwright only if Obscura fails
└─ web/                       # React + react-globe.gl
```

### Core traits

```rust
#[async_trait]
pub trait FareSource: Send + Sync {
    fn id(&self) -> &'static str;
    fn access(&self) -> Access;          // Api | File | Html | Browser | Partner
    async fn search(&self, q: &Query) -> Result<Vec<Offer>, SourceError>;
    async fn verify(&self, offer: &Offer) -> Result<VerifiedOffer, SourceError>;
}

pub trait RouteGeometry {
    fn geometry(&self, from: &Place, to: &Place) -> Geometry;
}
```

Every adapter returns the same `Offer`. The globe never knows whether a segment came from SerpAPI, GTFS or Obscura.

### Crates

`tokio`, `axum`, `reqwest` (rustls), `serde`, `serde_json`, `quick-xml`, `csv`, `zip`, `gtfs-structures`, `gtfs-rt`, `prost`, `sqlx`, `moka`, `governor`, `tower-http`, `tracing`, `rmcp`, `scraper`, `teloxide`, `grammers`, `feed-rs`, `geo`, `geodesy`, `obscura`.

---

## 5. Obscura

### What it is

A headless browser engine written in Rust, for scraping and agent automation. It runs JavaScript through V8, speaks the Chrome DevTools Protocol, and works as a drop-in for headless Chrome with Puppeteer and Playwright. Apache-2.0, created April 2026, actively maintained as of October 2026.

Vendor-reported advantages: ~30 MB memory versus 200+ MB for headless Chrome, ~70 MiB binary, fast startup. Treat those as marketing until measured on our workload.

### Three ways to use it

| Interface | Use |
|---|---|
| **Rust crate** (`obscura`, git dependency) | Embed the browser in a worker; `Browser` / `Page` / `Element`, no CDP round-trips. Best for production scrapers. |
| **CDP server** (`obscura serve --port 9222`) | Puppeteer/Playwright compatibility, debugging, parallel workers. |
| **MCP server** (`obscura mcp`) | Let Claude agents drive a live browser during development or research. Not the production data path. |

The crate API is the right default because the rest of the data layer is already Rust. It exposes:

- `Browser::builder().stealth(true).proxy(url).storage_dir(dir).build()`
- `page.goto(url)`, `content()`, `url()`, `evaluate(js)`
- `query_selector`, `wait_for_selector`, `Element::text/attribute/click`
- `settle(ms)` to let in-page `fetch` and timers resolve
- `on_request` / `on_response` passive callbacks
- `enable_interception()` to block, mock or rewrite requests
- `add_preload_script(js)` before page scripts run
- `CookieStore` with save/load

**The feature that matters most for us is `on_response`.** For a JS-heavy airline or OTA, do not parse the rendered DOM. Capture the JSON the page fetches, then parse that with `serde`. DOM scraping breaks on every redesign; JSON endpoints change less often and carry cleaner data.

### Recommended pattern

```rust
use obscura::{Browser, ResourceType};
use std::sync::{Arc, Mutex};

pub struct BrowserPool {
    browser: Browser,
    permits: tokio::sync::Semaphore,
}

impl BrowserPool {
    pub fn new(concurrency: usize) -> anyhow::Result<Self> {
        Ok(Self {
            browser: Browser::builder().stealth(true).build()?,
            permits: tokio::sync::Semaphore::new(concurrency),
        })
    }

    pub async fn capture_json(
        &self,
        url: &str,
        wait_ms: u64,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let _permit = self.permits.acquire().await?;
        let mut page = self.browser.new_page().await?;
        let captured = Arc::new(Mutex::new(Vec::new()));

        {
            let captured = captured.clone();
            page.on_response(Arc::new(move |info, resp| {
                if info.resource_type == ResourceType::Fetch {
                    if let Ok(value) = serde_json::from_slice(&resp.body) {
                        captured.lock().unwrap().push(value);
                    }
                }
            }));
        }

        page.goto(url).await?;
        page.settle(wait_ms).await?;
        let out = captured.lock().unwrap().clone();
        Ok(out)
    }
}
```

### One browser, or a pool?

- **Development:** one `Browser`, a few pages.
- **Production:** one `Browser` per worker process, with a semaphore per domain and a global cap. Use `obscura serve --workers N` if driving through CDP; use one container per N CPUs if embedding.
- Recycle workers with `--worker-max-connections`, `--worker-max-rss` or `--worker-max-age` when using the CDP server; with the embedded crate, build a fresh `Browser` every few hundred pages.
- Pin a release tag. The documented example is `v0.1.7`; the crate is a git dependency because V8 is built from source.

### What Obscura does not solve

From its own documentation:

- Cloudflare interactive challenges.
- Datadome and Akamai bot manager active challenges.
- CAPTCHAs.
- IP-based rate limiting — use proxies.
- Passive TCP/IP stack fingerprinting — the kernel sets those fields; run in a matching container/VM or through a proxy with the right stack.

So: use Obscura for JS rendering, SPA JSON capture, cookie/session handling and basic TLS/UA stealth. Do **not** promise that it defeats an active challenge. If a source is hard-blocked, the answer is a partner API or dropping the source, not a more elaborate browser.

### Security and operations

- V8 runs untrusted page JavaScript **in process**. Run Obscura in a container or dedicated user, non-root, with a read-only root filesystem and a writable storage volume only for cookies.
- Egress allowlist: the browser should reach only the domains in the source registry. Do not give it general internet access.
- One proxy per source where needed; align timezone and geolocation with the proxy region.
- No airline or OTA credentials in the browser. Public pages only. Login-gated deal clubs: use the user's own session only with explicit consent, and expect ToS/account risk.
- Strip cookies and storage between unrelated sources.
- Bound navigation time, page count and response size. Reject non-HTML/JSON content early.
- Log every navigation and extraction with source, URL hash and duration; never log session cookies.

### Where it sits in the pipeline

```
API sources ───────────────┐
GTFS / files ──────────────┤
static HTML (reqwest) ─────┤──► normalizer ──► ranker ──► verifier ──► API
JS / SPA / session ──► Obscura ─┘
```

Obscura is a **collector**, never a verifier. A price scraped through a browser is a lead until an API or the operator's own page confirms it.

---

## 6. Source implementation notes

| Source | Approach | Notes |
|---|---|---|
| **Google Flights** | SerpAPI | Do not scrape directly in production. fast-flights is a fragile Python parser; porting it to Rust is not worth it. |
| **Kayak / Momondo / Skyscanner** | Partner API or nothing | Scraping prohibited; do not build. |
| **Azair** | `reqwest` + `scraper` | Static HTML; the expired TLS certificate must be handled in an isolated client, never globally. |
| **Ryanair** | `reqwest` | Needs `client-version` header and correlation cookie; unofficial. |
| **Wizz** | Schedules only | Search is behind Kasada; Obscura is likely blocked. |
| **DB** | Timetables XML + `db-vendo-client` endpoints | Schedules official; fares unofficial and rate-limited. |
| **National Rail** | Darwin SOAP/JSON | SOAP envelope via `quick-xml`; cache station messages. |
| **Trenitalia** | Reverse-engineered JSON | `reqwest`; brittle; personal use only. |
| **Renfe** | `reqwest`, Obscura fallback | JS-heavy; start with the mobile JSON if discoverable. |
| **12306** | Do not build | Active anti-bot and no authorized third-party access. |
| **FlixBus** | GTFS for schedules; Omio for prices | Do not scrape the consumer site for prices. |
| **Megabus / National Express** | `reqwest` + `scraper` | Mostly static; check ToS and rate-limit hard. |
| **Deal sites** | RSS/HTML; IMAP for newsletters | Going/JFC are login-gated; prefer email over scraping. |
| **Award tools** | Seats.aero API/MCP | point.me/AwardFares/ExpertFlyer have no public API; do not scrape. |

---

## 7. Effort

| Work | Estimate |
|---|---:|
| REST/JSON API adapter (with tests) | 0.5–1 day |
| XML/SOAP adapter | 1–2 days |
| GTFS + GTFS-RT ingest | 2–4 days |
| Static HTML scraper | 1–3 days, plus maintenance |
| Obscura adapter + JSON capture | 2–4 days, plus maintenance |
| Browser pool, rate limits, retries, circuit breaker | 3–5 days |
| Normalizer + fingerprint + ranker | 4–6 days |
| Axum API + cache + quota | 4–6 days |

**Build order:** API sources first, GTFS second, static scrapers third, Obscura last. Obscura is the highest-maintenance component; add it only for sources that genuinely need JavaScript.

---

## 8. Decisions recorded

1. **Rust for the data layer.** One workspace, one type system, no runtime bridge.
2. **Obscura for browser-bound sources.** Apache-2.0, Rust-native, embeddable, CDP-compatible. Pin a tag and isolate it.
3. **TypeScript for the web app.** `react-globe.gl` and the panel are not worth porting.
4. **SerpAPI for air verification.** Not a Google scrape.
5. **No browser for sources that expose JSON**, even if the site is JS-heavy. Capture the network response instead of parsing the DOM.
6. **Obscura is a collector, not a verifier.** Every price is re-priced at the operator or a licensed API before display.

---

## 9. Sources

- [Obscura repository](https://github.com/h4ckf0r0day/obscura)
- [Use as a Rust library](https://github.com/h4ckf0r0day/obscura/blob/main/docs/Use-as-a-Rust-library.md)
- [Run in production at scale](https://github.com/h4ckf0r0day/obscura/blob/main/docs/Run-in-production-at-scale.md)
- [Configure stealth and proxies](https://github.com/h4ckf0r0day/obscura/blob/main/docs/Configure-stealth-and-proxies.md)
- [Use the MCP server](https://github.com/h4ckf0r0day/obscura/blob/main/docs/Use-the-MCP-server.md)
- [Obscura documentation](https://docs.obscura.sh)

Access status for each feed: [03-public-apis-vs-scraping.md](03-public-apis-vs-scraping.md). Data-source detail: [02-data-sources.md](02-data-sources.md).
