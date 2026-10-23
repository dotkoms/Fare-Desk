# Fare Desk web

The React app: search console, results, the globe stage and the route detail. It reads prices from the Rust data plane in [`../crates`](../crates) and falls back to bundled fixtures when that service is not running.

## Run

```bash
# terminal 1 — data plane
cd ..
cargo run -p faredesk-api        # http://127.0.0.1:8080

# terminal 2 — web
npm install
npm run dev
```

In development the app calls `http://127.0.0.1:8080` by default. Point it elsewhere, or disable the API entirely for a static deploy, with an environment variable:

```bash
cp .env.example .env.local
```

`VITE_API_BASE` unset in a production build means fixtures only; no request is attempted.

## What is here

- Search console with origin, destination, dates, travellers and checked bags.
- Results sorted by Best, Cheapest or Fastest. Totals include the checked bags the traveller asked for.
- Route strip, itinerary detail and the buy/wait preview (the preview is still demo history).
- The globe stage with land, airports, direct arcs, one-stop paths and selection sync with the list.
- A provenance badge in the header: `Live data`, `Cached data`, `Stale data` or `Demo data`, with the data card and any degradation reason on hover.
- Light/dark theme toggle; `?theme=light` and `?tab=intel` are available for screenshots.
- Fixture routes: BER–LIS, BER–LHR, BER–BCN, BER–JFK.

Every itinerary shows `Verified N min ago` only when the source provided a verification time. Fixtures show `Demo fare` instead, and the stage says `Demo fare — not verified`.

## Failure behaviour

- **Data plane unreachable:** the app serves its bundled fixtures, shows a notice above the results and keeps the demo usable offline.
- **4xx from the data plane** (bad date, unknown airport): the message is shown with a retry, rather than falling back silently.
- **Live round trip requested:** the live source prices one direction at a time, so the app asks the same service for the fixture answer and says so in the notice.
- **Stale cache:** the API returns the last good answer with `freshness: "stale"`; the badge and notice say so.

There is no booking and no payment anywhere in the app.

## Next

The proposed 2D poster map is planned in [`../../docs/23-2d-poster-map.md`](../../docs/23-2d-poster-map.md); the current globe is documented in [`../../docs/21-globe-implementation.md`](../../docs/21-globe-implementation.md).
