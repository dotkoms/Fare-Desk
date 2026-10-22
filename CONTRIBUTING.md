# Contributing to Fare Desk

Fare Desk is an open-source flight finder and buy/wait engine. The code is GPL-3.0. Contributions are accepted under the [Contributor License Agreement](CLA.md).

## Before you start

- Open an issue or discussion for anything larger than a typo or a small fix.
- Check the roadmap and the build planner: [docs/22-build-planner.md](docs/22-build-planner.md).
- Do not add scraping for a source that prohibits it. Read [docs/03-public-apis-vs-scraping.md](docs/03-public-apis-vs-scraping.md) first.

## CLA

Every contributor must sign the [CLA](CLA.md). The CLA lets the project owner license the code under GPL-3.0 and, where needed, under a commercial licence. You keep your copyright.

A pull request cannot be merged until the CLA check passes. Corporate contributors must sign a corporate CLA.

## Code

- Rust: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
- TypeScript: `npm run lint`, `npm run typecheck`, `npm test`.
- Keep the deterministic price path out of the LLM. Extraction, normalisation, fingerprinting and ranking are code.
- No secrets, API keys, cookies or personal data in the repository.
- Pin dependencies and update the lockfile deliberately.
- Add a test for every bug fix and every new adapter.

## Data contributions

Data is as important as code, and easier to get wrong.

- Include the source URL, licence, coverage and the date you checked it.
- Fee and access data: one record per airline/airport, with `source` and `checked_at`.
- Do not submit fare history, scraped fare content, API keys or personal data.
- Airport and route corrections: cite the source.
- If a dataset is ODbL, note the attribution and share-alike obligation.

## Security

Do not open a public issue for a vulnerability. Use GitHub’s private vulnerability reporting or contact the maintainer directly. See [SECURITY.md](SECURITY.md).

## Pull requests

- One change per PR.
- Describe the user-visible effect and the test you ran.
- Include screenshots for UI changes, at desktop and mobile widths.
- Do not weaken a check to get green. Fix the cause.
- By submitting a PR, you confirm you have read and agree to the CLA.

## Licence of contributions

Code contributions are licensed to the project under GPL-3.0 and the CLA. Documentation contributions are accepted under CC BY 4.0 unless you state otherwise. Data contributions must carry a licence compatible with the project’s data policy.
