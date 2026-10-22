# Fare Desk web

The list-first shell from `docs/22-build-planner.md`, Step 1.

## Run

```bash
npm install
npm run dev
```

Open the printed local URL.

## What is here

- Search console with origin, destination, dates, travellers and checked bags.
- Results sorted by Best, Cheapest or Fastest.
- Route strip and selected itinerary detail.
- Route-intelligence preview with fixture data.
- Light/dark theme toggle; `?theme=light` and `?tab=intel` are available for screenshots.
- Fixture routes: BER–LIS, BER–LHR, BER–BCN, BER–JFK.

There is no live data and no booking. Every price and recommendation is demo fixture data.

## Next

Step 2 adds the react-globe.gl scene and list/globe sync. See `docs/21-globe-implementation.md`.
