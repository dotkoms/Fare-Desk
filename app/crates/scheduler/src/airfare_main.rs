//! Ingest US DOT Consumer Airfare Report (Table 1) CSV into airfare_baselines.
//!
//! Usage:
//!   faredesk-airfare-ingest --file airfare-report-table1.csv
//!
//! Expected columns (SODA CSV): year, quarter, city1, city2, fare, passengers, nsmiles, carrier_lg
//! (extra columns are ignored). City names are used as-is; for airport codes we prefer
//! the first token when the field looks like an IATA code.

use std::path::PathBuf;

use faredesk_store::Db;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("csv: {0}")]
    Csv(#[from] csv::Error),
    #[error("db: {0}")]
    Db(#[from] faredesk_store::DbError),
    #[error("{0}")]
    Other(String),
}

fn main() -> Result<(), Error> {
    let mut file = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--file" {
            file = args.next().map(PathBuf::from);
        }
    }
    let path = file.ok_or_else(|| Error::Other("missing --file".into()))?;
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "faredesk.sqlite".into());
    let db = Db::open(&db_path)?;

    let mut rdr = csv::Reader::from_path(&path)?;
    let headers = rdr.headers()?.clone();
    let idx = |name: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(name));

    let year_i = idx("year").ok_or_else(|| Error::Other("missing year column".into()))?;
    let quarter_i = idx("quarter").ok_or_else(|| Error::Other("missing quarter column".into()))?;
    let city1_i = idx("city1").ok_or_else(|| Error::Other("missing city1 column".into()))?;
    let city2_i = idx("city2").ok_or_else(|| Error::Other("missing city2 column".into()))?;
    let fare_i = idx("fare").ok_or_else(|| Error::Other("missing fare column".into()))?;
    let passengers_i = idx("passengers");
    let miles_i = idx("nsmiles");
    let carrier_i = idx("carrier_lg");

    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        let get = |i: usize| record.get(i).unwrap_or("").trim().to_string();
        let year: i32 = get(year_i).parse().unwrap_or(0);
        let quarter: i32 = get(quarter_i).parse().unwrap_or(0);
        if year == 0 || !(1..=4).contains(&quarter) {
            continue;
        }
        let origin = normalize_code(&get(city1_i));
        let dest = normalize_code(&get(city2_i));
        if origin.len() != 3 || dest.len() != 3 {
            continue;
        }
        let fare: f64 = get(fare_i).parse().unwrap_or(0.0);
        if fare <= 0.0 {
            continue;
        }
        let passengers = passengers_i.and_then(|i| get(i).parse().ok());
        let miles = miles_i.and_then(|i| get(i).parse().ok());
        let carrier = carrier_i.map(|i| get(i)).filter(|s| !s.is_empty());
        rows.push((
            origin, dest, year, quarter, fare, passengers, miles, carrier,
        ));
    }

    let n = db.record_airfare_baselines(&rows)?;
    println!("Ingested {n} airfare baseline rows from {}", path.display());
    Ok(())
}

fn normalize_code(value: &str) -> String {
    // Prefer a 3-letter token if present; otherwise take the first 3 alphanumeric chars.
    for token in value.split(|c: char| !c.is_ascii_alphanumeric()) {
        if token.len() == 3 && token.chars().all(|c| c.is_ascii_alphabetic()) {
            return token.to_ascii_uppercase();
        }
    }
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(3)
        .collect::<String>()
        .to_ascii_uppercase()
}
