//! BTS On-Time Performance ingest.
//!
//! The Bureau of Transportation Statistics publishes one monthly zip of
//! reporting-carrier flights. We stream the CSV out of the zip, aggregate
//! on-time counts per route and carrier, and store them for the reliability
//! endpoint. BTS data is public domain.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use faredesk_core::ReliabilityCount;
use faredesk_store::Db;

#[derive(Debug, thiserror::Error)]
pub enum BtsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("csv: {0}")]
    Csv(#[from] csv::Error),
    #[error("download failed: {0}")]
    Http(String),
    #[error("the archive contains no CSV")]
    Empty,
}

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestReport {
    pub months: usize,
    pub rows: usize,
    pub flights: u32,
    pub on_time: u32,
    pub routes: usize,
    pub carriers: usize,
}

impl IngestReport {
    pub fn from_rows(rows: &[ReliabilityCount]) -> Self {
        let months: BTreeSet<&str> = rows.iter().map(|row| row.month.as_str()).collect();
        let carriers: BTreeSet<&str> = rows.iter().map(|row| row.carrier.as_str()).collect();
        let routes: BTreeSet<(&str, &str)> = rows
            .iter()
            .map(|row| (row.origin.as_str(), row.dest.as_str()))
            .collect();
        Self {
            months: months.len(),
            rows: rows.len(),
            flights: rows.iter().map(|row| row.flights).sum(),
            on_time: rows.iter().map(|row| row.on_time).sum(),
            routes: routes.len(),
            carriers: carriers.len(),
        }
    }
}

/// Aggregate a BTS on-time CSV (streamed, never fully held in memory).
pub fn aggregate_csv<R: Read>(reader: R) -> Result<Vec<ReliabilityCount>, BtsError> {
    let mut csv = csv::ReaderBuilder::new().flexible(true).from_reader(reader);
    let headers = csv.headers()?.clone();
    let index = |name: &str| headers.iter().position(|header| header == name);
    let flight_date = index("FlightDate").ok_or(BtsError::Empty)?;
    let carrier = index("Reporting_Airline").ok_or(BtsError::Empty)?;
    let origin = index("Origin").ok_or(BtsError::Empty)?;
    let dest = index("Dest").ok_or(BtsError::Empty)?;
    let arr_del15 = index("ArrDel15").ok_or(BtsError::Empty)?;
    let cancelled = index("Cancelled");
    let diverted = index("Diverted");

    type RouteKey = (String, String, String, String);
    type Aggregate = (u32, u32, NaiveDate, NaiveDate);
    let mut aggregates: BTreeMap<RouteKey, Aggregate> = BTreeMap::new();

    for record in csv.records() {
        let record = record?;
        let get = |index: usize| record.get(index).unwrap_or("").trim();
        let is_flag = |index: Option<usize>| {
            index
                .and_then(|index| record.get(index))
                .map(|value| matches!(value.trim(), "1" | "1.00" | "1.0"))
                .unwrap_or(false)
        };
        if is_flag(cancelled) || is_flag(diverted) {
            continue;
        }
        let Ok(delayed) = get(arr_del15).parse::<f64>() else {
            continue;
        };
        let Ok(date) = NaiveDate::parse_from_str(get(flight_date), "%Y-%m-%d") else {
            continue;
        };
        let origin = get(origin).to_ascii_uppercase();
        let dest = get(dest).to_ascii_uppercase();
        let carrier = get(carrier).to_ascii_uppercase();
        if origin.is_empty() || dest.is_empty() || carrier.is_empty() {
            continue;
        }
        let month = date.format("%Y-%m").to_string();
        let entry = aggregates
            .entry((origin, dest, carrier, month))
            .or_insert((0, 0, date, date));
        entry.0 += 1;
        if delayed == 0.0 {
            entry.1 += 1;
        }
        if date < entry.2 {
            entry.2 = date;
        }
        if date > entry.3 {
            entry.3 = date;
        }
    }

    Ok(aggregates
        .into_iter()
        .map(
            |((origin, dest, carrier, month), (flights, on_time, first, last))| ReliabilityCount {
                origin,
                dest,
                carrier,
                month,
                flights,
                on_time,
                first_date: first,
                last_date: last,
            },
        )
        .collect())
}

/// Read the CSV inside a BTS zip, aggregate and store it.
pub fn ingest_zip(path: &Path, db: &Db) -> Result<IngestReport, BtsError> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let name = (0..archive.len())
        .find_map(|index| {
            let entry = archive.by_index(index).ok()?;
            let name = entry.name().to_string();
            name.to_ascii_lowercase().ends_with(".csv").then_some(name)
        })
        .ok_or(BtsError::Empty)?;
    let entry = archive.by_name(&name)?;
    let rows = aggregate_csv(entry)?;
    db.record_reliability(&rows)
        .map_err(|error| BtsError::Http(format!("store: {error}")))?;
    Ok(IngestReport::from_rows(&rows))
}

/// An error with every cause behind it; reqwest's top-level text alone
/// ("error sending request") hides whether it was DNS, TLS or a timeout.
fn error_chain(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

pub async fn download_month(year: i32, month: u32) -> Result<PathBuf, BtsError> {
    let url = format!(
        "https://transtats.bts.gov/PREZIP/On_Time_Reporting_Carrier_On_Time_Performance_1987_present_{year}_{month}.zip"
    );
    let response = reqwest::Client::builder()
        .user_agent("FareDesk/0.1 (+https://github.com/dotkoms/Fare-Desk)")
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_default()
        .get(&url)
        .send()
        .await
        .map_err(|error| BtsError::Http(error_chain(&error)))?;
    if !response.status().is_success() {
        return Err(BtsError::Http(format!(
            "transtats returned {}",
            response.status()
        )));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| BtsError::Http(error_chain(&error)))?;
    let path = std::env::temp_dir().join(format!("bts-on-time-{year}-{month}.zip"));
    std::fs::write(&path, bytes)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str = "FlightDate,Reporting_Airline,Origin,Dest,ArrDel15,Cancelled,Diverted\n\
2025-12-01,AA,JFK,LAX,0.00,0.00,0.00\n\
2025-12-01,AA,JFK,LAX,0.00,0.00,0.00\n\
2025-12-02,AA,JFK,LAX,1.00,0.00,0.00\n\
2025-12-02,AA,JFK,LAX,,1.00,0.00\n\
2025-12-02,UA,JFK,LAX,0.00,0.00,0.00\n";

    #[test]
    fn aggregates_on_time_counts_and_skips_cancelled() {
        let rows = aggregate_csv(CSV.as_bytes()).unwrap();
        assert_eq!(rows.len(), 2);
        let aa = rows.iter().find(|row| row.carrier == "AA").unwrap();
        assert_eq!(aa.flights, 3);
        assert_eq!(aa.on_time, 2);
        assert_eq!(aa.month, "2025-12");
        assert_eq!(aa.first_date, NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
        assert_eq!(aa.last_date, NaiveDate::from_ymd_opt(2025, 12, 2).unwrap());
        let ua = rows.iter().find(|row| row.carrier == "UA").unwrap();
        assert_eq!((ua.flights, ua.on_time), (1, 1));
    }

    #[test]
    fn report_totals() {
        let rows = aggregate_csv(CSV.as_bytes()).unwrap();
        let report = IngestReport::from_rows(&rows);
        assert_eq!(report.flights, 4);
        assert_eq!(report.on_time, 3);
        assert_eq!(report.carriers, 2);
        assert_eq!(report.routes, 1);
        assert_eq!(report.months, 1);
    }

    #[test]
    fn missing_headers_is_an_error() {
        assert!(aggregate_csv("foo,bar\n1,2\n".as_bytes()).is_err());
    }
}
