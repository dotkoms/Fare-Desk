//! Daily fare-history collection.
//!
//! One search per route per day, one travel date per route rotating through
//! its window, one row per route, travel date and observation day: the lowest
//! price and the offer that produced it.

pub mod alerts;
pub mod bts;

use std::path::Path;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::Deserialize;

use faredesk_core::{fingerprint, price_offer, SearchQuery};
use faredesk_sources::{ResolvedQuery, Source};
use faredesk_store::{QuotaOutcome, Store};

pub const DEFAULT_MIN_OBSERVATIONS: u32 = 14;

#[derive(Debug, Deserialize)]
pub struct Watchlist {
    #[serde(default)]
    pub home_airports: Vec<String>,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub routes: Vec<WatchRoute>,
    #[serde(default)]
    pub baseline: Baseline,
}

#[derive(Debug, Deserialize)]
pub struct WatchRoute {
    #[serde(default)]
    pub name: String,
    pub origin: Vec<String>,
    pub destination: Vec<String>,
    pub date_window: String,
    #[serde(default)]
    pub bags: Option<String>,
    #[serde(default)]
    pub modes: Vec<String>,
    #[serde(default)]
    pub cadence: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Baseline {
    #[serde(default)]
    pub window_days: Option<u32>,
    #[serde(default)]
    pub min_observations: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum WatchlistError {
    #[error("could not read the watchlist: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not parse the watchlist: {0}")]
    Yaml(#[from] serde_yml::Error),
    #[error("{0}")]
    Invalid(String),
}

fn default_currency() -> String {
    "EUR".to_string()
}

impl Watchlist {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, WatchlistError> {
        let text = std::fs::read_to_string(path)?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, WatchlistError> {
        let watchlist: Watchlist = serde_yml::from_str(text)?;
        if watchlist.routes.is_empty() {
            return Err(WatchlistError::Invalid(
                "the watchlist has no routes".to_string(),
            ));
        }
        for route in &watchlist.routes {
            route.window()?;
            if route.origin.is_empty() || route.destination.is_empty() {
                return Err(WatchlistError::Invalid(format!(
                    "route {:?} needs origin and destination airports",
                    route.name
                )));
            }
        }
        Ok(watchlist)
    }

    pub fn min_observations(&self) -> u32 {
        self.baseline
            .min_observations
            .unwrap_or(DEFAULT_MIN_OBSERVATIONS)
    }
}

impl WatchRoute {
    pub fn window(&self) -> Result<(NaiveDate, NaiveDate), WatchlistError> {
        let (start, end) = self.date_window.split_once('/').ok_or_else(|| {
            WatchlistError::Invalid(format!(
                "bad date_window {:?}; use start/end",
                self.date_window
            ))
        })?;
        let start = NaiveDate::parse_from_str(start.trim(), "%Y-%m-%d")
            .map_err(|error| WatchlistError::Invalid(format!("bad date_window start: {error}")))?;
        let end = NaiveDate::parse_from_str(end.trim(), "%Y-%m-%d")
            .map_err(|error| WatchlistError::Invalid(format!("bad date_window end: {error}")))?;
        if end < start {
            return Err(WatchlistError::Invalid(format!(
                "date_window ends before it starts: {}",
                self.date_window
            )));
        }
        Ok((start, end))
    }

    /// The travel date for this route on `today`: one step through the window
    /// per calendar day, so the whole window is sampled over time.
    pub fn travel_date(&self, today: NaiveDate) -> Result<NaiveDate, WatchlistError> {
        let (start, end) = self.window()?;
        let span = (end - start).num_days() + 1;
        let offset = days_since_epoch(today).rem_euclid(span);
        Ok(start + Duration::days(offset))
    }

    /// Checked bags per traveller from the free-text watchlist field.
    pub fn checked_bags(&self) -> u8 {
        let Some(bags) = &self.bags else {
            return 0;
        };
        let lower = bags.trim().to_lowercase();
        if lower.contains("carry_on") || lower.contains("carry-on") || lower == "none" {
            return 0;
        }
        lower
            .split_whitespace()
            .next()
            .and_then(|first| first.parse().ok())
            .unwrap_or(0)
    }
}

fn days_since_epoch(date: NaiveDate) -> i64 {
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch");
    date.signed_duration_since(epoch).num_days()
}

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub searches: u32,
    pub recorded: u32,
    pub would_record: u32,
    pub skipped_existing: u32,
    pub failed: u32,
    pub quota_exhausted: bool,
    pub messages: Vec<String>,
}

pub struct Collector {
    pub store: Store,
    pub source: Source,
    pub currency: String,
    pub adults: u8,
    pub daily_limit: u32,
    pub now: DateTime<Utc>,
    pub dry_run: bool,
    /// Stop after this many source searches. None means unlimited.
    pub max_searches: Option<usize>,
}

impl Collector {
    /// Run one collection pass. A route and travel date that already has an
    /// observation today is skipped, so re-running the collector is safe.
    pub async fn run(&self, watchlist: &Watchlist) -> Report {
        let mut report = Report::default();
        let today = self.now.date_naive();
        let live = self.source.is_live();

        'routes: for route in &watchlist.routes {
            let travel_date = match route.travel_date(today) {
                Ok(date) => date,
                Err(error) => {
                    report.failed += 1;
                    report.messages.push(error.to_string());
                    continue;
                }
            };

            for origin_code in &route.origin {
                for destination_code in &route.destination {
                    if origin_code.eq_ignore_ascii_case(destination_code) {
                        continue;
                    }
                    let Some(origin) = self.store.reference.resolve(origin_code).cloned() else {
                        report.failed += 1;
                        report
                            .messages
                            .push(format!("unknown origin airport {origin_code}"));
                        continue;
                    };
                    let Some(destination) = self.store.reference.resolve(destination_code).cloned()
                    else {
                        report.failed += 1;
                        report
                            .messages
                            .push(format!("unknown destination airport {destination_code}"));
                        continue;
                    };
                    let route_key = format!("{}-{}", origin.code, destination.code);

                    match self
                        .store
                        .db
                        .has_observation(&route_key, travel_date, today)
                    {
                        Ok(true) => {
                            report.skipped_existing += 1;
                            continue;
                        }
                        Ok(false) => {}
                        Err(error) => {
                            report.failed += 1;
                            report
                                .messages
                                .push(format!("{route_key}: history lookup failed: {error}"));
                            continue;
                        }
                    }

                    if live {
                        match self
                            .store
                            .db
                            .consume_quota(self.source.id(), today, self.daily_limit)
                        {
                            Ok(QuotaOutcome::Granted { .. }) => {}
                            Ok(QuotaOutcome::Exhausted { reset_on, .. }) => {
                                report.quota_exhausted = true;
                                report
                                    .messages
                                    .push(format!("daily quota spent; resuming on {reset_on}"));
                                break 'routes;
                            }
                            Err(error) => {
                                report.failed += 1;
                                report
                                    .messages
                                    .push(format!("quota ledger failed: {error}"));
                                break 'routes;
                            }
                        }
                    }

                    if self
                        .max_searches
                        .is_some_and(|max| report.searches as usize >= max)
                    {
                        break 'routes;
                    }
                    report.searches += 1;
                    let query = SearchQuery {
                        origin: origin.code.clone(),
                        destination: destination.code.clone(),
                        depart_date: travel_date,
                        adults: self.adults.max(1),
                        checked_bags: route.checked_bags(),
                        currency: watchlist.currency.to_uppercase(),
                        ..SearchQuery::default()
                    };
                    let resolved = ResolvedQuery {
                        request: &query,
                        origin: &origin,
                        destination: &destination,
                    };

                    match self.source.search(&resolved).await {
                        Ok(offers) => {
                            let Some((total, offer)) = cheapest(offers, &query) else {
                                report.failed += 1;
                                report
                                    .messages
                                    .push(format!("{route_key}: source returned no priced offers"));
                                continue;
                            };
                            if self.dry_run {
                                report.would_record += 1;
                                continue;
                            }
                            let id = fingerprint(&offer.segments);
                            match self.store.db.record_daily_minimum(
                                &route_key,
                                travel_date,
                                today,
                                total,
                                &offer.currency,
                                self.source.id(),
                                &id,
                            ) {
                                Ok(true) => report.recorded += 1,
                                Ok(false) => report.skipped_existing += 1,
                                Err(error) => {
                                    report.failed += 1;
                                    report
                                        .messages
                                        .push(format!("{route_key}: could not record: {error}"));
                                }
                            }
                        }
                        Err(error) => {
                            report.failed += 1;
                            report.messages.push(format!("{route_key}: {error}"));
                        }
                    }
                }
            }
        }

        report
    }
}

fn cheapest(
    offers: Vec<faredesk_core::Offer>,
    query: &SearchQuery,
) -> Option<(f64, faredesk_core::Offer)> {
    offers
        .into_iter()
        .map(|offer| {
            let (total, _) = price_offer(&offer, query);
            (total, offer)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use faredesk_sources::fixtures::FixtureSource;

    fn fixture_source(store: &Store) -> Source {
        Source::Fixtures(FixtureSource::new(
            store.reference.place_map(),
            store.providers.clone(),
        ))
    }

    fn collector(store: Store, source: Source) -> Collector {
        Collector {
            store,
            source,
            currency: "EUR".into(),
            adults: 1,
            daily_limit: 100,
            now: DateTime::parse_from_rfc3339("2026-10-22T06:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            dry_run: false,
            max_searches: None,
        }
    }

    #[test]
    fn example_watchlist_parses() {
        let text = include_str!("../../../../data/watchlist.example.yaml");
        let watchlist = Watchlist::parse(text).expect("example watchlist");
        assert!(watchlist.routes.len() >= 2);
        assert_eq!(watchlist.currency, "EUR");
        assert_eq!(watchlist.routes[0].checked_bags(), 1);
        assert_eq!(watchlist.routes[1].checked_bags(), 0);
        assert_eq!(watchlist.min_observations(), 14);
    }

    #[test]
    fn travel_date_rotates_through_the_window() {
        let route = WatchRoute {
            name: "test".into(),
            origin: vec!["BER".into()],
            destination: vec!["LIS".into()],
            date_window: "2027-03-01/2027-03-08".into(),
            bags: None,
            modes: vec![],
            cadence: None,
        };
        let first = route
            .travel_date(NaiveDate::from_ymd_opt(2026, 10, 22).unwrap())
            .unwrap();
        let next = route
            .travel_date(NaiveDate::from_ymd_opt(2026, 10, 23).unwrap())
            .unwrap();
        assert!((NaiveDate::from_ymd_opt(2027, 3, 1).unwrap()
            ..=NaiveDate::from_ymd_opt(2027, 3, 8).unwrap())
            .contains(&first));
        assert_eq!(next, first + Duration::days(1));
    }

    #[tokio::test]
    async fn collector_records_the_cheapest_offer_once_per_day() {
        let store = Store::in_memory().unwrap();
        let source = fixture_source(&store);
        let collector = collector(store, source);

        let watchlist = Watchlist::parse(
            r#"
currency: EUR
routes:
  - name: Berlin to Lisbon
    origin: [BER]
    destination: [LIS]
    date_window: 2027-03-01/2027-03-01
    bags: 1 checked
"#,
        )
        .unwrap();

        let first = collector.run(&watchlist).await;
        assert_eq!(first.searches, 1);
        assert_eq!(first.recorded, 1);
        assert_eq!(first.failed, 0);

        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        let points = collector.store.db.daily_minima("BER-LIS", travel).unwrap();
        assert_eq!(points.len(), 1);
        // One adult, one checked bag: TAP €119 (bag included) is the cheapest.
        assert_eq!(points[0].1, 119.0);

        let second = collector.run(&watchlist).await;
        assert_eq!(second.searches, 0);
        assert_eq!(second.skipped_existing, 1);
        assert_eq!(
            collector
                .store
                .db
                .daily_minima("BER-LIS", travel)
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn dry_run_writes_nothing() {
        let store = Store::in_memory().unwrap();
        let source = fixture_source(&store);
        let mut collector = collector(store, source);
        collector.dry_run = true;

        let watchlist = Watchlist::parse(
            r#"
routes:
  - origin: [BER]
    destination: [LIS]
    date_window: 2027-03-01/2027-03-01
"#,
        )
        .unwrap();

        let report = collector.run(&watchlist).await;
        assert_eq!(report.would_record, 1);
        assert_eq!(report.recorded, 0);
        assert_eq!(
            collector
                .store
                .db
                .daily_minima("BER-LIS", NaiveDate::from_ymd_opt(2027, 3, 1).unwrap())
                .unwrap()
                .len(),
            0
        );
    }
}
