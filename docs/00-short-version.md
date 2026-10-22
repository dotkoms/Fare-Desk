# The short version

Six rules explain most of the savings. Everything else in this repo is evidence or machinery.

## Six rules

### 1. The itinerary is the lever
Changing **how** you fly beats changing **when**. Split tickets, positioning legs, secondary airports and mixed modes routinely cut 30–60% where a date change cuts 5–15%.

### 2. Price the whole trip
A $40 fare with a $45 bag and a $30 airport transfer is a $115 fare. Compare **total cost and total time**, door to door, or you are optimizing the wrong number. See the generalized-cost formula in [04-playbook.md](04-playbook.md).

### 3. Search wide, verify narrow
Free, fragile sources are for discovery. Paid, stable sources are for the price you actually quote. Never alert off a cache.

### 4. Book against a baseline
"Cheap" means below the route's own median for that season. A fare is only a deal relative to a 90-day or 12-month distribution you keep. Without a ledger, you are guessing.

### 5. Timing is a distribution, not a rule
Advance-purchase windows and fare-bucket releases are real. "Tuesday at midnight" and "always last minute" are not. Watch the route; don't guess the calendar. Full detail: [05-when-to-buy.md](05-when-to-buy.md).

### 6. The floor is structural
A fare below airport taxes plus the carrier's marginal cost is either an error, a stale cache, or a misparse. Verify before you celebrate — and before you book anything non-refundable.

---

## The five highest-ROI moves

1. **Search the whole week, not one date.** Calendar and flexible-date views expose the days when the cheap fare bucket is still open.
2. **Add every nearby airport.** Metro gaps average 15–25%; outliers reach 60%. Subtract the ground transfer before believing it.
3. **Run the rail split check.** On UK long-distance routes a split is found on roughly 60% of journeys and saves 20–30% when it exists.
4. **Compare night coach or sleeper against a flight plus hotel.** On 6–12 hour legs the hotel night is the real saving.
5. **Price total cost and total time.** This single habit prevents most fake deals.

---

## What “cheap” means

A fare is cheap when it is below the route's own recent distribution for comparable dates and cabin. Practical baseline:

- Store the daily minimum for the exact route and date window you care about.
- Compare against the trailing 90-day median for shoulder season, and the same month last year for peak dates.
- Treat anything below the 10th percentile as a candidate, not a deal, until it is re-priced at the source.

Alert thresholds that work: a new low, or a drop of 15–20% below the trailing median. Not "any change".

---

## What this repo is not

- Not a booking bot. The agents find and verify; you buy.
- Not a source of guaranteed prices. Fares move by the minute.
- Not legal advice. Hidden-city and throwaway ticketing are contract breaches even where they are not crimes.
- Not a complete inventory. No public API covers all low-cost carriers and OTAs; that gap is the reason scraping exists, and the reason to treat scraped prices as leads.
