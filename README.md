# Fare Desk

**Repository:** `dotkoms/fleet` — the app is Fare Desk; the repo keeps its original name.

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
| Build the flights-only 3D finder and buy/wait engine | [PLAN.md](PLAN.md) |
| See the unique data layers and open-source playbook | [PLAN.md §10](PLAN.md#10-unique-data-and-visual-layers) and [§18](PLAN.md#18-standout-open-source-playbook) |
| Solve cold start and Explore coverage | [docs/14-cold-start-and-coverage.md](docs/14-cold-start-and-coverage.md) |
| Deploy the web on Vercel | [docs/15-hosting-on-vercel.md](docs/15-hosting-on-vercel.md) |
| Design the interface | [UI/UX](docs/16-ui-ux.md) · [design system](docs/17-design-system.md) · [colour](docs/18-color.md) |
| Add gamification without dark patterns | [docs/19-gamification.md](docs/19-gamification.md) |
| Understand users and competitors | [docs/20-users-and-competitors.md](docs/20-users-and-competitors.md) |
| Build the globe with react-globe.gl | [docs/21-globe-implementation.md](docs/21-globe-implementation.md) |
| Plan the build step by step | [docs/22-build-planner.md](docs/22-build-planner.md) |
| Decide what is an API and what needs a browser | [docs/12-rust-data-layer.md](docs/12-rust-data-layer.md) |
| Check whether another storefront is cheaper | [docs/13-point-of-sale.md](docs/13-point-of-sale.md) |

If you only do three things: search the whole week and every nearby airport, run a split-ticket check before buying rail, and price the trip door to door including bags and the hotel night you are replacing.

---

## The repository

```
fleet/
├─ README.md
├─ PLAN.md                         Flights-only 3D finder + buy/wait plan
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
│  ├─ 11-sources.md                Evidence and uncertainty
│  ├─ 12-rust-data-layer.md        Rust feeds, API vs scrape, Obscura
│  ├─ 13-point-of-sale.md          Storefront, currency and payment arbitrage
│  ├─ 14-cold-start-and-coverage.md History seeding and current route coverage
│  ├─ 15-hosting-on-vercel.md      Vercel split, data plane and costs
│  ├─ 16-ui-ux.md                  Screens, flows, copy, accessibility
│  ├─ 17-design-system.md          Type, layout, components, motion
│  ├─ 18-color.md                  Palette, semantics, contrast
│  ├─ 19-gamification.md           Ethical mechanics and metrics
│  ├─ 20-users-and-competitors.md  Beachhead, personas, competitive gap
│  ├─ 21-globe-implementation.md   react-globe.gl props, data and budgets
│  └─ 22-build-planner.md          Effort, gates, docs and agents per step
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

- **Redirect, don't sell.** Fare Desk links to the airline, OTA or operator; it does not process payment, issue tickets, or handle changes and refunds.
- **No booking automation.** The agents find and capture; a human transacts on the provider's site.
- **Total cost or nothing.** Bags, seats, booking fees, terminal transfers and the value of time belong on every line.
- **Link to the source.** Fetching public data is one risk; republishing airline fare content is the risk that cost Skiplagged $9.4M.
- **Fares are not guaranteed until ticketed.** Mistake fares can be cancelled; “live” award results can be stale.
- **Not legal advice.** Contracts of carriage and local rules still govern your ticket.

## Licence

- **Code:** GPL-3.0. See [LICENSE](LICENSE).
- **Contributions:** CLA. See [CLA.md](CLA.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
- **Name and logo:** [TRADEMARK.md](TRADEMARK.md).
- **Fare data, fee database and hosted service:** proprietary. See [DATA-LICENSE.md](DATA-LICENSE.md).
- **Documentation:** CC BY 4.0 unless a file says otherwise.
- **Security:** [SECURITY.md](SECURITY.md).

GPL-3.0 requires source when the software is distributed. It does **not** require source for a hosted service. If that network protection becomes the priority, AGPL-3.0 is the alternative licence.

## Contributing

Fare Desk is an open-source project. Read [CONTRIBUTING.md](CONTRIBUTING.md) and sign the [CLA](CLA.md) before your first pull request. If you change an access claim, note the date and the source. The data landscape moves faster than the prose.
