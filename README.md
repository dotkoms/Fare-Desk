# Fleet

A field manual for paying less for planes, trains and coaches — the data sources that actually exist, which ones need an API and which need scraping, when to buy, what is live right now, and a multi-agent setup that watches it all.

**Research snapshot:** 22 October 2026. Access terms, prices and deals change constantly; every source and caveat is documented rather than assumed.

---

## Start here

| If you want to… | Read |
|---|---|
| Get the six rules that explain most savings | [docs/00-short-version.md](docs/00-short-version.md) |
| Know when to buy, by mode and region | [docs/05-when-to-buy.md](docs/05-when-to-buy.md) |
| Know which apps have APIs and which need scraping | [docs/03-public-apis-vs-scraping.md](docs/03-public-apis-vs-scraping.md) |
| See deals that are live now | [docs/06-live-deals.md](docs/06-live-deals.md) |
| Run the full tactic list | [docs/04-playbook.md](docs/04-playbook.md) |
| Understand why fares behave this way | [docs/01-how-fares-are-priced.md](docs/01-how-fares-are-priced.md) |
| Build the agent fleet | [docs/08-agent-fleet.md](docs/08-agent-fleet.md) and [agents/](agents/) |
| Avoid wasting time on myths | [docs/07-myths.md](docs/07-myths.md) |
| Build the interactive 3D map app | [PLAN.md](PLAN.md) |

If you only do three things: search the whole week and every nearby airport, run a split-ticket check before buying rail, and price the trip door to door including bags and the hotel night you are replacing.

---

## The repository

```
fleet/
├─ README.md
├─ PLAN.md                         Interactive 3D flight/train finder plan
├─ docs/
│  ├─ 00-short-version.md          Six rules and the highest-ROI moves
│  ├─ 01-how-fares-are-priced.md   Fare buckets, unbundling, quota pricing, yield
│  ├─ 02-data-sources.md           Every practical source, by mode
│  ├─ 03-public-apis-vs-scraping.md API/scrape matrix and legal posture
│  ├─ 04-playbook.md               Every tactic, with saving and catch
│  ├─ 05-when-to-buy.md            Historical booking windows and seasonality
│  ├─ 06-live-deals.md             Dated snapshot of live deals
│  ├─ 07-myths.md                  What does not work
│  ├─ 08-agent-fleet.md            Architecture, schemas, quotas, guardrails
│  ├─ 09-build-order.md            Build it in phases
│  ├─ 10-points-and-protections.md Points math, EU261, refunds, OTA risk
│  └─ 11-sources.md                Evidence and uncertainty
└─ agents/
   ├─ README.md                    How to install the fleet
   ├─ desk-ops.md
   ├─ air-scout.md
   ├─ rail-scout.md
   ├─ coach-scout.md
   ├─ arbitrage-analyst.md
   ├─ fare-verifier.md
   ├─ watchkeeper.md
   └─ points-strategist.md
```

---

## API or scrape?

| Mode | Public self-serve price API? | Practical answer |
|---|---|---|
| **Air** | No from Google Flights, Kayak, Skyscanner, Momondo or Hopper. Yes from paid wrappers (SerpAPI, SearchApi) and agent MCPs (Skiplagged, Kiwi) | Scrape or use a wrapper for discovery; SerpAPI/SearchApi for the quote; Duffel or the airline for booking |
| **Rail** | Schedules yes (DB Timetables, National Rail Darwin, GTFS). Fares mostly no | Operator site or a retailer for prices; `db-vendo-client` unofficially for DB |
| **Coach** | FlixBus gates prices; BlaBlaCar Bus has a free key; Omio is affiliate/B2B | Omio or BlaBlaCar deeplinks, then verify at the operator |
| **Awards** | Seats.aero has a paid API; point.me, AwardFares, ExpertFlyer do not | Seats.aero for discovery; confirm in the loyalty program |
| **Deal feeds** | Mostly no; Reddit and Telegram have APIs | Telegram/X push for speed; verify on the operator |

The full per-app matrix, including partner-only and closed services, is in [docs/03-public-apis-vs-scraping.md](docs/03-public-apis-vs-scraping.md).

---

## The stack in one paragraph

Scan wide with free or cheap scrapers. Verify the shortlist with a stable paid source. Store every price in a ledger so “cheap” means below the route's own median. For air, Google Flights data via SerpAPI or SearchApi is the verification backbone, with fast-flights and Travelpayouts for discovery and baselines. For rail, there is no global fare API: use DB Timetables, National Rail Darwin, GTFS and the operators for schedules, and a retailer or the operator for prices. For coaches, FlixBus publishes schedules as GTFS but gates prices; BlaBlaCar Bus has a nominally open API; Omio is the practical multimodal retailer. Keep the language model out of the price path.

---

## Ground rules

- **No booking automation.** The fleet finds and captures; a human transacts.
- **Total cost or nothing.** Bags, seats, booking fees, terminal transfers and the value of time belong on every line.
- **Link to the source.** Fetching public data is one risk; republishing airline fare content is the risk that cost Skiplagged $9.4M.
- **Fares are not guaranteed until ticketed.** Mistake fares can be cancelled; “live” award results can be stale.
- **Not legal advice.** Contracts of carriage and local rules still govern your ticket.

## Contributing

This is a personal research repo. If you change an access claim, note the date and the source. The data landscape moves faster than the prose.
