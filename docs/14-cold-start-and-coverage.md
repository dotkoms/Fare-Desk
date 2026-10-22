# Cold start and route coverage

Two problems can make a flight app look finished while being empty underneath:

1. **No history** — buy/wait advice needs weeks of observations, and fare APIs sell today, not the past.
2. **No current route graph** — Explore needs to know which destinations actually operate from a home airport, and the free route datasets are stale.

This document gives the practical fixes, in the order to build them.

---

## 1. Cold-start history

### Why it is hard

- Fare APIs return current offers, not a historical time series.
- Cached affiliate data is days old and sparse, not a daily panel.
- Public fare datasets are quarterly averages, not daily quotes.
- Scraping the Wayback Machine or old fare pages is unreliable, against terms, and produces numbers you cannot defend.

You cannot buy three months of route-level daily history at a price a hobby project can afford. So the plan is to **start early, seed honestly, borrow strength, and refuse to advise until the evidence exists**.

### 1.1 Shadow collection before launch

Start the collector before the UI is finished.

- Pick **30 routes** from one home airport: the routes you would actually fly, plus a few long-haul and LCC-heavy ones.
- Query **once a day**, one travel date per route, rotating through the next 90 days.
- Store the daily minimum per route and travel date.
- Do not verify every observation; verify only the candidates you would show.

Budget: 30 routes × 90 days is 2,700 searches. At roughly $25 per 1,000, that is about **$68 for a three-month baseline**, before round-trip doubling. Sampling only a third of the routes each day cuts it further.

By launch you have 60–90 days of history on the routes that matter. Everything else starts as a watch page.

### 1.2 Seed from data that already exists

Use these as **priors**, not as daily history:

| Source | What it gives | Use |
|---|---|---|
| Travelpayouts / Aviasales Data API | Cached cheap fares, price calendar, popular destinations | Seed the route baseline and seasonality; label as cached |
| Google Flights price insights via SerpAPI | Typical range and history where the source exposes it | Seed a route prior; verify before display |
| US BTS DB1B | Quarterly 10% ticket sample, average fare by city pair | US route existence, average fare, seasonality |
| US DOT Consumer Airfare Report | Quarterly city-pair average fares | US baseline and seasonality |
| Your own community submissions | What people actually paid | Supplement, never the verified price |

Do not pretend a quarterly average is a daily series. Keep the source label on the number.

### 1.3 Borrow strength across routes

A new route is not a blank slate. It resembles other routes.

**Route classes.** Group by:

- distance band (short-haul, medium, long-haul)
- region pair (intra-Europe, transatlantic, US domestic, US–Asia)
- carrier mix (LCC-dominated, legacy-dominated)
- season (peak, shoulder, off-peak)

Pool the days-to-departure curves within a class. That gives a prior for a route with no history.

**Empirical Bayes.** Blend the route's own observations with the class prior, weighted by sample size and variance:

```
estimate = w × route_estimate + (1 − w) × class_prior
w = n / (n + k)
```

`k` is the strength of the prior. Tune it on routes with enough history: hold out the last 30 days and see which `k` predicts best.

**Cross-sectional curve.** You cannot go back in time, but you can sample different lead times today:

- search travel dates 30, 60 and 90 days out
- fit a curve with date fixed effects
- use the shape as a prior, not as a route fact

This is cheaper than waiting three months, and it is honest if labelled as a prior.

### 1.4 Honest UX

The product must be allowed to say “not yet.”

- **0–13 observations:** “Collecting baseline — 8 of 14 days.” Show the class prior separately.
- **14–29 observations:** advice with **low confidence**, and a clear note that the route estimate is still mostly prior.
- **30+ observations:** normal confidence.
- **No comparable lead-time data:** no advice, only a watch.

Show the sample count and the split between prior and route data. A confidence number without a sample count is theatre.

### 1.5 Backtest before launch

Use the shadow period as the test set:

- Replay the advice at 60, 30 and 14 days out.
- Compare with buying at that moment.
- Publish the wait-win rate, average saving, and worst loss.

If the advice does not beat “buy now” after fees, launch as search plus alerts and say so. That is a better product than a confident guess.

### 1.6 What not to do

- Do not invent history from a single snapshot.
- Do not scrape old fare pages or the Wayback Machine.
- Do not present a quarterly average as a daily baseline.
- Do not show a recommendation on a route with three observations.
- Do not hide the prior share of the estimate.

---

## 2. Current route coverage for Explore

### Why it is hard

Explore asks: “from my airport, where can I go cheaply?” That needs a **current route graph** — which destinations operate, on which days, in which season.

The free options are weak:

- **OpenFlights routes** is a 2014 snapshot. It is fine for a demo globe, not for a production Explore map.
- **OurAirports** has airports, not routes.
- **OpenSky** observes aircraft, not schedules; inferring routes is possible but heavy and incomplete.
- **Wikipedia destination tables** are current-ish but inconsistent, and scraping them is a maintenance and terms problem.
- **OAG, Cirium, Amadeus** have the real schedules; they are paid and contract-only.

So: **do not draw a global route map from a stale dataset.** Build a validated candidate set instead.

### 2.1 Candidate generation

Use several cheap sources, then validate:

| Source | What it gives | Role |
|---|---|---|
| **Travelpayouts popular destinations / latest prices** | Destinations with cached fares from an origin | Candidate list |
| **Skiplagged MCP `destinations_anywhere`** | “Anywhere” destinations from an origin | Candidate list |
| **Curated corridor** | 30–50 routes chosen by hand from the home airport | Launch set |
| **User requests** | “Add my destination” | Growth loop |
| **BTS DB1B (US)** | City-pair routes and average fares | US route existence and baseline |
| **Airline destination pages** | Current destinations per carrier | Manual curation |

### 2.2 Validation loop

A route is **live** only when a search returns at least one itinerary in the next 90 days.

```
candidate → cheap search → at least one itinerary?
   yes → store route, first/last validated, price sample
   no  → drop or mark dormant, recheck in 30 days
```

Cache the validation for a week. Recheck dormant routes monthly. Do not re-search everything every day; Explore is a cached surface.

### 2.3 Explore algorithm

1. Take the candidate set for the home airport.
2. Filter to validated routes.
3. For each, fetch the cheapest current fare and the route's baseline (or class prior).
4. Score by discount versus baseline, with a minimum sample rule.
5. Draw only the top N arcs, coloured by discount, with the price and percentile in the panel.
6. Show coverage: “42 destinations validated from BER; last checked 6 hours ago.”

This is smaller than a world map and far more honest.

### 2.4 Coverage metrics

Publish, on the Explore page and in the data card:

- destinations validated
- destinations with a usable baseline
- last validation time
- median age of the price observations
- routes dropped as dormant in the last 30 days

If coverage is thin, say so. “We cover 38 destinations from Berlin” is a feature; a map full of dead routes is a bug.

### 2.5 Expanding beyond one corridor

In order:

1. **US:** BTS DB1B and DOT city-pair data give route existence and quarterly fares; add live search for current prices.
2. **Europe:** no equivalent open route-fare dataset; use curated corridors and Travelpayouts candidates, validated by search.
3. **Global:** license OAG/Cirium/Amadeus, or build a community-maintained route graph with validation. Until then, coverage is explicitly regional.

Never let the globe imply global coverage the data does not support.

---

## 3. The combined launch plan

| When | Action |
|---|---|
| **T-12 weeks** | Start shadow collection on 30 routes |
| **T-8 weeks** | Seed class priors from Travelpayouts, BTS (US), and cross-sectional samples |
| **T-4 weeks** | Backtest the advice; publish the method and the result |
| **T-2 weeks** | Validate the Explore candidate set; drop dormant routes |
| **Launch** | Show advice only where the sample supports it; show coverage counts |
| **T+4 weeks** | Add community fee/access data and the first user-requested routes |

The cold start is not a reason to delay the UI. It is a reason to start the collector before the UI is ready.

---

## 4. Sources

- [Travelpayouts Data API](https://support.travelpayouts.com/hc/en-us/articles/203956163)
- [SerpAPI Google Flights](https://serpapi.com/google-flights-api)
- [BTS DB1B](https://www.bts.gov/topics/airlines-and-airports/airline-origin-and-destination-survey)
- [DOT Consumer Airfare Report](https://www.transportation.gov/policy/aviation-policy/domestic-airline-consumer-airfare-report)
- [Skiplagged MCP](https://skiplagged.github.io/mcp/)
- [OpenFlights data](https://openflights.org/data.html) — 2014 snapshot; demo only
- [OurAirports](https://ourairports.com/data/)
