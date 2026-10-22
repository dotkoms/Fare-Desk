# Build order

Build the ledger before the agents. A fleet without a memory cannot tell a deal from a price.

Each phase is useful on its own, and each one is cheaper than the phase after it. Stop wherever the returns stop; most people get most of the value by phase 2.

| Phase | What you build | Cost | Done when |
|---|---|---|---|
| **0 — Days 1–2** | Trip-brief template, a SQLite ledger, and a watchlist of 10–25 routes with date windows. Manual Google Flights alerts on the top five. | $0 | You can answer "what did this route cost on each of the last 30 days?" |
| **1 — Week 1** | Air collector: fast-flights for wide scans, Travelpayouts for cached baselines. Deterministic normalization into the offer schema. No model in the path. | $0 | A daily scan writes clean, fingerprinted rows to the ledger. |
| **2 — Week 2** | Verifier and notifier: SerpAPI re-price, total-cost assembly, dedupe, per-channel alert budget. This is where the system becomes trustworthy. | ~$25/month | Every alert has a verified total, a timestamp and a link that resolves. |
| **3 — Week 3** | Ground collectors: db-vendo-client for DB fares, FlixBus GTFS for schedules, BlaBlaCar Bus and Omio deeplinks for prices. Rail and coach enter the same schema. | $0 | A trip can be priced in all three modes from one brief. |
| **4 — Week 4** | Arbitrage analyst: enumerate split, positioning, open-jaw and multimodal structures; score on generalized cost with a risk premium. | Compute only | The cheapest itinerary is often not the cheapest fare — and the system can say why. |
| **5 — Later** | Points strategist and deal-feed watcher. Add only if you actually hold transferable points and will act within minutes. | $0 | Cash-vs-award comparisons use your valuation and include surcharges. |
| **6 — Maybe never** | Booking automation through Duffel. The fleet's job is to find and capture; transacting automatically removes the human check. | $3/order | Only if the manual step is genuinely the bottleneck. |

---

## If you build nothing else

Set price alerts on your five most-flown routes, keep a simple spreadsheet of what you actually paid, and always run a split check before buying a UK or European rail ticket. That captures most of the available savings with none of the engineering.

---

## What not to build

- **Your own Southwest or easyJet scraper.** Both prohibit it and have enforced it. Use Duffel or a GDS if you need the content.
- **A fare-history database for every route.** Store only the routes you watch. Broad history is expensive to collect and stale by the time it is interesting.
- **An LLM that reads prices off a page.** Deterministic extraction, model explanation. That is the difference between a fare and a guess.
- **A booking bot.** The last mile has the most liability and the least savings.
- **A consumer app.** Republishing airline fare content is the legal exposure that cost Skiplagged $9.4M. A personal tool that links to the source is a different risk class.

---

## Minimum viable ledger

One table is enough to start:

```sql
CREATE TABLE fares (
  route        TEXT,
  travel_date  TEXT,
  mode         TEXT,
  fingerprint  TEXT,
  price        REAL,
  currency     TEXT,
  source       TEXT,
  observed_at  TEXT,
  verified_at  TEXT,
  PRIMARY KEY (route, travel_date, mode, fingerprint, observed_at)
);
```

Daily query: the minimum price per route and travel date over the trailing 90 days. That is your baseline. Add a `notified_at` column when you start alerting.

## Success metric

Count **verified deals per week**, not alerts sent. An alert feed that fires often but never converts is just noise. Track the median price you paid against the trailing baseline for the same route; that number is the only honest scoreboard.
