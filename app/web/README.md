# Fare Desk web

The React app: search console, results, the MapLibre globe stage and the route detail. Every price, airport and buy/wait call comes from the Rust data plane in [`../crates`](../crates). The app bundles no fare data and no fallback.

## Run

```bash
# terminal 1 — data plane (live Kiwi.com fares, no key needed)
cd ..
cargo run -p faredesk-api        # http://127.0.0.1:8080

# terminal 2 — web
npm install
npm run dev
```

In development requests go to the dev server's own origin and Vite proxies `/api` and `/healthz` to `http://127.0.0.1:8080`, so any dev port works without a CORS allowance. `API_PROXY_TARGET` points the proxy at another data plane (for example a branch build on `:8081`).

A production build needs `VITE_API_BASE` set to the data plane's URL:

```bash
cp .env.example .env.local
```

## What is here

- Search console with an airport combobox (`/api/places`), dates, travellers and checked bags.
- Results sorted by Best, Cheapest or Fastest. Totals include the checked bags the traveller asked for.
- Round trips as two one-way tickets: an Outbound / Return switch, the pair's trip total and a link per ticket.
- Route strip, itinerary detail with minimum-connection-time warnings, and When to buy from `/api/route-intel`.
- The globe stage: a flat-shaded MapLibre globe under a poster sky, with routes, labels and selection sync with the list. Land is Natural Earth 50m in `public/data`.
- A provenance badge in the header: `Live data`, `Cached data`, `Stale data`, `Data error` or `No data`, with the data card and any degradation reason on hover.

Every itinerary shows `Verified N min ago` only when the source provided a verification time; otherwise it says `Not verified`.

## Failure behaviour

- **No live fare source** (only if `KIWI=off`): the data plane answers `source_not_configured`; the app shows that message and no fares.
- **Data plane unreachable:** an error with a retry. Nothing is invented.
- **4xx from the data plane** (bad date, unknown airport): the message is shown with a retry.
- **Return leg fails:** the outbound still shows; the notice names the problem.
- **Stale cache:** the API returns the last good answer with `freshness: "stale"`; the badge and notice say so.

There is no booking and no payment anywhere in the app.
