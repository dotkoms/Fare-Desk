# How fares are actually priced

Every real trick exploits a specific pricing mechanism. Learn the mechanism and you can invent tactics; memorize a list and you will fall for myths.

## Air: nested fare buckets

An airline files a fare for a city pair with rules — advance purchase, minimum stay, changeability — and allocates seats across booking classes, cheapest first. Each class has a letter (Y, B, M, K, and so on). When a bucket sells out, the next price applies. That is why:

- One seat can cost double the seat next to it.
- Prices jump in steps rather than smoothly.
- Sales and error fares are the airline opening a cheap bucket or filing the wrong one.
- Flexible-date search works: it finds the days when a cheap bucket still has seats.
- Splitting a party across bookings can keep one passenger in the cheaper bucket.

There is no secret booking code. The fare basis and booking class are diagnostic tools: they tell you **why** a price jumped, not how to hack it. ExpertFlyer and ITA Matrix expose them.

## Low-cost carriers: unbundling

The headline fare excludes bags, seats, priority boarding and airport check-in. The base fare can genuinely be below cost because ancillaries carry the margin. Consequences:

- LCC totals must be assembled, not compared fare-to-fare.
- A legacy carrier's "expensive" fare is sometimes cheaper once a bag is added.
- OTA extras are frequently more expensive than the airline's own: a Which? study found a £2 saving became £102 more after luggage and seats on one Gatwick–Orlando trip.

## Rail: quota pricing with a reservation floor

European high-speed rail sells a fixed quota of cheap advance fares per train, then climbs. The marginal cost of a seat is near zero, so the operator uses price to fill early and capture business travel late.

- **Split ticketing works** because each segment is priced against its own quota and the through fare is not the sum of the parts.
- **Railcards work** because they are price discrimination: the operator trades a discount for an annual commitment fee.
- **Advance windows matter** because the cheap quota is released when the timetable loads, and it sells first.
- **Border splits work** because international through-fares are often constructed above the sum of two domestic fares.

## Coaches: yield-managed but capacity-heavy

Coaches have low marginal cost and high load factors, so prices start near the floor and rise as seats sell.

- The cheapest bucket disappears first.
- An unsold seat is perishable, so its option value falls toward departure, partly offsetting the climb in the final week.
- Peer-reviewed evidence on FlixBus finds fares rise with occupancy and little evidence of intertemporal price discrimination.
- Promotional releases and passes are marketing spend, not pricing errors.
- The structural win is that a coach often beats rail on the same corridor by a factor of three — and an overnight coach removes a hotel night.

## All three: segmentation by flexibility

The same seat is sold at many prices based on who is willing to commit, wait, carry less, or travel at a worse time. Almost every legitimate tactic is a way of qualifying for a cheaper segment. Almost every myth is a story about beating the system that ignores which segment you are actually in.

| Mechanism | What it predicts | Tactic that follows |
|---|---|---|
| Fare buckets | Prices jump when a class sells out | Search flexible dates; split the party; book the cheap bucket when it opens |
| LCC unbundling | Headline fares mislead | Compare total cost with bags and seats |
| Rail quota | Cheap seats sell first per train | Book at the opening of the advance window; check splits |
| Coach yield | Prices rise with occupancy | Book early; use night legs to replace hotels |
| Segmentation | Cheap fares require restrictions | Accept rigidity, worse times, or less baggage deliberately |
| Perishable inventory | Last-week discounts sometimes appear | Watch under-booked off-peak departures, not peak ones |
| Market-based fares | Through-fares can exceed segment sums | Split tickets, border splits, open-jaws |
| Surcharges | Award value dies in fees | Use programs that do not pass on YQ |

## What this means for a scanner

- Price the **same physical service** across sources and keep the lowest total.
- Store the **daily minimum** for the route, not every quote.
- Re-price before alerting; a bucket can close between scan and notification.
- Do not model fares as a smooth function of days-out. They are step functions with seasonal demand layered on top.
