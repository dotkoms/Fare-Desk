use std::path::Path;
use std::sync::Mutex;

use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Watch {
    pub id: i64,
    pub email: String,
    pub route: String,
    pub travel_date: NaiveDate,
    pub threshold: Option<f64>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRow {
    pub id: i64,
    pub email: String,
    pub route: String,
    pub travel_date: NaiveDate,
    pub price: f64,
    pub kind: String,
    pub created_at: String,
    pub delivered_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReliabilityAggregate {
    pub carrier: String,
    pub flights: u32,
    pub on_time: u32,
    pub first_date: NaiveDate,
    pub last_date: NaiveDate,
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

            CREATE TABLE IF NOT EXISTS pos_quotes (
                route              TEXT NOT NULL,
                travel_date        TEXT NOT NULL,
                storefront_country TEXT NOT NULL,
                currency           TEXT NOT NULL,
                fare               REAL NOT NULL,
                bags               REAL NOT NULL,
                fx_fee             REAL NOT NULL,
                net_total          REAL NOT NULL,
                status             TEXT NOT NULL,
                provider           TEXT NOT NULL,
                observed_at        TEXT NOT NULL,
                PRIMARY KEY (route, travel_date, storefront_country, observed_at)
            );

            CREATE INDEX IF NOT EXISTS pos_quotes_lookup
                ON pos_quotes (route, travel_date, storefront_country, observed_at);

            CREATE TABLE IF NOT EXISTS reliability (
                origin     TEXT NOT NULL,
                dest       TEXT NOT NULL,
                carrier    TEXT NOT NULL,
                month      TEXT NOT NULL,
                flights    INTEGER NOT NULL,
                on_time    INTEGER NOT NULL,
                first_date TEXT NOT NULL,
                last_date  TEXT NOT NULL,
                PRIMARY KEY (origin, dest, carrier, month)
            );

            CREATE INDEX IF NOT EXISTS reliability_route ON reliability (origin, dest);

            CREATE TABLE IF NOT EXISTS watches (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                email       TEXT NOT NULL,
                route       TEXT NOT NULL,
                travel_date TEXT NOT NULL,
                threshold   REAL,
                active      INTEGER NOT NULL DEFAULT 1,
                created_at  TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS watches_email ON watches (email, active);
            CREATE INDEX IF NOT EXISTS watches_route ON watches (route, travel_date, active);

            CREATE TABLE IF NOT EXISTS alerts (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                watch_id     INTEGER NOT NULL,
                email        TEXT NOT NULL,
                route        TEXT NOT NULL,
                travel_date  TEXT NOT NULL,
                price        REAL NOT NULL,
                kind         TEXT NOT NULL,
                created_at   TEXT NOT NULL,
                delivered_at TEXT
            );

            CREATE INDEX IF NOT EXISTS alerts_email ON alerts (email, created_at);
            CREATE INDEX IF NOT EXISTS alerts_watch ON alerts (watch_id, created_at);
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

    /// Store one storefront quote for a day, replacing any earlier quote from
    /// the same market that day.
    #[allow(clippy::too_many_arguments)]
    pub fn record_pos_quote(
        &self,
        route: &str,
        travel_date: NaiveDate,
        storefront: &str,
        currency: &str,
        fare: f64,
        bags: f64,
        fx_fee: f64,
        net_total: f64,
        status: &str,
        provider: &str,
        observed_on: NaiveDate,
    ) -> Result<(), DbError> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO pos_quotes
             (route, travel_date, storefront_country, currency, fare, bags, fx_fee, net_total, status, provider, observed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(route, travel_date, storefront_country, observed_at)
             DO UPDATE SET currency = excluded.currency, fare = excluded.fare, bags = excluded.bags,
                           fx_fee = excluded.fx_fee, net_total = excluded.net_total,
                           status = excluded.status, provider = excluded.provider",
            params![
                route,
                travel_date.format("%Y-%m-%d").to_string(),
                storefront,
                currency,
                fare,
                bags,
                fx_fee,
                net_total,
                status,
                provider,
                observed_on.format("%Y-%m-%d").to_string()
            ],
        )?;
        Ok(())
    }

    /// True when this market was already cheaper than `net_below` on an
    /// earlier day in the window. Used for the persistence threshold.
    #[allow(clippy::too_many_arguments)]
    pub fn prior_pos_gap(
        &self,
        route: &str,
        travel_date: NaiveDate,
        storefront: &str,
        net_below: f64,
        since: NaiveDate,
        before: NaiveDate,
    ) -> Result<bool, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pos_quotes
             WHERE route = ?1 AND travel_date = ?2 AND storefront_country = ?3
               AND net_total <= ?4 AND observed_at >= ?5 AND observed_at < ?6",
            params![
                route,
                travel_date.format("%Y-%m-%d").to_string(),
                storefront,
                net_below,
                since.format("%Y-%m-%d").to_string(),
                before.format("%Y-%m-%d").to_string()
            ],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Upsert monthly on-time counts per route and carrier.
    pub fn record_reliability(
        &self,
        rows: &[faredesk_core::ReliabilityCount],
    ) -> Result<usize, DbError> {
        let mut conn = self.conn.lock().expect("db lock");
        let tx = conn.transaction()?;
        for row in rows {
            tx.execute(
                "INSERT INTO reliability (origin, dest, carrier, month, flights, on_time, first_date, last_date)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(origin, dest, carrier, month)
                 DO UPDATE SET flights = excluded.flights, on_time = excluded.on_time,
                               first_date = excluded.first_date, last_date = excluded.last_date",
                params![
                    row.origin,
                    row.dest,
                    row.carrier,
                    row.month,
                    row.flights,
                    row.on_time,
                    row.first_date.format("%Y-%m-%d").to_string(),
                    row.last_date.format("%Y-%m-%d").to_string()
                ],
            )?;
        }
        tx.commit()?;
        Ok(rows.len())
    }

    /// Summed counts per carrier for a route, across every ingested month.
    pub fn reliability_for(
        &self,
        origin: &str,
        dest: &str,
    ) -> Result<Vec<ReliabilityAggregate>, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let mut statement = conn.prepare(
            "SELECT carrier, SUM(flights), SUM(on_time), MIN(first_date), MAX(last_date)
             FROM reliability
             WHERE origin = ?1 AND dest = ?2
             GROUP BY carrier
             ORDER BY carrier",
        )?;
        let rows = statement.query_map(params![origin, dest], |row| {
            let carrier: String = row.get(0)?;
            let flights: i64 = row.get(1)?;
            let on_time: i64 = row.get(2)?;
            let first: String = row.get(3)?;
            let last: String = row.get(4)?;
            Ok((carrier, flights, on_time, first, last))
        })?;

        let mut aggregates = Vec::new();
        for row in rows {
            let (carrier, flights, on_time, first, last) = row?;
            let (Ok(first_date), Ok(last_date)) = (
                NaiveDate::parse_from_str(&first, "%Y-%m-%d"),
                NaiveDate::parse_from_str(&last, "%Y-%m-%d"),
            ) else {
                continue;
            };
            aggregates.push(ReliabilityAggregate {
                carrier,
                flights: flights.max(0) as u32,
                on_time: on_time.max(0) as u32,
                first_date,
                last_date,
            });
        }
        Ok(aggregates)
    }

    // -- Watches and alerts -------------------------------------------------

    pub fn create_watch(
        &self,
        email: &str,
        route: &str,
        travel_date: NaiveDate,
        threshold: Option<f64>,
        created_at: &str,
    ) -> Result<i64, DbError> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO watches (email, route, travel_date, threshold, active, created_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5)",
            params![
                email,
                route,
                travel_date.format("%Y-%m-%d").to_string(),
                threshold,
                created_at
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_watches(&self, email: &str) -> Result<Vec<Watch>, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let mut statement = conn.prepare(
            "SELECT id, email, route, travel_date, threshold, created_at
             FROM watches WHERE email = ?1 AND active = 1 ORDER BY id DESC",
        )?;
        let rows = statement.query_map(params![email], map_watch)?;
        collect_watches(rows)
    }

    pub fn active_watches(&self) -> Result<Vec<Watch>, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let mut statement = conn.prepare(
            "SELECT id, email, route, travel_date, threshold, created_at
             FROM watches WHERE active = 1 ORDER BY id",
        )?;
        let rows = statement.query_map([], map_watch)?;
        collect_watches(rows)
    }

    /// Soft-delete a watch the caller owns.
    pub fn delete_watch(&self, id: i64, email: &str) -> Result<bool, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let changed = conn.execute(
            "UPDATE watches SET active = 0 WHERE id = ?1 AND email = ?2 AND active = 1",
            params![id, email],
        )?;
        Ok(changed > 0)
    }

    pub fn alerts_for_watch_since(&self, watch_id: i64, since: &str) -> Result<i64, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let count = conn.query_row(
            "SELECT COUNT(*) FROM alerts WHERE watch_id = ?1 AND created_at >= ?2",
            params![watch_id, since],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn alerts_for_email_route_since(
        &self,
        email: &str,
        route: &str,
        since: &str,
    ) -> Result<i64, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let count = conn.query_row(
            "SELECT COUNT(*) FROM alerts WHERE email = ?1 AND route = ?2 AND created_at >= ?3",
            params![email, route, since],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn alerts_for_email_since(&self, email: &str, since: &str) -> Result<i64, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let count = conn.query_row(
            "SELECT COUNT(*) FROM alerts WHERE email = ?1 AND created_at >= ?2",
            params![email, since],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_alert(
        &self,
        watch_id: i64,
        email: &str,
        route: &str,
        travel_date: NaiveDate,
        price: f64,
        kind: &str,
        created_at: &str,
    ) -> Result<i64, DbError> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO alerts (watch_id, email, route, travel_date, price, kind, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                watch_id,
                email,
                route,
                travel_date.format("%Y-%m-%d").to_string(),
                price,
                kind,
                created_at
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn alerts_for_email(&self, email: &str, limit: i64) -> Result<Vec<AlertRow>, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let mut statement = conn.prepare(
            "SELECT id, email, route, travel_date, price, kind, created_at, delivered_at
             FROM alerts WHERE email = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![email, limit], map_alert)?;
        collect_alerts(rows)
    }

    pub fn undelivered_alerts(&self, limit: i64) -> Result<Vec<AlertRow>, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let mut statement = conn.prepare(
            "SELECT id, email, route, travel_date, price, kind, created_at, delivered_at
             FROM alerts WHERE delivered_at IS NULL ORDER BY id LIMIT ?1",
        )?;
        let rows = statement.query_map(params![limit], map_alert)?;
        collect_alerts(rows)
    }

    pub fn mark_alerts_delivered(&self, ids: &[i64], at: &str) -> Result<usize, DbError> {
        let conn = self.conn.lock().expect("db lock");
        let mut changed = 0;
        for id in ids {
            changed += conn.execute(
                "UPDATE alerts SET delivered_at = ?1 WHERE id = ?2 AND delivered_at IS NULL",
                params![at, id],
            )?;
        }
        Ok(changed)
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

type WatchRow = (i64, String, String, String, Option<f64>, String);
type AlertRowData = (
    i64,
    String,
    String,
    String,
    f64,
    String,
    String,
    Option<String>,
);

fn map_watch(row: &rusqlite::Row<'_>) -> rusqlite::Result<WatchRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
    ))
}

fn collect_watches(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<WatchRow>>,
) -> Result<Vec<Watch>, DbError> {
    let mut watches = Vec::new();
    for row in rows {
        let (id, email, route, travel_date, threshold, created_at) = row?;
        if let Ok(travel_date) = NaiveDate::parse_from_str(&travel_date, "%Y-%m-%d") {
            watches.push(Watch {
                id,
                email,
                route,
                travel_date,
                threshold,
                created_at,
            });
        }
    }
    Ok(watches)
}

fn map_alert(row: &rusqlite::Row<'_>) -> rusqlite::Result<AlertRowData> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
    ))
}

fn collect_alerts(
    rows: rusqlite::MappedRows<
        '_,
        impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<AlertRowData>,
    >,
) -> Result<Vec<AlertRow>, DbError> {
    let mut alerts = Vec::new();
    for row in rows {
        let (id, email, route, travel_date, price, kind, created_at, delivered_at) = row?;
        if let Ok(travel_date) = NaiveDate::parse_from_str(&travel_date, "%Y-%m-%d") {
            alerts.push(AlertRow {
                id,
                email,
                route,
                travel_date,
                price,
                kind,
                created_at,
                delivered_at,
            });
        }
    }
    Ok(alerts)
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
    fn watches_and_alert_caps_round_trip() {
        let db = Db::open_in_memory().unwrap();
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        let id = db
            .create_watch(
                "a@example.com",
                "BER-LIS",
                travel,
                Some(100.0),
                "2026-10-23T09:00:00Z",
            )
            .unwrap();
        assert_eq!(db.list_watches("a@example.com").unwrap().len(), 1);
        assert_eq!(db.active_watches().unwrap().len(), 1);

        db.record_alert(
            id,
            "a@example.com",
            "BER-LIS",
            travel,
            99.0,
            "threshold",
            "2026-10-23T09:05:00Z",
        )
        .unwrap();
        assert_eq!(
            db.alerts_for_email_since("a@example.com", "2026-10-20T00:00:00Z")
                .unwrap(),
            1
        );
        assert_eq!(
            db.alerts_for_email_route_since("a@example.com", "BER-LIS", "2026-10-20T00:00:00Z")
                .unwrap(),
            1
        );
        assert_eq!(
            db.alerts_for_email("a@example.com", 10).unwrap()[0].price,
            99.0
        );
        assert_eq!(db.undelivered_alerts(10).unwrap().len(), 1);
        assert_eq!(
            db.mark_alerts_delivered(
                &[db.undelivered_alerts(10).unwrap()[0].id],
                "2026-10-23T10:00:00Z"
            )
            .unwrap(),
            1
        );
        assert!(db.undelivered_alerts(10).unwrap().is_empty());
        assert!(db.delete_watch(id, "a@example.com").unwrap());
        assert!(db.list_watches("a@example.com").unwrap().is_empty());
    }

    #[test]
    fn reliability_counts_aggregate_by_carrier() {
        let db = Db::open_in_memory().unwrap();
        let day = NaiveDate::from_ymd_opt(2025, 12, 1).unwrap();
        let rows = vec![
            faredesk_core::ReliabilityCount {
                origin: "JFK".into(),
                dest: "LAX".into(),
                carrier: "AA".into(),
                month: "2025-11".into(),
                flights: 100,
                on_time: 80,
                first_date: day,
                last_date: day,
            },
            faredesk_core::ReliabilityCount {
                origin: "JFK".into(),
                dest: "LAX".into(),
                carrier: "AA".into(),
                month: "2025-12".into(),
                flights: 50,
                on_time: 45,
                first_date: day,
                last_date: day,
            },
        ];
        assert_eq!(db.record_reliability(&rows).unwrap(), 2);
        let aggregates = db.reliability_for("JFK", "LAX").unwrap();
        assert_eq!(aggregates.len(), 1);
        assert_eq!(aggregates[0].flights, 150);
        assert_eq!(aggregates[0].on_time, 125);
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
