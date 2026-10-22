# UI and UX

**Primary user:** Maya, the deal-curious planner. 25–44, US or UK, two to four leisure trips a year, ±3 days of flexibility, books one to four months out, already has a Google Flights alert running. See [20-users-and-competitors.md](20-users-and-competitors.md).

**The job:** “Show me the best flight for my dates, the real total, and whether I should book now or wait — without handing me to a stranger.”

**The product:** a search console and a globe. The list is the tool. The globe is the map, the context and the discovery surface. The route page is the differentiator.

---

## 1. Principles

1. **List first, globe second.** Every result is readable, selectable and comparable without touching the globe. The globe is an enhancement, never the only path.
2. **One primary action per state.** Search, select, set alert, or open the provider. Never two competing calls to action.
3
. **Total cost or no price.** Bags, seats, fees, transfers and fare rules are part of the number, not a surprise at checkout.
4. **Evidence over confidence.** Show the sample, the window, the percentile and the track record. “Buy” is a claim; the receipt is the product.
5. **Say “not enough data.”** A thin route gets a watch page, not a confident guess.
6. **No fake urgency.** No countdowns, no “3 people are looking”, no pre-selected add-ons, no “last chance” unless the fare genuinely expires.
7. **Name the seller.** The provider is part of the result: airline or OTA, country, support path, affiliate disclosure.
8. **We do not book.** The final action is “Open on [provider]”. The app never takes payment.
9. **Calm by default.** Muted surfaces, one accent, motion that explains a change rather than decorates it.
10. **Private by default.** No account for search. The eligibility profile stays on the device. Data export is a first-class action.

---

## 2. Information architecture

```
Home / Search
├─ Results
│  ├─ Itinerary detail
│  ├─ Provider options
│  └─ Route intelligence
├─ Explore
│  ├─ Month view
│  ├─ Destination detail
│  └─ Route intelligence
├─ Watchlist
│  ├─ Saved routes
│  ├─ Alerts
│  └─ Savings ledger
└─ Data & settings
   ├─ Eligibility profile
   ├─ Currency, units, theme
   └─ Data sources, licences, privacy
```

**Desktop:** a 380px console on the left, the globe filling the rest. The route-intelligence view expands the console to 460px; it does not open a second pane. A detail drawer can overlay the globe from the right for a single itinerary.

**Route scale:** below a camera altitude of 1.35 R, crossfade the globe to a flat route board. A globe shows one hemisphere; at route scale the destination can sit behind the horizon. The globe is for overview and Explore; the route board is for “what am I buying?”

**Tablet:** 340px console, globe; the detail drawer becomes a sheet.

**Mobile:** full-bleed globe, bottom sheet with three snap points: search summary, results, full filters/detail.

---

## 3. First run

1. Globe idles slowly. The console asks: **“Where from?”**
2. The place combobox searches airports and cities. No account, no onboarding carousel.
3. Choosing an origin flies the camera there and draws the top routes.
4. The console asks: **“Where to?”** with a “Not sure yet — explore” link.
5. If the user explores, the console switches to Explore; if they pick a destination, it asks for dates.

No modal, no cookie wall beyond what is legally required, no newsletter prompt before the first result.

**Return visit:** restore the last search and watchlist locally. Do not re-ask for origin. Show a single line: “3 watched routes; last checked 06:12.”

---

## 4. Screens

### 4.1 Search console

Order:

1. One-way / return.
2. From / To, with swap.
3. Dates, with ±1/±3 days.
4. Travellers, cabin, bags.
5. More filters (collapsed): nonstop, max duration, departure window, price ceiling.
6. Search.
7. Recent searches and watched routes.

**Rules:**

- Changing dates must not reset the filters.
- The bag count is visible before search, not after.
- “Nearby airports” is a toggle, not a hidden default.
- The search button is sticky at the bottom of the console.

### 4.2 Results

Header: result count, sort (Best, Cheapest, Fastest, Earliest), active filter chips.

Each card:

```
06:40 → 09:00        BER → LIS        6h 20
Air Baltic · 1 stop via MAD · 1h 05
€89 total · 1 checked bag included
Verified 4 min ago           [When to buy]
```

**Rules:**

- Price is the total for the party, with bags, in the user's currency.
- “Basic economy” is stated explicitly; never label it just “economy”.
- The cheapest card is not automatically first under “Best”; the score explains why.
- Hovering a card lifts its arc. Clicking selects it and calls `ensureVisible` only if the route is outside the safe area.
- A card never opens a new tab until the user chooses “Open on [provider]”.

### 4.3 Itinerary detail

- Segment timeline with local times, terminals, layovers and operating carrier.
- Total duration and door-to-door estimate.
- Baggage and fare rules.
- Self-transfer and OTA warnings.
- Provider options table: seller, country, currency, net total, status.
- One primary action: “Open on [provider]”.
- “When to buy” secondary action.

### 4.4 Total cost

The total is the unit of comparison. A fare that is not bookable at that total for the selected party and bags is not a price; it is an advertisement.

- **Mandatory is included.** Taxes, carrier charges and unavoidable fees are in the headline.
- **Optional is opt-in.** Bags, seats, insurance and priority are never pre-selected.
- **Bag context changes everything.** Default to “personal item only”, with a visible selector for carry-on and checked. Changing it updates every price and the sort order.
- **Fare rules at the point of choice.** Three chips: refundability, change cost, credit/cancellation. Full tariff behind a link.
- **Show the delta, not only the total.** “+€35 each way” is easier to judge than a new number.
- **State whether the fare is held.** “Prices update in real time. Your fare is not held until you pay.”
- **One persistent total.** A footer that updates as options change, with an itemised breakdown one tap away.

Good:

> **€412 total per person**
> Includes taxes and carrier charges. Personal item included.
> Add a checked bag for €35 each way.
>
> Personal item only **€412** · Carry-on **€447** · Checked bag **€482**

Bad:

> “€299*”
> “*Excludes taxes, fees, bags and seat selection. Final price at checkout.”

### 4.5 Route intelligence

**Route strip first.** For a short route: `BER 10:40 ── 1h 50m ── LIS 12:30 / nonstop`. For a one-stop: `BER 07:10 ─ 1h 20m ─ AMS 08:30 / layover 50m / AMS 09:20 ─ 2h 40m ─ LIS 11:00`. This is more legible than any globe for “what am I buying?”, and it avoids drawing a direct arc for a connecting itinerary.

The differentiator. Composition:

1. **Current total** and position in the typical range.
2. **Verdict:** BOOK NOW / WAIT / WATCH / NOT ENOUGH DATA. No raw confidence score.
3. **Evidence:** typical range, sample size, track record, last checked.
4. **Days-out curve:** what usually happens from here.
5. **Backtest:** “On this route, waiting beat buying in 6 of 10 similar cases”, including misses.
6. **Fare rules:** refundable, changeable, bags.
7. **Provider options:** only if a material gap exists.
8. **Alert:** “Tell me if the total drops below €110.”
9. **What would change our mind:** one sentence.

**Verdict states:**

| State | Meaning | UI |
|---|---|---|
| **Book now** | The fare is outside the typical range and the base rate supports booking | Specific claim, range, base rate, primary action |
| **Wait** | The expected value of waiting is positive, with risk | Upside and downside in currency, re-check date |
| **Watch** | Enough data, but no reliable edge | “Your call”, alert, no pressure |
| **Not enough data** | Too few observations or too much volatility | Say what is missing, show known facts, watch only |

**Copy rules:**

- “Book now or wait?” not “Fare forecast”.
- “There is not enough data for this route yet” not “Insufficient observations”.
- “Prices on this route usually rise €30–€60 in the next three weeks” not “Predicted price”.
- “When we said this on similar routes, it was right 7 times in 10 (n=214)” not “78% confidence”.
- “Middle 80% of fares paid: €110–€151” not a bare percentile.
- “We don’t book. Here’s who does.” near the provider action.
- “No clear signal” is a valid answer; do not manufacture a verdict.

### 4.6 Explore

- Home airport selector.
- Month scrubber.
- Globe arcs coloured by discount versus baseline; legend visible.
- Destination list with price, percentile, nonstop/stops and “last checked”.
- Filter: nonstop, under €X, weekend, month.
- Coverage line: “42 destinations validated from BER; last checked 6 hours ago.”

**Rules:**

- Do not draw an arc until the route is validated by a live search.
- If a destination has no baseline, show “watching” rather than a fake percentile.
- The month scrubber changes the data, not just the colour; show the month's median and sample count.

### 4.7 Watchlist

- Saved routes with current price, baseline, last change and alert status.
- “Saved €X” ledger: the difference between the decision-time baseline and the price paid, only if the user confirms the paid price. Label verified versus self-reported.
- Alert rules in plain language: “Alert me when the total drops below €80” or “when it is 15% below the median”.
- **Opt in at the moment of intent**, on the route card, not at first launch.
- Default to an **email digest**, not push. Push only after the user chooses it.
- Caps: one alert per route per week and three per user per week.
- Quiet hours 21:00–08:00 local time; respect the OS Focus state.
- Every alert includes the total, what changed, why it matters, the action, and “stop watching”.
- Immediate delivery only for a target hit or a genuine fare expiry. No countdowns, no “instant deal” language.

### 4.8 Data and settings

- Currency, units, theme, reduced motion.
- Eligibility profile: residency, nationality, card countries, payment methods. Explain why each field exists; allow “prefer not to say” and show fewer provider results.
- Data sources, licences, last-checked times, and a “download my data” action.
- A plain statement: “We don’t sell tickets or take payment.”

---

## 5. Interaction model

| Action | Result |
|---|---|
| Hover a result card | Its arc lifts and brightens; tooltip with airports and times. No camera move. |
| Click a result card | Select; `ensureVisible` only if the route is outside the safe area; other arcs dim to 15%; detail drawer opens. |
| Click an arc | Select the corresponding itinerary; scroll its card into view. |
| Click empty globe | Clear selection; camera returns to the result bounds. |
| Change a filter | Results update; selection persists if the itinerary is still present. |
| Change dates | Results invalidate; previous selection stays at 30% until new results arrive. |
| Press `/` | Focus the search field. |
| Arrow keys | Move through results. Enter selects. Esc clears. |

**Camera:** 350–800ms, scaled by angular distance; interruptible by any pointer or key input. Prefer `ensureVisible` over a flight: if both endpoints are already in the safe rectangle, do not move. Suppress auto-fit for 8 seconds after the user moves the camera, and show a “Fit route” chip instead. No auto-rotation. Respect reduced motion: jump with a 150ms crossfade.

**Selection:** theme-aware ink on light, white on dark; thicker, with a 2px casing and endpoint rings. Dimmed: 60% default, 35% during hover, 15% when another itinerary is selected. Never hide the rest completely; the route context matters.

---

## 6. Loading, empty and error states

| State | What the user sees |
|---|---|
| First load | A static globe poster crossfades to WebGL; console skeleton; no spinner over the globe |
| Search loading | Cached results immediately; “verifying prices” line; skeleton only for missing cards |
| No results | “No flights match all filters.” Chips show which filter to relax; one-click relax |
| Source failure | “Live prices unavailable; showing cached results from 06:12.” |
| Price changed | Card marked “price changed”; new price shown; old price struck only if honest |
| Insufficient history | “Collecting baseline — 12 of 14 days”; show what is known; no verdict |
| Route not validated | “We haven’t confirmed this route yet”; offer to check |
| Offline | Last cached search and watchlist; no fake live state |

---

## 7. Accessibility

- WCAG 2.2 AA; body text 4.5:1.
- The result list is the primary interface; the globe is optional.
- Full keyboard path: search, results, detail, provider action.
- Visible focus ring on every interactive element.
- `aria-live="polite"` for result counts, selection and price changes.
- Charts have a text summary and a data table alternative.
- Reduced motion disables auto-rotation, camera flight, arc animation and selection rings.
- Colour is never the only signal.
- Hit targets at least 48×48px on touch.
- “List only” toggle hides the globe and keeps all functionality.

---

## 8. Mobile

- Globe full-bleed behind a bottom sheet.
- Sheet snap points: 88px (summary), 48dvh (results), 92dvh (filters/detail).
- The search button is reachable with one thumb.
- Hover does not exist; tap selects, tap again opens detail.
- Pinch-zoom is reserved for the globe; do not hijack it for sheet resizing.
- The route-intelligence chart scrolls horizontally inside its own container, never the page.

---

## 9. Copy

**Use:**

- “Book now or wait?”
- “Good price for this route”
- “Here’s the evidence”
- “Price changed since you searched”
- “We don’t have enough data for this route yet”
- “We don’t book. Open on [provider] to complete the purchase.”
- “Total for 2 travellers, including 2 checked bags”

**Avoid:**

- “AI prediction”, “algorithm”, “optimal”
- “Guaranteed”, “lowest price ever” without proof
- “Last chance”, “hurry”, “only 2 seats left” unless verified and relevant
- “Deal” as a generic label
- “Book” as the app's action; use “Open on [provider]”

---

## 10. What we are not building

- A checkout, payment form or ticket issuance.
- A chat assistant that books.
- A wall of filters.
- A full-screen map with no list.
- A newsletter pop-up before the first result.
- A streak that punishes you for not opening the app.

---

## 11. Usability tests

Five tasks, moderated, ten participants from the beachhead segment:

1. Find the cheapest total for two people with one checked bag.
2. Decide whether to book now or wait, and explain why.
3. Find a cheaper nearby airport and judge whether it is worth it.
4. Set an alert for a target price.
5. Open the provider and identify who sells the ticket.

Measure: task success, time, wrong turns, trust rating, and whether the participant can explain the verdict in their own words.

**Kill criteria:** if fewer than seven of ten can explain why the app recommended buy or wait, the route page is not working.
