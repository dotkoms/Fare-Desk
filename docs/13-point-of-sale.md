# Point-of-sale pricing

How the same flight gets different prices on different country storefronts, which of those differences a traveller can legitimately use, and how the app should check them without pretending a VPN is a strategy.

---

## 1. What is actually happening

Airlines file fares **per market**, not one global price. For the same physical flight, the local storefront can differ in:

- **Fare filing** — the fare basis, booking class and rules loaded for that market.
- **Inventory** — which buckets are open to that point of sale.
- **Taxes and airport fees** — local departure taxes, VAT treatment, surcharges.
- **Currency** — the airline's own conversion rate, which can be stale.
- **Promotions** — local sales, app-only fares, resident or member fares.
- **Distribution cost** — what the airline pays to sell through that market.
- **Payment requirements** — some fares require a card or account issued in that country.

This is **point-of-sale (POS) pricing**. It is different from personalised pricing: there is no credible evidence that airlines change the fare because of your cookies, incognito mode or device. The segmentation is by market and fare conditions, not by your browser history.

The academic evidence is sobering. In a study of more than two million European low-cost fares, about **19% showed discriminatory pricing** across markets, but only **3.8% were genuine arbitrage opportunities** once fare conditions were accounted for. Deviations persisted for about two weeks and were most common in the last 14 days before departure. Ryanair's fares were non-uniform across markets roughly 40% of the time.

So: real, but usually modest, route-specific and inconsistent. It is a check, not a strategy.

---

## 2. The four levers

| Lever | What changes | Legitimate use |
|---|---|---|
| **Storefront / market** | Fare filing, inventory, taxes, promos | Search the airline's departure-country, destination-country and competitive-market sites for the same itinerary |
| **Currency** | Airline FX rate, card FX fee, DCC | Pay in the fare's currency with a no-FX card; never accept dynamic currency conversion |
| **Billing country** | Card issuing country, address verification, local-only fares | Use only a card or account you genuinely hold in that country |
| **Payment method** | Local cards, wallets, direct debit | Use a method you genuinely have; surface when one is required |

**A VPN alone changes none of these reliably.** It changes your IP. The airline still prices by market filing and validates payment by card country and billing address. That is why VPN tests usually show single-digit savings or nothing.

**A false billing address or borrowed local card is not a price hack.** It is misrepresentation. The airline can cancel the ticket, and payment fraud filters will catch the mismatch. The app must not facilitate it.

---

## 3. What the app should do

### 3.1 Multi-storefront screening

For a shortlisted itinerary, run the same search across a small set of markets:

- departure country
- destination country
- United States
- United Kingdom
- one competitive low-fare market relevant to the route

SerpAPI's Google Flights endpoint accepts `gl` (country), `hl` (language) and `currency`, so the screen can be automated without scraping. Three to five searches per shortlisted itinerary is affordable; never run it on every result.

For the airline's own fare, deep-link to its local storefront and re-price there. Google Flights may show an OTA fare that is not the airline's POS fare.

### 3.2 Eligibility profile

Store only what is needed to decide whether a fare is usable:

```ts
type EligibilityProfile = {
  residency: string;              // ISO country
  nationality: string;            // ISO country
  cardCountries: string[];        // countries where the traveller holds a card
  paymentMethods: Array<
    'card' | 'ideal' | 'bancontact' | 'giropay' | 'pix' | 'upi' | 'alipay' | 'wechat'
  >;
};
```

No card numbers, no addresses, no documents. The profile is a filter, not an identity.

### 3.3 Net total

For each storefront:

```
net = fare
    + bags
    + seats
    + booking fee
    + card FX fee
    + forced currency conversion
    + ground transfer, if it differs
```

Convert at the card's rate plus its FX fee, not the airline's marketing rate. Show the fare currency and the converted total. A €20 gap can vanish with a 3% FX fee.

### 3.4 Status, not certainty

Every POS row gets one of three labels:

- **Bookable** — fare rules and payment requirements match the profile.
- **Listed, unconfirmed** — the price is visible, but a local card or billing address may be required. The app cannot verify the final price without entering payment.
- **Ineligible** — residency-restricted, local-payment-only, or otherwise not available to this traveller.

Never show a foreign price as bookable without that distinction.

### 3.5 Persistence threshold

Recommend a foreign storefront only when:

- the net gap is above **€20 or 5%**, whichever is higher;
- the gap has persisted for several days;
- the fare rules are equivalent (bags, changes, refundability);
- and the traveller is eligible.

Otherwise it is friction for nothing. The app should be willing to say “your home storefront is the best total today.”

---

## 4. Worked example

Illustrative numbers, not a quote:

| Storefront | Currency | Fare | Bags | FX fee | Net | Status |
|---|---|---:|---:|---:|---:|---|
| Airline DE | EUR | 89 | 30 | 0 | 119 | Bookable |
| Airline US | USD | 104 | 35 | 0 | 139 | Bookable |
| Airline PT | EUR | 92 | 25 | 0 | 117 | Listed; local card may be required |
| OTA | EUR | 84 | 45 | 0 | 129 | Listed; OTA-only, refund path weaker |

The cheapest fare is not the cheapest trip. The cheapest **bookable net total** is the recommendation.

---

## 5. Where the gaps are largest

- **Premium cabins and long-haul.** Bigger absolute differences; worth checking.
- **Close-in bookings.** Dispersion rises toward departure, and POS differences are more common in the last two weeks.
- **Low-cost carriers.** Market-specific fares and promotions create larger relative gaps.
- **Departure-country POS.** Often the cheapest because the local market fares and taxes are filed there.
- **Local promotions.** Some fares exist only on the local-language site or app.
- **Currency.** The airline's conversion can be worse than the card's; paying in the local currency with a no-FX card is the clean win.

Not worth it: short-haul economy with a small gap, a card that charges 3% FX, or a fare whose after-sales support becomes harder than the saving is worth.

---

## 6. Product copy

The app should say:

> “We checked the storefronts you can actually use. Your home storefront is €89 all-in. The German storefront is €119 with the bag. No bookable saving today.”

Not:

> “Book from Argentina and save 70%.”

The first is a trust feature. The second is the reason people stop believing travel apps.

---

## 7. Guardrails

- **No VPN or proxy to fake residency.** Compare storefronts, not identities.
- **No fabricated billing addresses.**
- **No borrowed cards or accounts.**
- **No residency-restricted fare without eligibility.**
- **No hidden-city constructions** in this feature; they are a separate, higher-risk topic.
- **No promise of the final price** until the traveller reaches the payment step.
- **No republishing of fare content**; link to the storefront.

Legal note: buying a ticket from another country's storefront is generally not unlawful, but fare rules may require residency or a local payment instrument. Misrepresenting either can void the ticket. This is risk framing, not legal advice.

---

## 8. Metrics

Track:

- **Gap frequency:** share of routes where the best eligible storefront beats home by more than the threshold.
- **Median net gap**, by cabin and region.
- **Persistence:** share of gaps still present after 7 days.
- **Bookable rate:** share of “listed” prices that complete at payment.
- **Conversion:** share of POS checks that lead to a booking.
- **Complaints:** price-changed or ticket-cancelled reports.

If the bookable rate is low, the feature is a tease. If the median net gap is below the threshold, it is not worth the UI.

---

## 9. Rollout

1. **Manual first.** For one week, check three storefronts by hand on ten routes. Record the net gaps and the payment failures. This tells you whether the feature is real.
2. **Screening.** Add `gl`/`hl`/`currency` queries for the shortlist, cached, no eligibility UI yet.
3. **Eligibility.** Add the profile and the three status labels.
4. **Recommendation.** Show the table only when the gap clears the threshold.
5. **Monitoring.** Keep the gap history per route and alert when a storefront drops below the home total.

Do not ship this before the core search and route history work. It is a second-order lever; the first-order levers are dates, airports, total cost and timing.
