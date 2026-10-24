# Real data plan

**Status:** plan, 23 October 2026. Nothing below is built unless it says so.

**Where we are.** Demo data is gone from the product. The server binary carries no fares; the fixture inventory compiles only for tests. Without `SERPAPI_KEY`, search and Explore answer `source_not_configured`. The web app reads everything from the data plane: fares, airports, coordinates, the buy/wait verdict. So the app now shows real data or nothing, and today it shows nothing for fares.

This plan takes it from "nothing" to "real data on every surface", in order of what unblocks the most.

---

## 0. Game plan: live fares this week

The code path is built and tested end to end against a SerpAPI stand-in (`app/scripts/live-check.sh`). What is missing is a key, a budget and a corridor. In order:

| # | Step | Who | Done when |
|---|---|---|---|
| 1 | Create a SerpAPI account; start on the free tier (250 searches/month) | Owner | A key exists |
| 2 | Put it in `app/.env` (gitignored, loaded at startup): `SERPAPI_KEY=…` and `SERPAPI_DAILY_LIMIT=8` for the free tier | Owner | The file exists; the key is never committed or pasted in chat |
| 3 | Restart the data plane (`cd app && cargo run -p faredesk-api`) | Owner or Claude | `/healthz` shows `serpapi.configured: true` |
| 4 | Run `app/scripts/live-check.sh` (3 searches) | Claude | It prints `PASS` against Google Flights data |
| 5 | Search in the web app; check prices against google.com/travel/flights for the same date | Owner | Totals match within bags and currency |
| 6 | Fix what real responses break (unknown connection airports, bag text, odd times) | Claude | Itineraries drop only with a logged reason |
| 7 | Choose the corridor; write `data/watchlist.yaml`; schedule the collector daily | Owner + Claude | `fare_history` grows by one row per route per day |
| 8 | Buy the plan that fits §3 and raise `SERPAPI_DAILY_LIMIT` | Owner | Collector and users fit inside the daily limit |
| 9 | Day 14: first verdicts on corridor routes | — | When to buy leaves "Collecting baseline" |

Steps 1–6 take an afternoon on the free tier. Step 7 is where the budget starts to matter: 30 routes a day is 900 searches a month, beyond the free tier.


---

## 1. What is real, what is missing

| Surface | Data today | Real? | What it needs |
|---|---|---|---|
| Airport search, map coordinates | OurAirports + mwgg timezones, bundled | Yes | Yearly refresh (`app/scripts/build-airports.mjs`) |
| Fares (search) | SerpAPI Google Flights when a key is set | **Off** | A key and a quota budget (Phase 1) |
| Return trips | Two one-way searches | Yes, once fares are on | True round-trip fares (Phase 6) |
| When to buy | `fare_history` from live searches and the collector | Real but empty | The collector running daily for 14+ days (Phase 2) |
| Route coverage, Explore | Routes with recorded live fares | Real but thin | Candidate graph and validation (Phase 4) |
| Seasonality, fare weather | `fare_history` | Real but empty | Same as When to buy |
| Point-of-sale check | SerpAPI per market | Off with fares | Key; one call per market (budget it) |
| Bag fees | Curated table when the source omits the amount, marked estimated | Curated | Offer-level ancillaries (Phase 5) |
| Fare rules | SerpAPI summary; defaults otherwise | Partial | Offer-level rules (Phase 5) |
| Provider links | Google Flights search URL | Real, indirect | Booking links to the seller (Phase 5) |
| Airport access, MCT | Curated table for 13 airports; live OSRM and Transitous legs | Curated + live | Wider table or derived MCTs (Phase 7) |
| FX, holidays, climate | ECB, Nager.Date, Open-Meteo | Yes | Nothing |

---

## 2. Phases

### Phase 1 — Turn on live fares (day 0, about an hour)

1. Get a SerpAPI key (owner action; keys never go in the repo or chat).
2. Set it on the data plane and the collector: `SERPAPI_KEY`, and `SERPAPI_DAILY_LIMIT` to match the plan bought (see §3). The default limit of 100/day spends the 250-call free tier in under three days.
3. Restart the API. Health shows `serpapi.configured: true`; the web badge reads **Live data**.

**Gate:** a BER–LIS one-way and a BER–LIS return both price live; `/healthz` shows `usedToday` rising by 1 and 2.

### Phase 2 — Start the history collector (day 0, then daily)

When to buy, seasonality and fare weather all read `fare_history`. Live user searches already record daily minima; the collector makes it regular.

1. Pick the corridor: one home airport and 25–30 routes (docs/14 §1.1). Write `data/watchlist.yaml` from the example.
2. Schedule `faredesk-collector` once a day (`docker compose --profile collect run --rm collector`, or the host's cron).
3. Rotate one travel date per route through the next 90 days.

**Gates:** day 14 — the first routes leave "Not enough data" and show a verdict. Day 60 — backtests and track records have enough cases to show.

### Phase 3 — Deploy the split (2–3 days)

Per docs/15: web on Vercel with an `/api/*` rewrite, the Rust API and collector on a container host with SQLite on a volume. Set `VITE_API_BASE` (or the rewrite) and `CORS_ORIGINS`.

**Gate:** the deployed app searches live, the collector writes to the same volume, and a restart keeps history.

### Phase 4 — A real route graph for Explore and coverage (4–6 days)

Today routes are "what we have priced". Build the validated candidate set from docs/14 §2:

1. **Candidates:** Travelpayouts popular destinations from the home airport (free affiliate token), the curated corridor, and user requests.
2. **Validation:** a route is live when a search returns an itinerary in the next 90 days; store first and last validated; recheck dormant routes monthly.
3. **Coverage line:** "42 destinations validated from BER; last checked 6 hours ago."

**Gate:** Explore shows only validated destinations, with a coverage count, and never draws an unvalidated arc.

### Phase 5 — Offer-level detail: bags, rules, seller links (5–7 days)

SerpAPI's list response summarises bags and rules and links to Google Flights. For the itinerary a traveller selects:

1. **Booking options:** fetch SerpAPI's booking options for the selected itinerary to get the actual sellers and their links, so "Open on TAP" opens TAP, not a Google search.
2. **Ancillaries and rules:** price the selected itinerary through Duffel (search is free up to its search-to-order ratio) to get exact bag fees and fare conditions, and retire the curated bag table for those offers.

**Gate:** a selected itinerary shows a named seller link, exact bag fees (no "estimated" flag) and refund/change rules from the offer.

### Phase 6 — True round-trip fares (3–4 days)

Two one-ways can cost more than a return fare. Add round-trip pricing for the selected outbound: SerpAPI `type=1` with the outbound's departure token returns return flights priced as a pair. One extra metered call per outbound the traveller inspects; show both "two one-ways" and "round-trip fare" and let the cheaper win.

**Gate:** for a legacy-carrier route, the return fare and the two-one-way total are shown side by side, each with its seller.

### Phase 7 — Widen curated reference data (ongoing)

- Airport access and MCT: extend `data/access.json` beyond 13 airports for the corridor's airports; label curated rows as such.
- Bag fees: keep the table only as a fallback where Phase 5 has no offer.
- Airports: refresh the dataset each quarter.

### Phase 8 — Watch the data (2 days)

- Quota: alert at 80% of the daily limit; show the day's usage on `/healthz` (exists) and in a small ops page.
- Freshness: alert when the source degrades (stale answers) for more than an hour.
- Coverage: weekly count of routes with 14+ observations.

---

## 3. Budget

Assumptions: 30 collector routes a day, 50 user searches a day with a third of them returns, Explore cached for six hours for one home airport.

| Use | Calls / day | Calls / month |
|---|---:|---:|
| Collector | 30 | 900 |
| User searches (1 per one-way, 2 per return) | ~67 | ~2,000 |
| Explore validation and feed | ~25 | ~750 |
| Point-of-sale checks (up to 5 markets each, ~5/day) | ~25 | ~750 |
| **Total** | **~150** | **~4,400** |

| Provider | Plan that fits | Cost / month |
|---|---|---:|
| SerpAPI | 5,000 searches | ~$75 |
| SearchApi (same Google Flights surface) | ~5,000 searches | ~$20 |

Set `SERPAPI_DAILY_LIMIT` to the plan's monthly allowance divided by 30. Searches are cached for ten minutes, so repeated searches cost nothing. Prices are from docs/02, checked October 2026; confirm on the provider's page before buying.

**Cheaper later:** a SearchApi adapter behind the same `Source` enum cuts the bill roughly by three for the same data. It is a small, contained change once Phase 1 proves the flow.

---

## 4. Decisions for the owner

1. **Provider:** SerpAPI (what the adapter speaks today) or SearchApi (cheaper, needs an adapter).
2. **Monthly budget:** the free tier (8 calls a day) is only enough to test Phase 1.
3. **Corridor:** the home airport and the 25–30 routes the collector watches.
4. **Hosting:** which container host for the API and collector (Fly.io, Render, Railway).
5. **Duffel:** whether to open an account for Phase 5 offer detail.

---

## 5. Order and timing

| Week | Work | What the traveller sees |
|---|---|---|
| 0 | Phases 1–2 | Live fares everywhere; "Collecting baseline" on When to buy |
| 1 | Phase 3 | The live app at a public URL |
| 2 | Phase 4 | Explore with validated destinations; first verdicts on corridor routes |
| 3 | Phase 5 | Exact bags, rules and seller links on a selected flight |
| 4 | Phase 6, 8 | Round-trip fares beside two one-ways; quota and freshness alerts |
| 8+ | — | Backtests and track records with real cases |
