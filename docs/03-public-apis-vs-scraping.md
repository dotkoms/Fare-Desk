# Public APIs vs web scraping

The short answer: **public APIs are best for verification and booking; scraping is how you discover the deals the APIs miss. The best-deal workflows need both, and the scraping half carries real terms-of-service risk.**

Most consumer travel apps do not publish a price API at all. Metasearch sites exist to sell clicks, so their business model depends on not handing out inventory. Where an API exists, it is usually partner-only, returns schedules rather than fares, or is a commercial wrapper around a scrape (SerpAPI, SearchApi, Apify).

Use this matrix as a starting point, not a contract. Access terms change; verify before building.

---

## Summary

| Access type | What it means | Examples |
|---|---|---|
| **Public, self-serve** | Sign up, get a key, use it today | SerpAPI, SearchApi, Travelpayouts, Duffel, DB Timetables, National Rail Darwin, Realtime Trains, GTFS, Mobility Database, BlaBlaCar Bus, Skiplagged MCP, Kiwi MCP |
| **Public but limited** | Free tier or cached data only | Travelpayouts (cached, not live), Jorudan (3,000 calls/month), Navitia (5,000/day, no fares), OpenSky (credits) |
| **Partner-only** | Contract, accreditation, traffic minimum or sales cycle | Skyscanner, Sabre, Travelport, Trainline, Rail Europe, FlixBus, Busbud, Distribusion, Wanderu (carrier side), Amtrak, IRCTC, JR East |
| **Scrape-only** | No API; the consumer site is the data source | Google Flights, Kayak, Momondo, Azair, Trenitalia, 12306, Megabus, National Express, Southwest, easyJet |
| **Closed** | No new access at a price you would pay | Amadeus Self-Service, Kiwi Tequila, Google QPX, Rome2Rio (new keys) |

---

## Air

| App / service | Public API? | Partner API? | Scrape needed? | Best use |
|---|---|---|---|---|
| **Google Flights** | No | No | **Yes**, or via SerpAPI/SearchApi | Broadest fare discovery; calendar and Explore |
| **ITA Matrix** | No | No | Manual/HTML only | Fare rules and routing research |
| **Kayak** | No | Limited, by assessment | **Yes**, but prohibited by ToS | Cross-OTA comparison |
| **Momondo** | No | No | **Yes**, but prohibited | Price comparison |
| **Skyscanner** | No | Yes, commercial, >100k traffic | **Yes**, prohibited | LCC-heavy markets, "Everywhere" |
| **Hopper** | No | Yes (HTS), sales-led | No consumer route | Prediction and freeze, app-only |
| **Skiplagged** | **Yes** — official MCP, free, no key | — | No | Hidden-city and flex-date leads |
| **Kiwi.com** | **Yes** — MCP, free, no key | Tequila invite-only | No | Self-transfer search |
| **Ryanair** | No official public API | — | **Yes**, unofficial clients | European LCC fares |
| **Wizz Air** | Partner portal by approval | Yes | Search endpoint is bot-gated | Schedules only without a deal |
| **easyJet** | No | Yes, approved channel/Direct API | **Yes**, prohibited | Use Duffel or a GDS |
| **Southwest** | No | Yes, SPS | **Yes**, prohibited and enforced | Use a partner or skip |
| **American / Delta / United** | No public fare API | NDC/GDS via aggregators | Prohibited | Book direct |
| **Amadeus** | No (self-service dead) | Enterprise only | No | Not viable for an individual |
| **Duffel** | **Yes**, self-serve | — | No | Booking 300+ airlines |
| **SerpAPI** | **Yes**, commercial | — | Wraps Google Flights | Verification-grade Google data |
| **SearchApi** | **Yes**, commercial | — | Wraps Google Flights | Cheapest at volume |
| **Travelpayouts** | **Yes**, affiliate | — | No | Cached baselines |
| **FlightAware / OpenSky** | **Yes** | — | No | Positioning, status, not fares |

**Bottom line for air:** every major metasearch app needs scraping (or a paid scraper API) because none publishes a consumer fare API. The two exceptions aimed at agents are Skiplagged and Kiwi MCP. For a reliable quote, use SerpAPI/SearchApi; for booking, Duffel; for discovery, fast-flights or Azair.

---

## Rail

| App / service | Public API? | Partner API? | Scrape needed? | Best use |
|---|---|---|---|---|
| **Trainline** | No | Yes, commercial, ~12-week integration | No consumer route | Booking and SplitSave at scale |
| **Rail Europe** | No | Yes, trade account | No | Multi-provider booking |
| **Omio** | Affiliate API | Yes, B2B | No | Multimodal deeplinks |
| **DB (Deutsche Bahn)** | **Yes** — Timetables API, free | Sales APIs partner-only | `db-vendo-client` unofficial for fares | Schedules free; fares unofficially |
| **SNCF** | **Yes** — Navitia free tier | Connect partner/affiliate | No for routing | Routing and real time, not fares |
| **Trenitalia** | No | No | **Yes**, reverse-engineered | Italian fares, fragile |
| **Renfe** | No | No | **Yes** | Spanish fares, fragile |
| **National Rail (UK)** | **Yes** — Darwin/LDBWS free | — | No | UK real time |
| **Realtime Trains** | **Yes** — paid self-serve | — | No | UK running detail |
| **Amtrak** | No | Yes, partner agreement | Prohibited | US rail |
| **JR East / Japan** | No | Business Eki-net for agencies | Via Ekispert/Jorudan/Navitime | Japan |
| **IRCTC (India)** | No | B2B PSP scheme | Prohibited | India |
| **12306 (China)** | No | No authorized third party | **Yes**, anti-bot | China |
| **GTFS / Mobility Database** | **Yes**, open | — | No | Schedules only |
| **Split tools** (TrainSplit, Split My Fare, TrainPal, Railboard) | No | Affiliate in some cases | No | Manual split check |

**Bottom line for rail:** no global fare API exists. Schedules are open; prices are gated. The practical stack is DB Timetables + National Rail Darwin + GTFS for discovery, the operator or a retailer for prices, and `db-vendo-client` only as a personal, rate-limited bridge.

---

## Coach

| App / service | Public API? | Partner API? | Scrape needed? | Best use |
|---|---|---|---|---|
| **FlixBus / Greyhound** | GTFS schedules only | Yes, Entra SSO + distribution deal | **Yes** for prices | Schedules free; prices gated |
| **BlaBlaCar Bus** | **Yes**, free key (docs stale) | — | No | European coach search + deeplink |
| **Omio** | Affiliate API | Yes, B2B | No | Multimodal deeplinks |
| **Wanderu** | No | Carrier/affiliate only | No | US bus metasearch |
| **Busbud** | No | Yes, revenue share | No | Global bus/rail |
| **Distribusion** | No | Yes, contract | No | B2B inventory |
| **Megabus** | No | No | **Yes** for fares; prohibited | US/Canada promos |
| **National Express** | No | Affiliate only | **Yes**, prohibited | UK coach |
| **OurBus** | No | Affiliate only | No | US regional |
| **Rome2Rio** | Legacy keys only | — | **Yes**, prohibited | Route discovery |
| **Mobility Database** | **Yes**, open | — | No | GTFS schedules |

**Bottom line for coach:** FlixBus gates prices, so either use a partner (Omio/Busbud/Distribusion), scrape and accept the risk, or verify manually. BlaBlaCar Bus is the only major coach operator with a nominally open, free API.

---

## Award and points

| App / service | Public API? | Notes |
|---|---|---|
| **Seats.aero** | Yes, paid Pro API and MCP | Best flexible-date award discovery |
| **point.me** | No public API | Paid consumer search, transfer guidance |
| **AwardFares** | No public API | Program research, real-time at top tier |
| **Roame** | No public API | Live search, SkyView |
| **ExpertFlyer** | No public API | Award/upgrade inventory |
| **AwardWallet** | No public API | Balances and expirations |

---

## Deal feeds

| Source | Public API? | Practical access |
|---|---|---|
| **Going** | No | Email; membership |
| **Jack's Flight Club** | No | Email/app; membership |
| **Secret Flying** | No | Website, X, Telegram |
| **The Flight Deal** | No | Website, RSS, X |
| **Fly4Free** | No | Website, Telegram |
| **FlyerTalk** | No | Forum, RSS per thread |
| **Reddit** | Yes | Public API with rate limits and paid tiers |
| **X** | Paid API | Expensive; scraping restricted |
| **Telegram** | Bot API for channels you control; public web previews for reading | Push is fastest |

---

## The legal posture, briefly

- **CFAA:** after *Van Buren* (2021) and *hiQ v. LinkedIn* (9th Cir.), scraping public pages without bypassing a gate is generally not a CFAA violation.
- **Contract/ToS:** this is where scrapers lose. *hiQ* itself lost on breach of contract after creating accounts. *Southwest v. Kiwi.com* (N.D. Tex. 2021) enforced Southwest's terms and enjoined scraping and republication.
- **Anonymous public scraping:** *Meta v. Bright Data* (N.D. Cal. 2024) and *X Corp. v. Bright Data* (2024) held browsewrap terms do not bind anonymous visitors who never manifested assent. This defence disappears if you register an account or bypass a gate.
- **Copyright in booking content is the live risk.** That is what cost Skiplagged $9.4M. Fetching is one thing; republishing airline fare content is another.

**Practical rule:** logged-out, rate-limited scraping of public facts is the lowest-risk end. Registering accounts, rotating proxies to defeat blocks, or reselling data moves you toward injunction territory.

This is risk framing, not legal advice.

---

## Recommendation

| Need | Use |
|---|---|
| Wide discovery, free | fast-flights, Azair, Skiplagged MCP, Kiwi MCP |
| Baselines and price history | Travelpayouts, your own ledger |
| Verification-grade air quote | SerpAPI or SearchApi |
| Booking | Duffel, or the airline direct |
| Rail schedules | DB Timetables, National Rail Darwin, GTFS |
| Rail prices | Operator site, Trainline/Rail Europe if you have a contract, `db-vendo-client` personally |
| Coach schedules | FlixBus GTFS, Mobility Database |
| Coach prices | Omio/BlaBlaCar deeplinks, operator verification |
| Awards | Seats.aero, point.me, AwardFares |
| Deals | Telegram/X first, newsletters second, verify on the operator |

**Never** put a scraped price in a user-facing alert without re-pricing it at the source.
