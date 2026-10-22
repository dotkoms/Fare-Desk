# Security policy

## Reporting a vulnerability

Do not open a public issue. Use GitHub’s private vulnerability reporting for this repository, or contact the maintainer through the repository profile.

Include:

- the affected component and version or commit,
- a reproduction or proof of concept,
- the impact,
- any suggested fix,
- whether you want credit.

## Scope

High-priority areas:

- API keys, tokens or cookies exposed in the client, logs or repository.
- Obscura sandbox escapes, unapproved network egress or credential exposure.
- SSRF or arbitrary URL fetching in the API.
- Injection, path traversal or unsafe deserialisation.
- Authentication or authorisation flaws in any hosted service.
- Data exposure affecting fare history, eligibility profiles or alerts.

## Supported versions

The project is pre-1.0. Security fixes land on `main`; there are no long-term support branches yet.

## Disclosure

We will acknowledge a report, investigate, and coordinate a fix and disclosure date. Please give us reasonable time before publishing.
