# The agent fleet

A fleet earns its keep here for three reasons. Sources have separate quotas and separate failure modes, so one dead scraper should not stop the scan. Discovery and verification want different temperaments: one should cast absurdly wide, the other should trust nothing. And the biggest savings come from **itinerary structures** — split tickets, positioning legs, mixed modes — which is a search over combinations, not a lookup.

The rule that keeps it honest: **the language model stays out of the price path.** Deterministic code extracts, normalizes, fingerprints and scores. The model parses intent ("ski trip, flexible, happy to position through Milan") and explains why a fare is good. An LLM reading a price off a page is how you ship a hallucinated fare — the exact failure the FTC fined Hopper $35M for in 2026.

---

## Architecture

```
                 ┌──────────────────────────────────────────────┐
                 │                  desk-ops                    │
                 │   brief · dispatch · budget · synthesis      │
                 └───────┬──────────┬──────────┬────────────────┘
                         │          │          │
                 ┌───────▼──┐ ┌─────▼─────┐ ┌──▼─────────┐
                 │ air-scout│ │ rail-scout│ │ coach-scout│
                 └───────┬──┘ └─────┬─────┘ └──┬─────────┘
                         │          │          │
                         └──────────┼──────────┘
                                    ▼
                          ┌───────────────────┐
                          │  offer normalizer │
                          │  fingerprinting   │
                          └─────────┬─────────┘
                                    ▼
                    ┌───────────────────────────────┐
                    │ arbitrage-analyst             │
                    │ split · position · open-jaw   │
                    │ multimodal · generalized cost │
                    └───────────────┬───────────────┘
                                    ▼
                          ┌───────────────────┐
                          │   fare-verifier   │
                          │ re-price · total  │
                          │ link · risk flags │
                          └─────────┬─────────┘
                                    ▼
                    ┌───────────────────────────────┐
                    │ watchkeeper  →  alerts        │
                    │ dedupe · budget · ledger      │
                    └───────────────┬───────────────┘
                                    ▼
                          ┌───────────────────┐
                          │      ledger       │
                          │ baseline · history│
                          └───────────────────┘
                                    ▲
                          points-strategist
                          (cash vs award, on demand)
```

The ledger feeds tomorrow's baseline. That loop is what makes "cheap" measurable.

---

## The roster

| Agent | Job | Reads | Writes | Fails when |
|---|---|---|---|---|
| [desk-ops](../agents/desk-ops.md) | Turn a request into a trip brief, scan plan and budget; merge specialists into ranked options | Request, ledger, quota, reports | Trip brief, dispatch, shortlist | It accepts an unverified price into the shortlist |
| [air-scout](../agents/air-scout.md) | Wide scan: flexible dates, nearby airports, fare buckets, anywhere-search | Brief, watchlist, baselines | Candidate offers with source and time | It treats a scraped price as bookable |
| [rail-scout](../agents/rail-scout.md) | Find the cheap quota, the split point and the cheaper operator | Brief, advance windows, card rules | Fare legs, split options, reservations | It prices only the through-fare |
| [coach-scout](../agents/coach-scout.md) | Read the coach yield curve, promos, terminals and night-leg economics | Brief, distance, hotel baseline | Coach legs with bags, fees, transfers | It ignores bags, fees or terminal transfer |
| [arbitrage-analyst](../agents/arbitrage-analyst.md) | Generate structurally different itineraries and score generalized cost | All candidate legs, value of time, bags | Ranked structures with risk premium | It compares fares instead of door-to-door cost |
| [fare-verifier](../agents/fare-verifier.md) | Re-price at the source, assemble the total, resolve the link, flag risk | Shortlist, fingerprints, policy | Verified offer with stamp and warnings | It verifies against another cache |
| [watchkeeper](../agents/watchkeeper.md) | Run the watchlist and deal feeds, dedupe, enforce the alert budget, keep baselines | Watchlist, ledger, feeds | Alerts, ledger rows, baselines | It alerts on every poll |
| [points-strategist](../agents/points-strategist.md) | Compare cash against awards, transfers, surcharges and stopovers | Cash total, award space, balances | Net-CPP comparison and recommendation | It values points at a headline rate |

---

## Shared contract

Four schemas are enough. Write them down before writing any agent.

### Trip brief

```yaml
trip_brief:
  origin: BER
  destination: LIS
  date_window: 2027-03-01/2027-03-08
  pax: 1
  bags: 1 carry-on + 1 checked
  flexibility: days        # hours | days | weeks
  must_arrive_by: null
  max_door_to_door_hours: 12
  value_of_time_per_hour: 15
  modes: [air, rail, coach]
  constraints: [no_red_eye]
```

### Offer

```yaml
offer:
  id: string
  mode: air | rail | coach
  source: string
  fetched_at: ISO-8601
  expires_at: ISO-8601
  currency: EUR
  total: 0.0
  segments: []
  fare_rules: {}
  ancillaries: []
  booking_url: string
  fingerprint: string       # not the source's offer id
```

### Fingerprint

```text
fingerprint = hash(
  marketing_carrier + operating_carrier +
  service_number + travel_date + departure_time_to_minute
)
```

Offer IDs expire in 15–30 minutes. Key anything on them and dedupe breaks every hour. Key on the physical service and the same flight from Google, Kiwi and Skiplagged collapses to one row; keep the lowest total.

### Quota

```yaml
quota:
  source: serpapi
  window_start: ISO-8601
  spent: 0
  ceiling: 250
  reserved_for_verification: 150
```

Discovery may not eat the verification reserve. One shared broker, not per-agent limits: six agents each believing they own 250 free searches will burn the month by Tuesday.

### Ledger

```yaml
ledger:
  route: BER-LIS
  travel_date: 2027-03-04
  mode: air
  fingerprint: string
  price: 0.0
  p10: 0.0
  p50: 0.0
  p90: 0.0
  currency: EUR
  first_seen: ISO-8601
  last_seen: ISO-8601
  notified_at: ISO-8601 | null
  verified_at: ISO-8601 | null
  verified_total: 0.0
```

---

## Cadence and budget

| Job | Frequency | Cost | Why this rate |
|---|---|---|---|
| Deal feeds (error fares, promos) | 10–15 min | Free | These are minutes-to-hours events; the feed is the fast path |
| Wide route scan | 2–3×/day | Free scrapers | Planning fares move in steps, not seconds |
| Baseline refresh | Daily | Free cached API | You need the distribution, not a live quote |
| Candidate verification | On demand | ~$0.01–0.03 each | Only the shortlist deserves a paid call |
| Watchlist re-check | 2×/day | Free + a few paid | Re-alert only on a new low or a ≥15% drop |

A realistic personal setup: 25 routes × 3 scans × 30 days is 2,250 free scrapes, with roughly 150 paid verifications a month — the $25 tier. The mistake is paying to scan. **Scan cheap, verify paid.**

---

## Guardrails

- **No booking, changing or cancelling without explicit approval.** The fleet finds and captures; you transact.
- **Total cost or nothing.** Bags, seats, booking fee, terminal transfer and the value of time go on every line.
- **Verified within 30 minutes of the alert.** A deal older than that is a story, not an offer.
- **Hidden-city warnings are mandatory.** Carry-on only, no return on the same ticket, no loyalty number — and say so in the alert.
- **Self-transfer risk is priced, not hand-waved.** Separate tickets mean you own the misconnection; show the buffer and the miss probability.
- **Link to the source, never republish fares.** Fetching public data is one risk; republishing airline content is the one that cost Skiplagged $9.4M.
- **One quota broker.**
- **No LLM in the price path.**

---

## Installing

These are Claude Code subagent definitions. Copy them into the **project** where you do travel planning, not the global config, so the fleet only wakes when you need it:

```bash
mkdir -p .claude/agents
cp agents/*.md .claude/agents/
```

Then invoke the orchestrator in plain language:

> Use desk-ops to plan Berlin → Lisbon, 1–8 March 2027, one checked bag, flexible ±2 days, value my time at €15/hour. Give me the three best itineraries on total cost, and verify the top one.

For continuous watching, the agents are prompts, not daemons. Run `watchkeeper` from a scheduled task, cron job, or Claude Code's `/loop`, with the ledger on disk between runs.

---

## What the fleet cannot do

- Watch every route. Pick 10–25.
- Guarantee a fare. It can only verify one.
- Replace the airline's booking flow.
- See inventory that no source exposes: negotiated corporate fares, unpublished agency fares, most LCC seats through APIs.
- Fix a bad itinerary structure. If the brief is wrong, the answer will be wrong.
