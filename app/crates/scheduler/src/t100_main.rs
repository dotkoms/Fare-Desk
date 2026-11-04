//! Ingest BTS T-100 Domestic Segment data (CSV inside a monthly zip or plain CSV).
//!
//! Usage:
//!   faredesk-t100-ingest --file T_T100D_SEGMENT_ALL_CARRIER_2025_12.zip
//!   faredesk-t100-ingest --file segment.csv
//!
//! Aggregates origin-destination pairs by year/month and stores passenger counts.
//! Public domain. Useful as a real route list for Explore when own history is thin.

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use faredesk_store::Db;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("csv: {0}")]
    Csv(#[from] csv::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
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

    let rows = if path.extension().map(|e| e == "zip").unwrap_or(false) {
        aggregate_zip(&path)?
    } else {
        aggregate_csv(File::open(&path)?)?
    };

    let n = db.record_t100_routes(&rows)?;
    println!("Ingested {n} T-100 route rows from {}", path.display());
    Ok(())
}

fn aggregate_zip(
    path: &std::path::Path,
) -> Result<Vec<(String, String, i32, i32, i64, Option<i64>)>, Error> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let name = (0..archive.len())
        .find_map(|i| {
            let entry = archive.by_index(i).ok()?;
            let n = entry.name().to_string();
            n.to_ascii_lowercase().ends_with(".csv").then_some(n)
        })
        .ok_or(Error::Other("zip contains no CSV".into()))?;
    let entry = archive.by_name(&name)?;
    aggregate_csv(entry)
}

fn aggregate_csv<R: Read>(
    reader: R,
) -> Result<Vec<(String, String, i32, i32, i64, Option<i64>)>, Error> {
    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(reader);
    let headers = rdr.headers()?.clone();
    let idx = |name: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(name));

    let origin_i = idx("ORIGIN")
        .or_else(|| idx("origin"))
        .ok_or_else(|| Error::Other("missing ORIGIN".into()))?;
    let dest_i = idx("DEST")
        .or_else(|| idx("dest"))
        .ok_or_else(|| Error::Other("missing DEST".into()))?;
    let year_i = idx("YEAR")
        .or_else(|| idx("year"))
        .ok_or_else(|| Error::Other("missing YEAR".into()))?;
    let month_i = idx("MONTH")
        .or_else(|| idx("month"))
        .ok_or_else(|| Error::Other("missing MONTH".into()))?;
    let passengers_i = idx("PASSENGERS").or_else(|| idx("passengers"));
    let flights_i = idx("DEPARTURES_PERFORMED").or_else(|| idx("departures_performed"));

    use std::collections::HashMap;
    let mut agg: HashMap<(String, String, i32, i32), (i64, i64)> = HashMap::new();

    for record in rdr.records() {
        let record = record?;
        let get = |i: usize| record.get(i).unwrap_or("").trim().to_string();
        let origin = get(origin_i).to_ascii_uppercase();
        let dest = get(dest_i).to_ascii_uppercase();
        if origin.len() != 3 || dest.len() != 3 {
            continue;
        }
        let year: i32 = get(year_i).parse().unwrap_or(0);
        let month: i32 = get(month_i).parse().unwrap_or(0);
        if year == 0 || !(1..=12).contains(&month) {
            continue;
        }
        let passengers: i64 = passengers_i.and_then(|i| get(i).parse().ok()).unwrap_or(0);
        let flights: i64 = flights_i.and_then(|i| get(i).parse().ok()).unwrap_or(0);
        let entry = agg.entry((origin, dest, year, month)).or_insert((0, 0));
        entry.0 += passengers;
        entry.1 += flights;
    }

    Ok(agg
        .into_iter()
        .map(|((o, d, y, m), (p, f))| (o, d, y, m, p, Some(f)))
        .collect())
}
