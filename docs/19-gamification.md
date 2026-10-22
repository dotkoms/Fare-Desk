# Gamification

**Thesis: gamify the decision, not the purchase.** Fare Desk is used a few times a year for a high-stakes choice. A daily streak imported from Duolingo would manufacture a habit the product does not need and cannot honestly reward. The game layer should make the trade-offs legible, reward better calls, and then get out of the way.

A useful test for every mechanic: **would this still be worth building if the user booked nothing?** If yes, it is probably aligned. If no, it is probably extraction.

---

## 1. What the evidence says

| Product / case | What worked | What went wrong | Lesson for Fare Desk |
|---|---|---|---|
| **Duolingo** | 7-day streaks correlate with course completion; friend streaks and streak freezes improve retention (company A/B tests) | Streak anxiety; loss aversion; paid power-ups; users ending streaks in protest | Streaks fit daily, identity-relevant behaviour. Flights are episodic. Do not copy the cadence. |
| **Strava** | Segments, kudos and challenges sustain a fitness community | Public leaderboards were gamed; the 2018 heatmap exposed sensitive locations | Social must be opt-in, friend-only, aggregate, and off by default. |
| **Robinhood** | Engagement mechanics drove growth | Confetti, lottery-style free stock and push notifications were cited in a $7.5M enforcement case | Celebrating a transaction is the wrong emotion. Regulators treat engagement design as conduct. |
| **Flighty** | A private Passport of flights, airports and memories; friend-only comparison | Travel-volume status can creep in | A private collection is a good model. Never make it a completion checklist. |
| **Hopper** | Prediction plus monitoring built a brand | $35M FTC settlement over pre-selected fees and undisclosed Price Freeze limits | Utility can coexist with dark patterns, but trust does not survive them. |
| **Delta SkyMiles** | Loyalty tiers drive behaviour | Spend-only status triggered backlash and a public retreat | Never tier users by spend, trips, cabins or bookings. |
| **Going** | Retains users on alerts alone, with no game layer | — | Gamification is optional. If it does not improve decisions, do not ship it. |

**Streak evidence.** Silverman and Barasch (2023) found that showing an intact streak increases engagement, while a broken streak demotivates — especially when the user blames themselves. Allowing repair weakens the negative effect. The mechanic is real; it is also a constructed goal that can detach from the outcome.

**Notification evidence.** Users receive dozens of notifications a day; increasing non-personal push frequency raises uninstalls and lowers direct open rates. Alerts must be opt-in, threshold-based, capped, and quiet-hours aware.

**The one metric that matters:** verified savings against the decision-time baseline, with regret and pressure as guardrails. Engagement is a constraint, not the goal.

---

## 2. Fit

### Strong fit

- **Fare Receipt** — the total, baseline, percentile, sample size, source and verification time.
- **Price Position** — “12th percentile, N=84 observations”, translated to “7 in 10 travellers paid more”.
- **Buy / Wait / Watch call** — with a range, sample and track record, and “what would change our mind”.
- **Decision Journal** — log the call before booking; score it after the trip.
- **Mastery path** — six real tasks that teach the rules, not a quiz.
- **Personal bests** — best percentile, best total-cost gap, most bags included.
- **Private Route Passport** — a memory of routes and airports, not a completionist checklist.
- **Verified savings ledger** — what the user actually paid versus the decision-time baseline.
- **Post-trip No-Regret Review** — a neutral debrief that closes the loop.
- **Optional friend circle** — aggregate savings versus baseline, no individual rank.

### Gray zone, only with guardrails

- **Streaks** — finite, weekly, auto-pausing, free repair, never gating features.
- **Leaderboards** — opt-in, friend-only or calibration-based, never public by default.
- **Badges and levels** — only for real skills or verified outcomes; sparse and explainable.
- **Notifications** — verified, threshold-based, capped, quiet hours.
- **Collections** — private, non-completionist, never rewarding more flights.

### Misaligned

- Fake countdowns, “only 2 seats left” without live inventory, “price rising” without a calibrated probability.
- Public savings leaderboards, spend-based tiers, “top traveller” rankings.
- Confetti or lottery mechanics on booking.
- Paid streak freezes, paid badges, paid leaderboard boosts.
- “Collect all countries/airports” challenges.
- FOMO pushes, “you haven’t searched in 3 days”, sad-mascot guilt.
- Commission-biased “book now” prompts.
- Mystery fares or randomised discounts.

### Trivial

Daily check-ins, avatars, virtual currency, travel-personality quizzes, badges for opening the app, confetti for searching.

---

## 3. Roadmap

### MVP — decision-first, local-only, no accounts

Ship after buy/wait and fare weather, before accounts or social features. Everything runs locally.

1. **Fare Receipt** on every itinerary. Total cost, baseline, percentile, sample size, verification age, source, “what would change this price”.
2. **Price Position** badge. “12th percentile vs 90-day median (N=84)”, not a bare number.
3. **Buy / Wait / Watch call** with a range, sample and track record, and “what would change our mind”.
4. **Decision Journal.** Save the recommendation, quoted price and decision locally; return after the trip to log the outcome.
5. **Six Rules Mastery Path.** Six real tasks: whole week, nearby airports, total cost, baseline, verify, wait. Six badges and a simple progress bar.
6. **First Watch.** Set a watch, preview the alert, choose email later; no marketing push.
7. **Shareable redacted receipt.** The user chooses what to show; dates hidden by default.

**Proves:** do receipts and calls improve understanding and savings? Does the journal bring users back after the trip without pressure?

### Next — optional accounts, verified outcomes, forgiving social

1. **Savings Ledger.** Verified booking confirmations or affiliate postbacks; self-reported entries labelled separately; personal bests.
2. **No-Regret Review.** Post-trip debrief comparing paid price to the decision-time baseline and the lowest comparable fare after booking.
3. **Trip Season.** A finite four-to-eight-week planning season with weekly missions, auto-pause, one flex week and free repair. Not a daily streak.
4. **Route Passport.** Private collection of routes, airports and trips; memory, no completion pressure.
5. **Real-Trip Challenges.** Wide Search, Nearby Airport, Total Cost, Baseline. Rewards mastery, not searches.
6. **Savings Circle.** Opt-in friends or co-op, aggregate savings versus baseline, no individual rank.
7. **Notification digest.** Maximum three per week, quiet hours, verified alerts only.

**Proves:** can verification and social accountability improve outcomes rather than just engagement?

### Later — public calibration and network effects

1. **FareBench calibration leaderboard.** Opt-in, scored by Brier score or reliability, not savings.
2. **State of Airfare / Year in Travel.** Public report and private annual recap; receipts, not bragging.
3. **Community “What people paid”.** Opt-in, aggregate, minimum sample, privacy-preserving.
4. **API/MCP receipt.** Let other agents cite Fare Desk’s fare evidence and savings methodology.
5. **Savings Circles at scale.** Group goals, no public profiles, no spend status.

---

## 4. Twelve mechanics

| # | Mechanic | User action → reward | Risk | Tier |
|---|---|---|---|---|
| 1 | **Fare Receipt** | Open an itinerary → total, baseline, percentile, source, verification time; share a redacted receipt | Information overload; false precision | MVP |
| 2 | **Price Position** | View percentile vs 90-day median → “below baseline” badge and sample size | Thin data makes percentile meaningless; always show N | MVP |
| 3 | **Buy / Wait / Watch** | Ask for a call → recommendation, range, sample, track record, “what would change our mind” | Overconfidence; liability; must show uncertainty | MVP |
| 4 | **Decision Journal / Called It** | Log a call before booking → score it after the trip; build calibration | Hindsight bias; small samples | MVP |
| 5 | **Six Rules Mastery Path** | Complete real tasks → six badges and tool unlocks | Grind; implies expertise; do not gate useful data | MVP |
| 6 | **Personal Bests** | Track best percentile, best total-cost gap, most bags included | Vanity; small sample; never compare publicly | MVP |
| 7 | **No-Regret Review** | After the trip, compare paid price to baseline and later fares → “No-Regret Trip” or a learning note | Negative affect; incomparable fares | Next |
| 8 | **Savings Ledger** | Confirm booking price → verified savings total | Verification gaps; self-report bias; privacy | Next |
| 9 | **Trip Season** | Weekly planning missions for one trip → season progress and a clean end | Guilt if it becomes a daily streak | Next |
| 10 | **Route Passport** | Save routes, airports and trips → private map and memories | Travel-volume status; carbon; privacy | Next |
| 11 | **Savings Circle** | Opt-in friends or co-op → group savings goal, aggregate only | Comparison, small N, gaming, privacy | Next |
| 12 | **FareBench Calibration Leaderboard** | Predict buy/wait on fixed snapshots → Brier score and reliability chart | Research toy; leaderboard pressure; not for consumers | Later |

**Badges:** Baseline Set, Wide Net, Total Cost, Bag Math, Nearby Airport, Receipt Reader, Waited Well, Called It, No-Regret Trip. Sparse, explainable, outcome-based. No badges for opening the app, searching, or spending.

---

## 5. Streaks

**Do not ship a daily streak in the MVP.** If a streak is tested later, make it a finite **Trip Season**:

- Four to eight weeks around a real trip, then a clean end.
- Auto-pause when there is no active trip.
- One flex week per season, automatically, free.
- One free repair, or reset without penalty. Never sell repair.
- Neutral copy: “3 weeks of watching. Paused until your next trip.” Never “Don’t lose your 21-week streak!”
- No red flame, sad mascot, or loss-framed push.
- Never gate alerts, history or savings behind streak status.
- Measure anxiety and regret, not streak length.

Do **not** build a savings streak. Savings are lumpy, route-dependent and partly luck; a streak would reward waiting too long or choosing inconvenient itineraries. If a streak exists, make it a **decision streak** (“I logged a buy/wait call before booking”), not an outcome streak.

---

## 6. Leaderboards and social

**Default: off.** Travel dates, home airport, destinations and companions are sensitive.

**Rank by savings percentage versus the route’s own baseline, not total dollars.** Absolute savings favour expensive trips. Report the sample size. Never rank by spend, trip count, countries, cabins or bookings.

Better designs:

- **Personal bests** as the default: “Your best find: 12th percentile on BER–LIS.”
- **Friend co-op circle:** a group goal (“our circle beat baseline by 14% this year”), no individual rank.
- **Most improved** or **best call**, not “top saver”.
- **FareBench:** a public, opt-in **calibration** leaderboard scored by Brier score or reliability, not savings.
- Minimum group size (e.g., five), aggregate-only visibility.
- No routes, dates, home airport or companion data in a shared view.
- Easy leave, delete and export.

**Metrics:** opt-in rate, retention of participants versus holdout, perceived fairness, pressure, privacy complaints. If the circle increases pressure without improving savings or regret, kill it.

---

## 7. Notifications

Value test:

```
value = P(alert is actionable) × expected savings − interruption cost
```

Send only when the first term is high and the second is low.

- The price is **verified at the source**, not cached.
- It is at or below the user’s threshold, or a new low versus the 90-day baseline with enough observations.
- The alert includes route, dates, total price, baseline, delta, verification time, track record, and a one-tap link.
- One-tap “not interested / snooze this route”.

Cadence:

- Default: **email digest**, not real-time push.
- Push only after the user sets a watch and chooses it.
- Cap at **one alert per route per week** and **three per user per week**.
- Local time zone; quiet hours **21:00–08:00**; no weekend pushes unless chosen.
- Separate transactional alerts from marketing; marketing is opt-in.
- No “prices are rising”, “you haven’t visited”, “your streak is at risk” or “last chance” pushes.
- If an alert is wrong or stale, correct it visibly; do not bury it.

**Measure:** alert-to-booking rate, alert precision (bookings at or below threshold ÷ alerts sent), disable/mute/unsubscribe per 1,000 watchers, “would you want this alert again?” score, complaint rate.

---

## 8. Guardrails

1. **No fake scarcity.** Countdowns only when a real fare rule, hold or seat availability is verified and timestamped.
2. **No pay-to-win.** Pro sells utility (more watches, longer history, storefront comparison, export), never levels, badges, leaderboard rank or streak protection.
3. **No spend-based status.** Never tier by bookings, spend, trips, cabins, countries or miles.
4. **No dark-pattern countdowns.** No pre-checked add-ons, hidden fees, confirmshaming, or cancellation maze.
5. **No exploiting loss aversion.** No “you’ll lose your streak/savings” messaging, no paid repair, no loss-framed push.
6. **No fake social proof.** No “12 people are watching” unless true and privacy-safe; no paid trending; no fabricated reviews.
7. **No variable-reward gambling.** No mystery fares, loot-box discounts or random rewards.
8. **Privacy by default.** Travel dates, home airport, destinations and companions are private. Social is opt-in, aggregate, friend-only. No public profiles or maps.
9. **Honest uncertainty.** Range, sample size, track record, “not enough data”. Never “the price will be X”.
10. **No commission bias.** Disclose affiliate links; do not let payout influence ranking; audit savings claims.
11. **Do not gamify more flying.** No country or airport completion, no “next destination unlocked”, no rewards for trip count. A private Passport is memory, not a score.
12. **Engagement is not success.** If a mechanic raises sessions but not decision quality, remove it.

Review question: *could we publish the rules, the data and the incentive behind this mechanic without embarrassment?* If not, do not ship it.

---

## 9. Metrics

| Metric | Definition | Role |
|---|---|---|
| **Verified savings rate** | `(baseline_p50_at_decision − verified_paid_total) / baseline_p50_at_decision`, like-for-like | Primary outcome |
| **Self-reported savings** | Same calculation from user-reported price, labelled separately | Secondary, never mixed |
| **Wait-win rate** | Share of “wait” calls where the comparable price was lower at 14/30 days, net of sell-out risk | Model quality |
| **Regret rate** | Share of trips where a comparable itinerary was ≥10% cheaper within 14 days, or the user says they regret it | Guardrail and outcome |
| **Calibration** | Brier score / reliability diagram for buy/wait probabilities | Model honesty |
| **Alert precision** | Bookings at or below threshold ÷ alerts sent; plus coverage | Alert quality |
| **Decision completion** | Share of trip-intent searches ending in a logged book/wait/watch decision | Process quality |
| **Understanding** | % who can correctly explain why the app recommended buy/wait/watch | Process quality |
| **Notification fatigue** | Disables, mutes, unsubscribes per 1,000 watchers per month; alerts per watcher per week; quiet-hour breaches | Hard guardrail |
| **Pressure / anxiety** | % agreeing “the app made me feel pressured” or “I regret checking” | Hard guardrail |
| **Retention** | D7/D30, WAU/MAU, return after trip | Secondary, never primary |
| **Shares** | Receipt shares, qualified visits from shares | Secondary |

Do not use alerts sent, sessions, DAU or streak length as the primary score. They are easy to inflate and weakly related to whether the user booked well.

---

## 10. What not to build

| Do not build | Why |
|---|---|
| Daily check-in streak | Travel is episodic; daily cadence manufactures anxiety. |
| Spend-based tiers/status | Delta’s 2023 MQD-only move triggered backlash; it conflicts with a savings mission. |
| Public savings leaderboard | Savings depend on route price and trip frequency; gameable and unfair. |
| Fake countdowns / scarcity | FTC dark-pattern guidance; Hopper’s Price Freeze case. |
| Confetti / lottery on booking | Robinhood’s confetti and lottery-style free stock were cited in an enforcement case. |
| Paid streak freeze/repair | Monetises loss aversion; repair should be free if it exists. |
| Unverified savings claims | Label verified versus self-reported. |
| “Collect all countries/airports” | Rewards more flying; a Passport should be memory, not completion. |
| FOMO push notifications | Fatigue, unsubscribes, uninstalls. |
| Referral cash for bookings | Misaligns incentives toward volume. |
| “Travel personality” share bait | Trivial, privacy-sensitive, unrelated to a better fare. |
| Uncalibrated “AI price prediction” | Overconfidence erodes trust; use distributions and sample size. |

---

## 11. Copy examples

**Fare Receipt**

> **€412 total, taxes included**
> Typical for these dates: €360–€450 (middle 80% of fares paid, N=84)
> You’re inside the typical range.
> **No clear signal.** Booking now and waiting are both reasonable.
> [Watch this route] [See fare rules]

**Strong buy**

> **€298 total**
> €54 below the typical range for these dates.
> Fares this low usually disappear within four days: 7 of 10 similar fares last year did.
> **Buy now** if these dates are fixed.
> [Select fare] [Why we think this]

**Wait**

> **€438 total**
> €58 above the typical range.
> Fares for these dates usually fall three to five weeks before departure: 6 of 10 similar trips did.
> **Wait**, but set a floor. We’ll alert you if it drops below €390.
> [Watch this route]

**Not enough data**

> **Not enough data to call this route.**
> We have 12 days of fares and no reliable pattern for your dates. Current total: €412. Similar routes typically run €360–€450.
> We’ll keep watching and tell you when we have a usable signal.
> [Watch this route]

**Bad**

> “Our AI predicts prices will drop 23%. Wait!”
> “95% accuracy. Book with confidence.”
> “Only 3 seats left at this price!”
> “Last chance — fare expires in 59:59.”

---

## 12. How to measure better decisions

Define a good decision in three parts:

1. **Information:** the user saw total cost, baseline, sample size and track record.
2. **Process:** the user made and logged a buy/wait/watch call before booking.
3. **Outcome:** the verified price beat the decision-time baseline, and the user does not regret the trip.

Report the three separately. A single composite score can hide harm.

**Decision ledger fields:** `decision_id`, route, dates, cabin, bags, `decision_at`, recommendation, model_confidence (internal), `baseline_p50_at_decision`, `quoted_total_at_decision`, `verified_paid_total`, `booking_at`, `price_14d_after`, `price_30d_after`, `lowest_comparable_after`, `regret_flag`, `understood` (1–5), `pressure` (1–5), `source` (verified/self-reported).

**Counterfactuals:** same route, dates, cabin, bags and fare rules; baseline at decision time, not “highest seen”; label “lowest after booking” as a proxy; exclude mistake fares and stale caches; report median and interquartile range.

**Holdouts:** user-level randomisation, stratified by urgency, route volatility and experience; pre-register verified savings and regret as primary; pressure, notification fatigue, unsubscribes and privacy complaints as guardrails; wait for trip completion; intention-to-treat.

**Test order:**

1. Receipt + Price Position vs plain results → understanding and savings; pressure guardrail.
2. Decision Journal + No-Regret Review vs none → calibration and regret; anxiety guardrail.
3. Savings Circle vs personal bests → savings and retention; pressure/privacy guardrail.
4. Trip Season vs no streak → return after trip; guilt/compulsive checking guardrail.
5. Digest vs real-time alerts → alert-to-booking; disable/unsubscribe guardrail.

**Scale/kill rule:** scale if verified savings improve and regret/pressure do not worsen. Kill if engagement rises while savings are flat or worse, or if pressure/anxiety rises. Engagement without decision quality is a failed experiment.

---

## 13. Sources

The full evidence list, including Duolingo, Strava, Robinhood, Flighty, Hopper, Delta, CFPB, Silverman & Barasch, and notification research, is in the research pass behind this document. Key references:

- Silverman & Barasch, “On or Off Track: How (Broken) Streaks Affect Consumer Decisions”, *Journal of Consumer Research*, 2023.
- Weathers & Poehlman, “Defining, and understanding commitment to, activity streaks”, *Journal of the Academy of Marketing Science*, 2024.
- FTC, Hopper settlement, July 2026.
- Vinson & Elkins, Robinhood $7.5M gamification settlement, 2024.
- CFPB, credit-card rewards reports, 2024.
- Nielsen Norman Group, push-notification guidance.
- Flighty and Going product pages.
