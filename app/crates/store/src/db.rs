use std::path::Path;
use std::sync::Mutex;

use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaOutcome {
    /// The call may proceed; `used` counts this call.
    Granted { used: u32, limit: u32 },
    /// The daily allowance is spent.
    Exhausted {
        used: u32,
        limit: u32,
        reset_on: NaiveDate,
    },
}

/// SQLite-backed ledger. One process holds one connection behind a mutex;
/// quota checks are a few reads and writes, so this stays cheap.
#[derive(Debug)]
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    pub fn open_in_memory() -> Result<Self, DbError> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Self, DbError> {
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS quota_ledger (
                source TEXT NOT NULL,
                day    TEXT NOT NULL,
                used   INTEGER NOT NULL DEFAULT 0,
                limit_ INTEGER NOT NULL,
                PRIMARY KEY (source, day)
            );

            CREATE TABLE IF NOT EXISTS fare_history (
                route       TEXT NOT NULL,
                travel_date TEXT NOT NULL,
                observed_on TEXT NOT NULL,
                price       REAL NOT NULL,
                currency    TEXT NOT NULL,
                source      TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                PRIMARY KEY (route, travel_date, observed_on, fingerprint)
            );

            CREATE INDEX IF NOT EXISTS fare_history_route
                ON fare_history (route, travel_date, observed_on);
            "#,
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Count this call against `source` for `day`, enforcing `limit`.
    pub fn consume_quota(
        &self,
        source: &str,
        day: NaiveDate,
        limit: u32,
    ) -> Result<QuotaOutcome, DbError> {
        let mut conn = self.conn.lock().expect("db lock");
        let tx = conn.transaction()?;
        let day = day.format("%Y-%m-%d").to_string();

        // Start the row at the current limit so a changed limit takes effect.
        tx.execute(
            "INSERT INTO quota_ledger (source, day, used, limit_) VALUES (?1, ?2, 0, ?3)
             ON CONFLICT(source, day) DO UPDATE SET limit_ = excluded.limit_",
            params![source, day, limit],
        )?;

        let used: u32 = tx.query_row(
            "SELECT used FROM quota_ledger WHERE source = ?1 AND day = ?2",
            params![source, day],
            |row| row.get(0),
        )?;

        if used >= limit {
            tx.commit()?;
            return Ok(QuotaOutcome::Exhausted {
                used,
                limit,
                reset_on: day_after(day.as_str()),
            });
        }

        tx.execute(
            "UPDATE quota_ledger SET used = used + 1 WHERE source = ?1 AND day = ?2",
            params![source, day],
        )?;
        tx.commit()?;
        Ok(QuotaOutcome::Granted {
            used: used + 1,
            limit,
        })
    }

    pub fn quota_used(&self, source: &str, day: NaiveDate) -> Result<u32, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let used = conn
            .query_row(
                "SELECT used FROM quota_ledger WHERE source = ?1 AND day = ?2",
                params![source, day.format("%Y-%m-%d").to_string()],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);
        Ok(used)
    }

    /// Keep exactly one observation per route, travel date and observation
    /// day: the lowest price and the offer that produced it. A later, higher
    /// quote the same day is ignored; a lower one replaces the row.
    #[allow(clippy::too_many_arguments)]
    pub fn record_daily_minimum(
        &self,
        route: &str,
        travel_date: NaiveDate,
        observed_on: NaiveDate,
        price: f64,
        currency: &str,
        source: &str,
        fingerprint: &str,
    ) -> Result<bool, DbError> {
        let mut conn = self.conn.lock().expect("db lock");
        let tx = conn.transaction()?;
        let travel = travel_date.format("%Y-%m-%d").to_string();
        let observed = observed_on.format("%Y-%m-%d").to_string();

        let existing: Option<f64> = tx
            .query_row(
                "SELECT MIN(price) FROM fare_history
                 WHERE route = ?1 AND travel_date = ?2 AND observed_on = ?3",
                params![route, travel, observed],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        if existing.is_some_and(|old| old <= price) {
            tx.commit()?;
            return Ok(false);
        }

        tx.execute(
            "DELETE FROM fare_history
             WHERE route = ?1 AND travel_date = ?2 AND observed_on = ?3",
            params![route, travel, observed],
        )?;
        tx.execute(
            "INSERT INTO fare_history (route, travel_date, observed_on, price, currency, source, fingerprint)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![route, travel, observed, price, currency, source, fingerprint],
        )?;
        tx.commit()?;
        Ok(true)
    }

    /// One point per observation day: the daily minimum, oldest first.
    pub fn daily_minima(
        &self,
        route: &str,
        travel_date: NaiveDate,
    ) -> Result<Vec<(NaiveDate, f64)>, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let mut statement = conn.prepare(
            "SELECT observed_on, MIN(price)
             FROM fare_history
             WHERE route = ?1 AND travel_date = ?2
             GROUP BY observed_on
             ORDER BY observed_on",
        )?;
        let rows = statement.query_map(
            params![route, travel_date.format("%Y-%m-%d").to_string()],
            |row| {
                let day: String = row.get(0)?;
                let price: f64 = row.get(1)?;
                Ok((day, price))
            },
        )?;

        let mut points = Vec::new();
        for row in rows {
            let (day, price) = row?;
            if let Ok(date) = NaiveDate::parse_from_str(&day, "%Y-%m-%d") {
                points.push((date, price));
            }
        }
        Ok(points)
    }

    /// Every observation for a route across travel dates, oldest first.
    pub fn route_observations(
        &self,
        route: &str,
    ) -> Result<Vec<(NaiveDate, NaiveDate, f64)>, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let mut statement = conn.prepare(
            "SELECT travel_date, observed_on, price
             FROM fare_history
             WHERE route = ?1
             ORDER BY travel_date, observed_on",
        )?;
        let rows = statement.query_map(params![route], |row| {
            let travel: String = row.get(0)?;
            let observed: String = row.get(1)?;
            let price: f64 = row.get(2)?;
            Ok((travel, observed, price))
        })?;

        let mut observations = Vec::new();
        for row in rows {
            let (travel, observed, price) = row?;
            if let (Ok(travel_date), Ok(observed_on)) = (
                NaiveDate::parse_from_str(&travel, "%Y-%m-%d"),
                NaiveDate::parse_from_str(&observed, "%Y-%m-%d"),
            ) {
                observations.push((travel_date, observed_on, price));
            }
        }
        Ok(observations)
    }

    /// The currency of the latest observation for a route and travel date.
    pub fn history_currency(
        &self,
        route: &str,
        travel_date: NaiveDate,
    ) -> Result<Option<String>, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let currency = conn
            .query_row(
                "SELECT currency FROM fare_history
                 WHERE route = ?1 AND travel_date = ?2
                 ORDER BY observed_on DESC, price ASC
                 LIMIT 1",
                params![route, travel_date.format("%Y-%m-%d").to_string()],
                |row| row.get(0),
            )
            .optional()?;
        Ok(currency)
    }

    pub fn has_observation(
        &self,
        route: &str,
        travel_date: NaiveDate,
        observed_on: NaiveDate,
    ) -> Result<bool, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM fare_history
             WHERE route = ?1 AND travel_date = ?2 AND observed_on = ?3",
            params![
                route,
                travel_date.format("%Y-%m-%d").to_string(),
                observed_on.format("%Y-%m-%d").to_string()
            ],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn observation_count(&self, route: &str) -> Result<i64, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let count = conn.query_row(
            "SELECT COUNT(*) FROM fare_history WHERE route = ?1",
            params![route],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

fn day_after(day: &str) -> NaiveDate {
    NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.succ_opt())
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2027, 1, 1).expect("valid fallback"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 22).unwrap()
    }

    #[test]
    fn quota_grants_then_exhausts() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(
            db.consume_quota("serpapi", day(), 2).unwrap(),
            QuotaOutcome::Granted { used: 1, limit: 2 }
        );
        assert_eq!(
            db.consume_quota("serpapi", day(), 2).unwrap(),
            QuotaOutcome::Granted { used: 2, limit: 2 }
        );
        assert_eq!(
            db.consume_quota("serpapi", day(), 2).unwrap(),
            QuotaOutcome::Exhausted {
                used: 2,
                limit: 2,
                reset_on: NaiveDate::from_ymd_opt(2026, 10, 23).unwrap()
            }
        );
        assert_eq!(db.quota_used("serpapi", day()).unwrap(), 2);
    }

    #[test]
    fn quota_days_are_independent() {
        let db = Db::open_in_memory().unwrap();
        db.consume_quota("serpapi", day(), 1).unwrap();
        let tomorrow = NaiveDate::from_ymd_opt(2026, 10, 23).unwrap();
        assert_eq!(
            db.consume_quota("serpapi", tomorrow, 1).unwrap(),
            QuotaOutcome::Granted { used: 1, limit: 1 }
        );
    }

    #[test]
    fn daily_minimum_is_kept_and_one_row_per_day() {
        let db = Db::open_in_memory().unwrap();
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        let day = NaiveDate::from_ymd_opt(2026, 10, 22).unwrap();

        assert!(db
            .record_daily_minimum("BER-LIS", travel, day, 129.0, "EUR", "serpapi", "TP-1")
            .unwrap());
        assert!(!db
            .record_daily_minimum("BER-LIS", travel, day, 139.0, "EUR", "serpapi", "U2-2")
            .unwrap());
        assert!(db
            .record_daily_minimum("BER-LIS", travel, day, 119.0, "EUR", "serpapi", "FR-3")
            .unwrap());

        let points = db.daily_minima("BER-LIS", travel).unwrap();
        assert_eq!(points, vec![(day, 119.0)]);
        assert_eq!(
            db.history_currency("BER-LIS", travel).unwrap().as_deref(),
            Some("EUR")
        );
        assert!(db.has_observation("BER-LIS", travel, day).unwrap());
    }

    #[test]
    fn daily_minima_are_grouped_by_observation_day() {
        let db = Db::open_in_memory().unwrap();
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        for (offset, price) in [(0, 131.0), (1, 124.0), (2, 119.0)] {
            let day =
                NaiveDate::from_ymd_opt(2026, 10, 20).unwrap() + chrono::Duration::days(offset);
            db.record_daily_minimum("BER-LIS", travel, day, price, "EUR", "serpapi", "TP-1")
                .unwrap();
        }
        let points = db.daily_minima("BER-LIS", travel).unwrap();
        assert_eq!(points.len(), 3);
        assert_eq!(points.last().unwrap().1, 119.0);
        assert_eq!(db.observation_count("BER-LIS").unwrap(), 3);
    }
}
