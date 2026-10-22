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

    /// Keep the lowest price seen for a route, travel date, observation day and
    /// physical service. Returns true when this call lowered the stored price.
    #[allow(clippy::too_many_arguments)]
    pub fn record_observation(
        &self,
        route: &str,
        travel_date: NaiveDate,
        observed_on: NaiveDate,
        price: f64,
        currency: &str,
        source: &str,
        fingerprint: &str,
    ) -> Result<bool, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let existing: Option<f64> = conn
            .query_row(
                "SELECT price FROM fare_history
                 WHERE route = ?1 AND travel_date = ?2 AND observed_on = ?3 AND fingerprint = ?4",
                params![
                    route,
                    travel_date.format("%Y-%m-%d").to_string(),
                    observed_on.format("%Y-%m-%d").to_string(),
                    fingerprint
                ],
                |row| row.get(0),
            )
            .optional()?;

        match existing {
            Some(old) if old <= price => Ok(false),
            _ => {
                conn.execute(
                    "INSERT INTO fare_history (route, travel_date, observed_on, price, currency, source, fingerprint)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(route, travel_date, observed_on, fingerprint)
                     DO UPDATE SET price = excluded.price, currency = excluded.currency, source = excluded.source",
                    params![
                        route,
                        travel_date.format("%Y-%m-%d").to_string(),
                        observed_on.format("%Y-%m-%d").to_string(),
                        price,
                        currency,
                        source,
                        fingerprint
                    ],
                )?;
                Ok(true)
            }
        }
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
    fn daily_minimum_is_kept() {
        let db = Db::open_in_memory().unwrap();
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        assert!(db
            .record_observation(
                "BER-LIS",
                travel,
                day(),
                129.0,
                "EUR",
                "fixtures",
                "TP-1041-2027-03-01"
            )
            .unwrap());
        assert!(!db
            .record_observation(
                "BER-LIS",
                travel,
                day(),
                139.0,
                "EUR",
                "fixtures",
                "TP-1041-2027-03-01"
            )
            .unwrap());
        assert!(db
            .record_observation(
                "BER-LIS",
                travel,
                day(),
                119.0,
                "EUR",
                "fixtures",
                "TP-1041-2027-03-01"
            )
            .unwrap());
        assert_eq!(db.observation_count("BER-LIS").unwrap(), 1);
    }
}
