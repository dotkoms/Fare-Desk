# Data sources

Grouped by what they are good for. Access status is as of **October 2026** and changes often; the caveats matter as much as the names.

Read the access column before the coverage column. The landscape has four tiers:

- **Self-serve** — you can sign up today.
- **Partner-only** — a contract and a sales cycle.
- **Scrape-only** — works, breaks, and usually breaches terms.
- **Closed** — no new access at any price you would pay.

For the per-app API/scrape matrix, see [03-public-apis-vs-scraping.md](03-public-apis-vs-scraping.md).

---

## Air

| Source | What you get | Access and cost | Use |
|---|---|---|---|
| [SerpAPI — Google Flights](https://serpapi.com/google-flights-api) | Live Google Flights fares, price history, multi-city, carbon | Self-serve. 250 free/month; $25/1k; $75/5k. A round trip costs two searches | Verification backbone |
| [SearchApi — Google Flights](https://www.searchapi.io/google-flights-api) | Same surface, including the calendar engine | Self-serve. 100 free; roughly $4/1k at entry | Cheapest at volume |
| [fast-flights](https://github.com/AWeirdDev/flights) | Scrapes Google Flights directly; filters for stops, airlines, bags | Open source, MIT, free. Breaks when Google changes its payload | Free wide net only |
| [Travelpayouts Data API](https://support.travelpayouts.com/hc/en-us/articles/203956163) | Cached cheap fares, price calendar, popular destinations | Free affiliate signup; token immediate. Data can be 2–7 days old | Baselines, not quotes |
| [Skiplagged MCP](https://skiplagged.github.io/mcp/) | Hidden-city search, flex-date calendars, "anywhere" | Official MCP, free, no key. Indicative prices | Leads only |
| [Kiwi MCP](https://mcp.kiwi.com) | One-way/round-trip search with self-transfer itineraries | Free, no key; no multi-city or checked-bag selection | Personal search |
| [Azair](https://github.com/yuriiter/azair-eu-mcp) | European LCC flexible-date and "anywhere" search | No API; HTML scrape. 10–60 seconds per query | Best free flex search, brittle |
| [@2bad/ryanair](https://github.com/2BAD/ryanair) | Ryanair availability, fares, cheapest-per-day | Unofficial, MIT, maintained. Needs a client-version header | Careful; ToS risk |
| [Duffel](https://duffel.com/pricing) | Bookable inventory from 300+ airlines, ancillaries, changes | Self-serve. $3/order; $0.005 per excess search above 1,500:1 | Booking layer, not a feed |
| Amadeus Self-Service | Formerly the standard free GDS sandbox | **Decommissioned 17 July 2026.** Enterprise only | Closed |
| Kiwi Tequila | Virtual-interlining REST API | Invitation-only since 30 May 2024; new keys return 403 | Closed |
| Skyscanner, Sabre, Travelport, Southwest, easyJet | Metasearch or airline inventory | Contract, accreditation or traffic minimums. Skyscanner skews to >100k monthly visitors | Partner-only |

**Round-trip note.** SerpAPI and SearchApi charge two calls for a round trip because the return leg needs a departure token. Budget for that before building the scanner.

**Google's own API.** QPX Express shut down in April 2018. Every "Google Flights API" is a wrapper around the page.

---

## Rail

| Source | What you get | Access and cost | Use |
|---|---|---|---|
| [DB Timetables API](https://developers.deutschebahn.com/db-api-marketplace/apis/product/timetables) | Timetables, station search, real-time changes | Free; register an app; 60 calls/min. No fares | Cleanest European schedule data |
| [db-vendo-client](https://github.com/public-transport/db-vendo-client) | DB fares, connections and booking via the mobile API | Open source, ISC. Unofficial; rate-limited and occasionally blocked | Community bridge, not a contract |
| [National Rail Darwin / LDBWS](https://www.nationalrail.co.uk/developers/darwin-data-feeds/) | GB live departures, platforms, delays, cancellations | Free via Rail Data Marketplace; 5m SOAP calls per 4-week period | Gold standard for UK real time |
| [Realtime Trains](https://www.realtimetrains.co.uk/about/developer/) | Detailed GB running, allocations, history | Free non-commercial; £4–£99/month self-serve | Paid tiers include commercial rights |
| [Trainline Global API](https://tps.thetrainline.com/our-products/global-api/) | Search, booking, aftersales, SplitSave; 270 carriers, 45 countries | Commercial agreement; ~12-week integration; no public docs | Partner-only |
| [Rail Europe RailAPI](https://agent.raileurope.com/) | Live availability and booking across 200+ providers | Trade account; negotiated | Partner-only |
| [Omio affiliate / B2B](https://www.omio.com/affiliate) | Rail, coach, flight and ferry search; deeplinks or white label | Affiliate free; booking API by agreement | Best sanctioned multimodal path |
| [SNCF Navitia](https://numerique.sncf.com/) | Routing, departures, isochrones, real time | Free to 5,000 requests/day. No fares or booking | Routing |
| Trenitalia lefrecce / China 12306 | Italian and Chinese rail inventory | Reverse-engineered / no authorized third-party API | Avoid |
| [Ekispert](https://api-info.ekispert.com/plan/), [Jorudan](https://norikae.jorudan.co.jp/openapi/), Navitime | Japanese routing, timetables, fare modelling | Free limited tiers; paid from ~¥5,500/5,000 calls | Japan's practical route in |
| IRCTC, Amtrak, JR East | Indian, US and Japanese rail booking | B2B schemes, partner agreements, travel-agency licences | Partner-only |
| [GTFS / Mobility Database](https://mobilitydatabase.org/), [Transitous](https://transitous.org/doc/) | Schedules, stops, real-time updates; free routing | Open; licences vary by agency; 6,000+ feeds | Discovery, never fares |

**There is no global rail fare API.** The honest stack is: schedules from GTFS and official timetable APIs, prices from a retailer or the operator itself, booking on the operator's site.

**Split-ticket tools** (consumer only, no usable public API): TrainSplit, Split My Fare, TrainPal, Railboard, Trainline SplitSave. They are covered in [04-playbook.md](04-playbook.md).

---

## Coach

| Source | What you get | Access and cost | Use |
|---|---|---|---|
| [FlixBus GTFS](https://gtfs.gis.flix.tech/) | Stops, routes and schedules for EU, US, GB networks | Free download, ODbL. No prices | The legitimate Flix route |
| [Flix partner API](https://developer.api.flixbus.com/) | Search, availability, booking, ticketing for Flix and Greyhound | Microsoft Entra SSO, commercial distribution agreement | Partner-only |
| [BlaBlaCar Bus API v3](https://bus-api.blablacar.com/) | Stops, fares, search; deep-link booking | Free key; token auth. Docs stale since 2021 | Nominally open; test first |
| [Omio](https://www.omio.com/corporate/omio-b2b/) | Coach, rail, flight and ferry search and booking | Affiliate free; B2B by agreement; 3,000+ operators | Best multimodal coverage |
| [Wanderu](https://www.wanderu.com/en-us/partners/) | US/Europe bus and rail metasearch | No public API; affiliate 4%, 30-day cookie | Affiliate; separate tickets |
| [Busbud](https://www.busbud.com/en/distribution), [Distribusion](https://www.distribusion.com/) | B2B inventory, white label, 2,000–4,500 carriers | Revenue-share contract; no public spec | Partner-only |
| Megabus, National Express, OurBus | Consumer fares, promos, affiliate links | No public API. Megabus charges ~$4.99 per booking; affiliates ~4% | Promos only |
| Greyhound (US) | Coach inventory | Owned by Flix; same partner gate | Partner-only |
| [Rome2Rio](https://cms.rome2rio.com/documentation/) | Global all-mode route discovery | Closed to new applications; legacy keys may work. Indicative prices | Legacy routing |
| [Mobility Database](https://mobilitydatabase.org/) | 6,000+ GTFS feeds, 100+ countries | Free account, token API, daily checks | Schedule-only |

**Coach price data is the hardest to get legitimately.** Every major source either gates prices behind a B2B contract or expects affiliate traffic. The workable personal path is Omio/BlaBlaCar deeplinks plus the operator's own site for verification.

---

## Deal feeds and alerts

| Source | What it is for | Cost | Notes |
|---|---|---|---|
| [Going](https://www.going.com/premium) (ex-Scott's Cheap Flights) | US-origin mistake fares and cheap fares | Free; $49/year; $199/year Elite | Most established; frequent promo codes |
| [Jack's Flight Club](https://jacksflightclub.com/us) | UK/EU-origin error fares, all cabins | Free; $60/year; $199/year | Strong UK/EU coverage |
| [Secret Flying](https://www.secretflying.com/) | Global error fares, fuel dumps, business class | Free; premium tier | Deals die fast; premium gates some |
| [The Flight Deal](https://www.theflightdeal.com/) | US-hub departures; RSS available | Free; paid real-time tier | Free site lags the paid feed |
| [Fly4Free](https://www.fly4free.com/flights/error/) | Europe and Central/Eastern Europe deals | Free | Good signal-to-noise on error fares |
| [Airfarewatchdog](https://www.airfarewatchdog.com/pricing) | Human-curated US fare drops | Free; $149/year; $249/year | Owned by Kayak/Booking |
| [FlyerTalk — Mileage Run Deals](https://www.flyertalk.com/forum/) | Deepest archive of fare errors | Free | Check daily if you want to be early |
| Telegram, X, Discord | Fastest push for error fares | Free | Telegram is seconds; email is 5–30 minutes behind |

Error fares typically live 30 minutes to a few hours. The practical stack is one regional Telegram channel with push on, one private X list of 15–20 accounts, Google Flights tracking on your top routes, and one paid newsletter as a slower backstop.

Skip paid "exclusive" Telegram groups that are mostly affiliate links, and scroll back through a channel's history before trusting it.

---

## Status feeds (not fares)

| Source | What it covers | Cost | Use |
|---|---|---|---|
| [FlightAware AeroAPI](https://www.flightaware.com/commercial/aeroapi/) | Flight status, positions, tracks, delays, historical | Personal free tier up to $5/month; Standard $100/month minimum | Positioning and irregular operations |
| [OpenSky Network](https://openskynetwork.github.io/opensky-api/rest.html) | ADS-B aircraft states and tracks | Free; OAuth2 since March 2026; credit limits | Positioning, not fares |

---

## Do not build on these

- **Amadeus Self-Service** — decommissioned 17 July 2026; repositories archived.
- **Kiwi Tequila** — invite-only since May 2024.
- **Google QPX** — shut down April 2018.
- **Skyscanner Partner API** — commercial only, effectively >100k monthly visitors.
- **Southwest and easyJet** — no public access; scraping prohibited and enforced.
- **Trainline** — ~12-week commercial integration.
- **Rome2Rio** — no new keys.

Planning around any of these is planning around a door that is closed.

---

## Sources

Vendor pricing and developer pages were preferred over listicles. Key links are inline above; the full evidence list with uncertainty flags is in [11-sources.md](11-sources.md).
