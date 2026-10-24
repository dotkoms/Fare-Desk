//! Evaluate fare watches against the latest collected observation.
//!
//! Rules: a threshold watch fires when the price is at or below the threshold;
//! a new-low watch fires when today's price beats every earlier daily minimum.
//! Caps: one alert per route per week per email, three per email per week.
//! Quiet hours are evaluated in UTC. Prices come from live-collected history,
//! which is the verification step.

use chrono::{DateTime, Duration, Utc};
use faredesk_core::{is_new_low, is_quiet, matches_threshold};
use faredesk_store::{Db, DbError};

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertReport {
    pub checked: usize,
    pub fired: usize,
    pub suppressed_quiet: usize,
    pub suppressed_cap: usize,
    pub skipped_stale: usize,
}

pub fn evaluate(
    db: &Db,
    now: DateTime<Utc>,
    quiet_hours: (u32, u32),
) -> Result<AlertReport, DbError> {
    let mut report = AlertReport::default();
    let today = now.date_naive();
    let since = (now - Duration::days(7)).to_rfc3339();

    for watch in db.active_watches()? {
        report.checked += 1;
        let rows = db.daily_minima(&watch.route, watch.travel_date)?;
        let Some((latest_day, price)) = rows.last() else {
            report.skipped_stale += 1;
            continue;
        };
        if *latest_day != today {
            report.skipped_stale += 1;
            continue;
        }

        let previous_min = rows
            .iter()
            .filter(|(day, _)| *day < today)
            .map(|(_, price)| *price)
            .min_by(f64::total_cmp);

        let kind = match watch.threshold {
            Some(threshold) if matches_threshold(*price, threshold) => Some("threshold"),
            Some(_) => None,
            None if is_new_low(*price, previous_min) => Some("new_low"),
            None => None,
        };
        let Some(kind) = kind else {
            continue;
        };

        if db.alerts_for_email_route_since(&watch.email, &watch.route, &since)? >= 1
            || db.alerts_for_email_since(&watch.email, &since)? >= 3
        {
            report.suppressed_cap += 1;
            continue;
        }
        if is_quiet(now, quiet_hours.0, quiet_hours.1) {
            report.suppressed_quiet += 1;
            continue;
        }

        db.record_alert(
            watch.id,
            &watch.email,
            &watch.route,
            watch.travel_date,
            *price,
            kind,
            &now.to_rfc3339(),
        )?;
        report.fired += 1;
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-23T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn travel() -> NaiveDate {
        NaiveDate::from_ymd_opt(2027, 3, 1).unwrap()
    }

    fn seed(db: &Db, today_price: f64, yesterday_price: f64) {
        let today = now().date_naive();
        db.record_daily_minimum(
            "BER-LIS",
            travel(),
            today - Duration::days(1),
            yesterday_price,
            "EUR",
            "serpapi",
            "TP-1",
        )
        .unwrap();
        db.record_daily_minimum(
            "BER-LIS",
            travel(),
            today,
            today_price,
            "EUR",
            "serpapi",
            "TP-1",
        )
        .unwrap();
    }

    #[test]
    fn threshold_watch_fires_once_then_hits_the_cap() {
        let db = Db::open_in_memory().unwrap();
        seed(&db, 99.0, 120.0);
        db.create_watch(
            "a@example.com",
            "BER-LIS",
            travel(),
            Some(100.0),
            "2026-10-01T00:00:00Z",
        )
        .unwrap();

        let first = evaluate(&db, now(), (0, 0)).unwrap();
        assert_eq!(first.fired, 1);
        assert_eq!(first.suppressed_cap, 0);

        let second = evaluate(&db, now(), (0, 0)).unwrap();
        assert_eq!(second.fired, 0);
        assert_eq!(second.suppressed_cap, 1);
        assert_eq!(db.alerts_for_email("a@example.com", 10).unwrap().len(), 1);
    }

    #[test]
    fn new_low_watch_fires_only_below_the_previous_minimum() {
        let db = Db::open_in_memory().unwrap();
        seed(&db, 110.0, 100.0);
        db.create_watch(
            "a@example.com",
            "BER-LIS",
            travel(),
            None,
            "2026-10-01T00:00:00Z",
        )
        .unwrap();
        let report = evaluate(&db, now(), (0, 0)).unwrap();
        assert_eq!(report.fired, 0);
    }

    #[test]
    fn quiet_hours_suppress_without_consuming_the_cap() {
        let db = Db::open_in_memory().unwrap();
        seed(&db, 99.0, 120.0);
        db.create_watch(
            "a@example.com",
            "BER-LIS",
            travel(),
            Some(100.0),
            "2026-10-01T00:00:00Z",
        )
        .unwrap();
        let quiet = evaluate(&db, now(), (8 * 60, 10 * 60)).unwrap();
        assert_eq!(quiet.fired, 0);
        assert_eq!(quiet.suppressed_quiet, 1);
        assert!(db.alerts_for_email("a@example.com", 10).unwrap().is_empty());

        let daytime = evaluate(&db, now(), (23 * 60, 7 * 60)).unwrap();
        assert_eq!(daytime.fired, 1);
    }

    #[test]
    fn stale_history_is_skipped() {
        let db = Db::open_in_memory().unwrap();
        let today = now().date_naive();
        db.record_daily_minimum(
            "BER-LIS",
            travel(),
            today - Duration::days(2),
            99.0,
            "EUR",
            "serpapi",
            "TP-1",
        )
        .unwrap();
        db.create_watch(
            "a@example.com",
            "BER-LIS",
            travel(),
            Some(100.0),
            "2026-10-01T00:00:00Z",
        )
        .unwrap();
        let report = evaluate(&db, now(), (0, 0)).unwrap();
        assert_eq!(report.skipped_stale, 1);
        assert_eq!(report.fired, 0);
    }
}
