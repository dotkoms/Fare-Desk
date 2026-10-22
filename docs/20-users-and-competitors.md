# Users and competitors

Research pass: 22 October 2026. US and UK primary, with global patterns noted. Sources are cited; claims marked **inference** are analyst judgment, not measured fact.

---

## 1. The beachhead

**Flexible leisure travellers who already use price alerts.** Roughly 25–44, US or UK, two to four leisure trips a year, at least one long-haul every 12–18 months, ±3 days of date flexibility, booking one to four months out. They already have a Google Flights alert running.

Why this segment:

1. **Advice only works when the date can move.** Google, Kayak, Hopper and Going all base their timing guidance on flexibility. Hopper quantifies up to 14% from flexible dates; Going says shifting one to three days can save hundreds internationally.
2. **The behaviour already exists, badly.** About 42% of travellers use or would use AI for price monitoring. People already watch fares manually for weeks.
3. **The trust gap is at fulfilment, not search.** Kayak scores about 4.1/5 on Trustpilot; Hopper about 1.3/5. Only 8% are comfortable booking through an AI platform, and 66% would not let an AI book for them. A no-booking decision layer sits where trust still exists.
4. **The incumbents left a specific gap.** Google gives generic windows; Going gives deals that often do not fit the dates; Hopper's forecasts are distrusted; Skyscanner removed the whole-month view users relied on. Nobody combines route-level history, a verdict with a track record, and an all-in price.
5. **It can monetise without a subscription.** Affiliate conversion on a large, price-sensitive audience is more plausible than a paid tier at $0–30/year. The paid tier can come later from the points subset.

**Second market: VFR/diaspora travellers on recurring routes.** Highest pain and thinnest competition, but harder to reach, lower willingness to pay, and payment friction is real. Enter once coverage is dense on routes such as Lagos–London, Karachi–Manchester or Manila–Dubai.

---

## 2. Personas

### Maya, 32 — the deal-curious planner (primary)

- Brooklyn or Manchester; ~$95k household; no children; three leisure trips a year.
- **Job:** “For a 10-day window next spring, find the best-value route to somewhere warm and tell me whether to book now.”
- **Best flight means:** price, total travel time, stops, bag inclusion, times, change flexibility — roughly in that order.
- **Tools:** Google Flights price graph and alerts, Skyscanner whole-month and Everywhere, Kayak, Going free tier, TikTok for inspiration, the US 24-hour rule to hold a fare.
- **Pains:** contradictory advice; alerts on every twitch; filters reset on date change; basic economy shown as economy; redirect to an unknown OTA; price moves at checkout.
- **Switch trigger:** one screen with this route's history, a clear verdict with evidence and confidence, the all-in price, and a handoff to a seller they recognise — no signup, no fake urgency.

### Dev, 38 — the points optimizer (credibility channel)

- San Francisco or Chicago; ~$180k; 6–8 trips a year; points and status; spreadsheet-comfortable.
- **Job:** “Is this fare exceptional for this route, and should I burn points or cash?”
- **Tools:** Going paid, seats.aero, point.me, AwardFares, ExpertFlyer, FlyerTalk, transfer bonuses, positioning flights.
- **Pains:** alerts not date- or origin-specific; stale award space; too many emails; no way to verify a deal is exceptional.
- **Willingness to pay:** highest of any segment; paid fare-intelligence products already exist.
- **What wins:** route percentile with methodology, a published track record, an API or open data, and no spam. They will audit the model.

### Amara, 34 — the diaspora traveller (second market)

- Houston or London; family origins in Lagos, Karachi or Manila; ~$70k; one to three trips home a year, often three to five tickets.
- **Job:** “Get the family home for Christmas or a wedding without overpaying, and without paying extra for bags.”
- **Best flight means:** price, baggage allowance (often 2×23kg), one stop versus two, carrier reputation, date certainty.
- **Tools:** family WhatsApp, ethnic travel agents, Google Flights, Skyscanner, airline direct.
- **Pains:** opaque agent pricing, event-period spikes, bag fees, no route history, currency and payment friction, no support when things go wrong.
- **What wins:** proven savings on the exact route and event window, a multi-passenger total including bags, local payment, and a human to call.

### Leo, 21 — the student backpacker (acquisition channel)

- Berlin or Manchester; under $20k; two to four trips a year; one-ways and multi-city; highly flexible.
- **Job:** “Get me anywhere cheap for reading week, carry-on only, without a 12-hour self-transfer.”
- **Tools:** Skyscanner Everywhere, Kiwi, Ryanair/Wizz apps, StudentUniverse, split tickets, overnight layovers.
- **Pains:** basic fares with no bag, self-transfer risk, card fees, subscription traps, no support.
- **Willingness to pay:** effectively zero. Affiliate only.

### Priya, 36 — the freelancer (not now)

- Austin or Toronto; ~$110k; 6–10 trips a year; books and expenses her own travel.
- **Job:** “Get me there reliably, changeable if the client moves, at a price I can defend.”
- **Best flight means:** time and reliability first, changeability second, price third.
- **Pains:** close-in prices, change fees, disruption rebooking, expense tracking.
- **Why not now:** she needs reliability and flexible fares, not buy/wait advice.

### The Hendersons, 41 and 39 — the family (later)

- Suburban Dallas or London; ~$140k; one or two big trips a year; four tickets; school-holiday constrained.
- **Job:** “Get all four of us there together, seated together, bags included, with no surprises.”
- **Pains:** seat fees, basic economy, four change fees, OTA support failures.
- **Why not now:** “wait” is often the wrong advice for a fixed school holiday. Their real need is total-family cost, which is a feature rather than the wedge.

### Dev, Amara, Leo, Priya and the Hendersons are not the first user.

Build for Maya. Use Dev as the credibility and paid-beta channel. Enter Amara's routes once coverage is dense. Treat Leo as free acquisition and a data source. Serve Priya and the Hendersons with total-cost features later, not with timing advice.

---

## 3. Priority matrix

Scores 1–5, weighted for a no-booking, affiliate-funded decision tool. These are analyst judgments, not measured values.

| Segment | Advice actionability (25%) | Size (20%) | Pain (15%) | WTP (20%) | Whitespace (15%) | Acquisition (10%) | Weighted |
|---|---:|---:|---:|---:|---:|---:|---:|
| Flexible leisure | 5 | 5 | 4 | 2 | 3 | 4 | **4.10** |
| Deal hunters / points | 4 | 3 | 4 | 5 | 2 | 4 | **3.90** |
| VFR / diaspora | 4 | 4 | 5 | 2 | 5 | 1 | **3.80** |
| Families | 2 | 4 | 5 | 4 | 4 | 3 | **3.75** |
| Students | 5 | 3 | 4 | 1 | 3 | 5 | **3.60** |
| Freelancers / SMB | 2 | 3 | 3 | 4 | 4 | 3 | **3.25** |

If willingness to pay is weighted at 30% and size at 10%, deal hunters move first. If acquisition is weighted at 20%, students rise above VFR. The top and bottom are stable; the middle is not.

---

## 4. What they want

| # | Need | Evidence | v1 response |
|---|---|---|---|
| 1 | **A verdict I can act on, with the evidence** | Google 89 days vs Kayak 7–14 vs Hopper 3–6 months vs Going 2–8 months; users say forecasts are inaccurate or missing | Route-specific percentile, sample size, confidence, visible track record, and an explanation of why sources disagree |
| 2 | **The real total, before I commit** | 61% cite hidden or unexpected fees; basic economy $100–360 more; bag and seat fees; self-transfer risk | All-in price for the passenger count, bags, seats, basic-economy restrictions and transfer warnings |
| 3 | **Tell me when it actually matters** | “Too many silly notifications”; alerts lose track; deals irrelevant to dates | Event-based alerts (signal change, target fare, expiry), persistent filters, route-level rather than every-tick |
| 4 | **Don’t hand me to a stranger** | “Refered to Trip.com … disaster”; unknown agencies; Hopper FTC settlement; eDreams Prime complaints | Name the seller, show fare rules and disruption rights, disclose affiliate status, prefer airline-direct, never auto-redirect |
| 5 | **Let me explore without losing my constraints** | Skyscanner month view removed; Kayak basic-fare filter removed; Going deals do not fit dates; filters reset | Month/±3-day grid, nearby airports, “anywhere,” filters that survive date changes, saved locally |

A sixth need — multi-passenger coordination and seats together — is folded into #2 because it is a pricing and comparison problem before it is a collaboration problem.

---

## 5. Language

Users say:

- **“Good price”** — cheap, cheapest, best price, reasonable, affordable, good deal, worth it.
- **“Deal”** — deal, bargain, steal, mistake fare, error fare, sale, price drop, low fare.
- **“Should I wait?”** — “should I book now or wait”, “will it go down”, “wait or buy”, “book now”, “hold off”, “pull the trigger”.

They do not say “fare forecasting”, “optimal purchase window”, or “predictive model”. Avoid those in the UI.

The word “deal” is distrusted because it is used for both a genuine anomaly and a marketing email. A review of a deal newsletter: “Every ‘Deal’ doesn’t work for the dates advertised.” Another: “A cheap ticket isn’t a good deal when the customer service costs you the entire trip.”

Use their words:

- “Book now or wait?”
- “Good price for this route”
- “Here’s the evidence”
- “Price changed since you searched”
- “We don’t have enough data for this route yet”

Avoid:

- “AI prediction”
- “Algorithm says”
- “Optimal”
- “Guaranteed”
- “Last chance” unless the fare genuinely expires and you can prove it.

---

## 6. Trust objections

| Objection | Product answer |
|---|---|
| “Another lead-gen site that dumps me on a sketchy OTA.” | Show the seller, its rating, fare rules and disruption rights before the click. Disclose affiliate status. Prefer airline-direct. Never auto-redirect. |
| “The price won’t be the price.” | All-in total, timestamped, with a “price changed since you searched” state. No pre-selected add-ons. |
| “Your data is stale or incomplete.” | Show last-checked time, coverage gaps and sample size. Say “not enough data” rather than guessing. |
| “Why trust your prediction over Google’s free one?” | Publish the method and the hit rate. Show the distribution, not a single number. Let users see past calls on the route. |
| “Who do I call when it goes wrong?” | State plainly: “We don’t book. Here is who is responsible, and here is their support route.” Link to EU261/UK CAA/DOT rights. |
| “What are you doing with my data?” | Local-first by default; no account for search; open-source client; plain-language data page. |
| “Will you exist next year?” | Open repository, public roadmap, clear funding model, exportable data. |
| “Is this even legal?” | Licensed feeds where possible; taxes and fees inclusive; a data-source page. |

The strongest trust move is structural: **do not take the booking.** Every major trust failure in the evidence is a fulfilment failure, not a search failure.

---

## 7. What v1 should not serve

1. Booking and payments.
2. Award/points search.
3. Multi-city, open-jaw and complex itineraries.
4. Groups larger than six, or group payment.
5. Packages, hotels and cars.
6. Real-time disruption support and rebooking.
7. Corporate policy, approvals and expense workflows.
8. Price guarantees, refunds or compensation promises.
9. Mistake fares as a headline feature.
10. Last-minute travel under seven days as a primary use case.
11. Thin routes with no history.
12. Localisation beyond US/UK/EU coverage.

---

## 8. Competitors

### 8.1 The field

| Product | Position | What it does better than a new entrant | Where it is weak |
|---|---|---|---|
| **Google Flights** | Free search and price-intelligence layer inside Google, now with AI Mode | Distribution, 300+ partners, speed, date grid, price graph, Explore map, free tracking in 180+ countries | Generic advice; no route-level track record; no door-to-door total; no booking or after-sales |
| **Kayak** | Broad metasearch with decision tools | Deepest filters, 7-day Price Forecast with confidence, Explore, Hacker Fares | Ad/redirect-heavy; stale prices at the OTA; app instability; referral business cannot own the post-click |
| **Skyscanner** | Global metasearch and budget-first discovery | 110M monthly visitors, 1,200 partners, Everywhere, LCC coverage | App performance; price mismatch after redirect; no route-level buy/wait model |
| **Hopper** | Mobile-first OTA plus prediction and fintech | Best mobile conversion, prediction brand, 120M users | Upsell/fee-heavy; opaque methodology; refund/support failures; $35M FTC settlement over pre-selected fees |
| **Expedia** | Full-service OTA with packages, rewards and AI | Bundles, One Key, broad inventory, trip management | Bloated; support/refunds; standalone flights are second-class; stopped OneKeyCash on standalone flights in Oct 2026 |
| **Momondo** | Kayak-owned design-led metasearch | Brand and simple search; same inventory as Kayak | Thin differentiation; redirect quality; privacy claim undercut by ad model |
| **Kiwi.com** | Virtual-interlining OTA | Routes and prices others do not show; LCC coverage; protection products | Support and refunds; cancellations; baggage fees; self-transfer complexity |
| **Skiplagged** | Hidden-city and arbitrary round-trip engine | Unique hidden-city fares; simple search | Service fees; checkout price jumps; baggage/visa risk; airline legal exposure |
| **Going** | Deal-alert membership, ex-Scott’s Cheap Flights | Curation, mistake fares, 2M+ members, email relationship | US-only departures; app bugs; paywall-heavy; deals often do not fit dates |
| **Jack’s Flight Club** | Human-curated deals, especially premium cabins | Editorial voice, 4M members, UK/EU coverage | Members-only app; frequently broken; limited airports; deals route back to Google Flights |
| **Flighty** | Pro-grade live flight tracker and design benchmark | Focus, speed, privacy, map/timeline, Apple Design Award | No fares; iOS-only; subscription prompts; data gaps |
| **Flightradar24** | Global flight-tracking map | 3D aircraft, airport boards, playback, coverage | Ad-heavy; key features paywalled; cluttered; battery/session issues |

### 8.2 Feature matrix

Y = present, P = partial, N = absent.

| Product | Price history | Buy/wait advice | Total cost | Explore | Alerts | Booking | Map / 3D | Local-first | Open source |
|---|---|---|---|---|---|---|---|---|---|
| Google Flights | Y | Y (generic) | P | Y | Y | N | Y / N | N | N |
| Kayak | P | Y (7-day) | P | Y | Y | N | Y / N | N | N |
| Skyscanner | P | N | P | Y | Y | N | Y / N | N | N |
| Hopper | P | Y (opaque) | P | Y | Y | Y | N / N | N | N |
| Expedia | P | N | P | Y | Y | Y | Y / N | N | N |
| Momondo | P | N | P | Y | Y | N | Y / N | P | N |
| Kiwi.com | P | N | P | Y | Y | Y | Y / N | N | N |
| Skiplagged | N | N | P | P | Y | Y | N / N | N | N |
| Going | N | N | P | Y | Y | N | P / N | N | N |
| Jack’s Flight Club | N | N | P | N | Y | N | N / N | N | N |
| Flighty | N | N | N | N | Y (status) | N | Y / Y | P | N |
| Flightradar24 | N | N | N | N | Y (status) | N | Y / Y | N | N |

Only Google, Kayak and Hopper offer genuine buy/wait advice. Google’s is generic; Kayak’s is a 7-day forecast on limited routes; Hopper’s is opaque. **None publishes a route-level, auditable track record.** That is the opening.

### 8.3 What they can do that we cannot, quickly

- **Google:** distribution, 300+ partners, free at the point of use, AI Mode, brand trust. Do not fight on breadth or speed.
- **Kayak:** inventory, deepest filters, an existing forecast. Do not fight on filter count.
- **Skyscanner:** global LCC coverage and the Everywhere brand. Do not promise global parity.
- **Hopper:** mobile conversion and fintech products. Do not sell financial protection.
- **Expedia:** packages and merchant infrastructure. Do not become an OTA.
- **Kiwi:** virtual-interlining technology. Expose the risk; do not become the merchant.
- **Flighty / Flightradar24:** live tracking data and 3D craft. Use a globe for discovery, not aircraft tracking.

### 8.4 The gap stack, ranked by defensibility

1. **Auditable buy/wait** — route-level evidence, sample size, lead-time curve, published track record, and “no advice” when data is thin.
2. **Total door-to-door cost** — fare, bags, seats, transfer, self-connect risk, time.
3. **Independence** — no booking, no affiliate-ranked results, no OTA redirect maze.
4. **Privacy / local-first** — no account, no surveillance-pricing surface.
5. **Flight-only focus** — OTAs are retreating to packages; own the standalone flight job.
6. **The 3D globe** — a discovery hook and marketing asset, not the moat.

### 8.5 Table stakes, differentiators, anti-patterns

**Table stakes:** fast search with one-way/return, nearby airports and flexible dates; filters for stops, airline, time, cabin, duration and bags; taxes and mandatory fees included; price alerts; a price graph or calendar; Explore; deep links; mobile and accessible list view; a timestamp and “prices change” disclosure.

**Differentiators:**

- A route-level percentile with sample size and a lead-time curve.
- An explainable buy/wait call with a published track record, including misses.
- Total door-to-door cost.
- Self-transfer and airport-change risk flags.
- Local-first, no account, no tracking.
- Open-source methodology and data cards.
- A first-class “no clear signal / not enough data” state.
- Independent ranking with no paid placement.
- A 3D globe that makes route geography legible.

**Anti-patterns:**

- Pre-selected tips, VIP support or ancillaries.
- Drip pricing and service fees at checkout.
- Paywalling basic search or filters.
- Opaque AI predictions without data, confidence or track record.
- Fake urgency, countdowns and “only 1 seat left” without live inventory.
- Unvetted OTA redirects.
- Hidden-city or throwaway ticketing as the default.
- A mandatory 3D globe with no list view.
- Account walls or contact scraping.
- Subscription traps and hard cancellation.
- Ad-heavy maps and interstitials.
- Cross-selling hotels, cars and packages.
- “Guaranteed lowest” or “100% accurate.”

### 8.6 What makes users switch

| Segment | Switching from | What would make them switch | What must be true |
|---|---|---|---|
| Google Flights power user | Google | A route-level percentile and “why” card Google does not show; total cost; push alerts; no tracking | At least as fast on the first search, clearly better on the route they care about |
| Hopper user | Hopper | The same book-or-wait answer without upsells, Pro paywall or opaque fees; a track record; direct airline links | They have been burned by a fee, cancellation or refund |
| Kayak / Skyscanner user | Kayak, Skyscanner | Verified prices with timestamps; no scam OTA redirects; a fast app; total cost | Price parity and coverage on their routes |
| Going / JFC subscriber | Going, Jack’s | Free or cheaper on-demand search; broader airports; fresher deals; no paywall for basic search | Curation quality comparable |
| Privacy / open-source user | Google, OTAs | Local-first storage, no account, auditable method, self-host, open data | Niche but high-loyalty |
| Aviation / geography fan | Flightradar24, Flighty | A fast 3D globe with fare layers, route network and airport pages | The globe must be useful, not a novelty |

The hardest switch is Google: the switching cost is near zero, but the habit is enormous. The easiest switch is a burned Hopper or Kiwi customer, because the trust breach is concrete. The most defensible niche is the frequent international traveller with flexible dates and high ticket values, for whom a bad call costs more than a subscription.

### 8.7 The ratings signal

Recent US App Store samples (150 reviews per app, 22 Oct 2026) diverge sharply from lifetime ratings:

| Product | Lifetime | Recent 150 avg | 1–2 star share, recent | Trustpilot |
|---|---:|---:|---:|---:|
| Hopper | 4.82 | 4.60 | 9% | **1.3** |
| Kayak | 4.83 | **2.59** | 59% | 4.1 |
| Skyscanner | 4.81 | 3.16 | 45% | 4.4 |
| Expedia | 4.79 | 2.96 | 47% | 3.8 |
| Kiwi.com | 4.59 | **1.87** | 78% | 4.0 |
| Going | 4.75 | **2.30** | 65% | 4.5 |
| Jack’s Flight Club | 2.93 | **1.69** | 81% | 4.1 |

App-store ratings are solicited in-app and skew positive; Trustpilot is self-selected and can be review-bombed. Use the direction, not the precision. The repeating theme is **what happens after the click**: price changes, OTA cancellations, refund delays, support failures. That is the trust vacuum a no-booking product can occupy.

---

## 9. Implications for Fare Desk

- **Position:** the independent flight-price analyst — the decision layer between search and the airline. Not a metasearch, not a booking site, not a deal newsletter, not an AI agent.
- **Primary user:** Maya, the flexible leisure planner. Every screen should answer her three questions: *what are my options, what is the real total, and should I book now?*
- **Primary proof:** route-level history, sample size, a range, and a published track record that includes misses.
- **Primary trust move:** name the seller and do not take the payment.
- **Do not fight Google on breadth or speed.** Google will win distribution and coverage. Win on judgment, explanation and total cost.
- **Start with high-stakes trips, not casual domestic.** Long-haul, flexible dates and high ticket values are where a bad call costs more than a subscription. Casual domestic bookers default to Google.
- **Easiest switchers:** people burned by Hopper, Kiwi or an OTA refund. Hardest: Google power users, because the habit is enormous even when the switching cost is zero.
- **The globe is the hook, not the product.** Use it for Explore and marketing; make the list and the advice carry retention.
- **Affiliate revenue is a bridge, not the model.** If the business depends on redirects, the advice will drift toward the highest bidder and the trust wedge collapses. Plan a paid Pro tier, API or hosted convenience, and keep ranking independent.
- **Primary distribution:** burned OTA customers, deal-hunter communities for credibility, privacy/open-source users, and diaspora routes once coverage is dense.
- **Primary constraint:** no advice without enough data. “Collecting baseline” and “no clear signal” are legitimate product states.
- **Biggest risk:** cold-start history, not competition. Google gives tracking away free; Fare Desk has to earn the right to advise route by route.

---

## 10. Sources

**Product and help pages**

- [Google Flights — finding best fares](https://support.google.com/travel/answer/7664728)
- [Google Flights — tracking](https://support.google.com/travel/answer/6235879)
- [Kayak — price alerts and forecast](https://www.kayak.com/news/flight-price-alerts/)
- [Skyscanner on the App Store](https://apps.apple.com/us/app/skyscanner-flights-travel/id415458524)
- [Hopper — price prediction](https://hopper.com/product/price-prediction)
- [Expedia on the App Store](https://apps.apple.com/us/app/expedia-hotels-flights-cars/id427916203)
- [Momondo — about](https://www.momondo.com/about)
- [Kiwi.com — guarantee](https://www.kiwi.com/ag/help/kiwi-com-product-services-256/article/what-s-the-kiwi-com-guarantee-512/)
- [Skiplagged on the App Store](https://apps.apple.com/us/app/skiplagged-travel-hacks/id823443083)
- [Going — how it works](https://www.going.com/how-it-works)
- [Jack’s Flight Club](https://www.jacksflightclub.com/)
- [Flighty — pricing](https://flighty.com/pricing)
- [Flightradar24 on the App Store](https://apps.apple.com/us/app/flightradar24-flight-tracker/id382233851)

**Regulatory and trust evidence**

- [FTC — Hopper $35M settlement](https://www.ftc.gov/news-events/news/press-releases/2026/07/travel-app-hopper-pay-35-million-settle-ftc-allegations-it-charged-fees-without-consent-deceived)
- [AGCM — eDreams €9M fine](https://en.agcm.it/en/media/press-releases/2026/2/PS12853)
- [Trustpilot — Hopper](https://www.trustpilot.com/review/hopper.com)
- [Trustpilot — Kayak](https://www.trustpilot.com/review/www.kayak.com)
- [Trustpilot — Kiwi.com](https://www.trustpilot.com/review/www.kiwi.com)

**User evidence**

- Deloitte 2026 summer travel survey, via GCN: <https://gcn.com/only-americans-plan-paid-summer-vacation/21830/>
- Expedia Group AI Trust Gap, via City AM: <https://www.cityam.com/expedia-group-reveals-the-ai-trust-gap-travelers-embrace-ai-for-planning-but-rely-on-trusted-brands-to-book/>
- Skift, “The Neither Economy”: <https://skift.com/2026/05/01/the-neither-economy-travels-big-blind-spot/>
- UK CMA, Ryanair family seating investigation: <https://www.reuters.com/world/uk/uk-watchdog-investigates-ryanair-over-fees-to-seat-parents-with-children-2026-06-11/>

App-store and Trustpilot samples are directional, not representative. Company-reported scale figures are treated as company claims.
