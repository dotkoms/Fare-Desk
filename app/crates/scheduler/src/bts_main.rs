use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrono::{Datelike, Duration, Utc};
use faredesk_scheduler::bts::{download_month, ingest_zip};
use faredesk_store::Db;

struct Args {
    year: i32,
    month: u32,
    file: Option<PathBuf>,
    database: String,
}

fn parse_args() -> Args {
    let previous = Utc::now().date_naive() - Duration::days(31);
    let mut args = Args {
        year: previous.year(),
        month: previous.month(),
        file: None,
        database: std::env::var("DATABASE_PATH").unwrap_or_else(|_| "faredesk.sqlite".into()),
    };
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--year" => {
                args.year = iter
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(args.year)
            }
            "--month" => {
                args.month = iter
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(args.month)
            }
            "--file" => args.file = iter.next().map(PathBuf::from),
            "--database" => {
                if let Some(value) = iter.next() {
                    args.database = value;
                }
            }
            other => eprintln!("ignoring unknown argument {other:?}"),
        }
    }
    args
}

#[tokio::main]
async fn main() -> ExitCode {
    let _ = dotenvy::dotenv();
    let args = parse_args();
    if !(1..=12).contains(&args.month) {
        eprintln!("month must be 1-12");
        return ExitCode::from(2);
    }

    let db = match Db::open(Path::new(&args.database)) {
        Ok(db) => db,
        Err(error) => {
            eprintln!("could not open {}: {error}", args.database);
            return ExitCode::from(1);
        }
    };

    let path = match args.file {
        Some(path) => path,
        None => match download_with_retries(args.year, args.month).await {
            Ok(path) => path,
            Err(error) => {
                eprintln!(
                    "could not download BTS {}-{}: {error}",
                    args.year, args.month
                );
                return ExitCode::from(1);
            }
        },
    };

    match ingest_zip(&path, &db) {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("report serialises")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("ingest failed for {}: {error}", path.display());
            ExitCode::from(1)
        }
    }
}

/// The monthly file is ~30 MB from a government host that drops connections now
/// and then; three tries with a pause beats failing the month.
async fn download_with_retries(
    year: i32,
    month: u32,
) -> Result<std::path::PathBuf, faredesk_scheduler::bts::BtsError> {
    let mut attempt = 1;
    loop {
        match download_month(year, month).await {
            Ok(path) => return Ok(path),
            Err(error) if attempt < 3 => {
                eprintln!("BTS {year}-{month}: attempt {attempt} failed ({error}); retrying");
                tokio::time::sleep(std::time::Duration::from_secs(10 * attempt)).await;
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    }
}
