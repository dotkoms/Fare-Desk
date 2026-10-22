# Data licence and boundary

The **source code** of Fare Desk is licensed under GPL-3.0. That licence does not automatically cover the project’s data or the hosted service.

## Proprietary data

The following are proprietary to the project owner and are **not** licensed under GPL-3.0:

- Fare history and daily price observations collected by the service.
- Point-of-sale quotes, eligibility results and provider comparisons.
- The True Fare database of ancillary fees.
- Airport-access, transfer-time and minimum-connection data curated by the project.
- The operational provider registry (affiliate terms, payment rails, eligibility rules), API keys, rate-limit budgets and operational configuration. The public deep-link list in `app/data/providers.json` is GPL-3.0 and is not part of this boundary.
- Hosted-service data, logs, accounts, alerts and usage records.
- Any dataset explicitly marked “proprietary” in its data card.

No rights are granted to copy, scrape, redistribute or use these commercially without written permission.

## Public exports

Some datasets may be published as open data with their own licence and data card. When that happens, the data card controls: it names the licence, coverage, freshness, methodology and attribution. A public export does not make the underlying service data open.

## Third-party data

Third-party datasets keep their own licences:

- **OurAirports** — public domain.
- **mwgg/Airports** — MIT; IANA timezones in `app/data/airports.json`.
- **Natural Earth** — public domain.
- **OpenFlights routes** — ODbL 1.0; attribution and share-alike apply. Demo use only.
- **Open-Meteo** — CC BY 4.0.
- **US BTS** — public domain.
- **GTFS feeds** — per-feed licences; see each feed’s attribution.
- **Airline and OTA fare content** — governed by the provider’s terms; the project links to the source and does not republish fare content.

## What this means for forks

A fork may reuse the GPL-3.0 code. It does not receive the proprietary fare history, fee database, access data, provider registry, API keys or hosted service. It must bring its own data sources and comply with their licences.

## Contributing data

Only contribute data you have the right to submit, with a source URL, licence and checked date. Do not contribute scraped fare content, personal data or data whose licence forbids redistribution. See [CONTRIBUTING.md](CONTRIBUTING.md).
