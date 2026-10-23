use std::process::ExitCode;

use chrono::Utc;
use faredesk_core::parse_quiet_hours;
use faredesk_scheduler::alerts::evaluate;
use faredesk_store::Db;

fn main() -> ExitCode {
    let _ = dotenvy::dotenv();

    let mut database = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "faredesk.sqlite".into());
    let mut digest = false;
    let mut quiet = std::env::var("ALERT_QUIET_HOURS").unwrap_or_else(|_| "23:00-07:00".into());

    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--database" => {
                if let Some(value) = iter.next() {
                    database = value;
                }
            }
            "--digest" => digest = true,
            "--quiet-hours" => {
                if let Some(value) = iter.next() {
                    quiet = value;
                }
            }
            other => eprintln!("ignoring unknown argument {other:?}"),
        }
    }

    let Some(quiet_hours) = parse_quiet_hours(&quiet) else {
        eprintln!("quiet hours must look like 23:00-07:00");
        return ExitCode::from(2);
    };

    let db = match Db::open(&database) {
        Ok(db) => db,
        Err(error) => {
            eprintln!("could not open {database}: {error}");
            return ExitCode::from(1);
        }
    };

    let report = match evaluate(&db, Utc::now(), quiet_hours) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("alert evaluation failed: {error}");
            return ExitCode::from(1);
        }
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("report serialises")
    );

    if digest {
        let alerts = match db.undelivered_alerts(500) {
            Ok(alerts) => alerts,
            Err(error) => {
                eprintln!("could not read the digest: {error}");
                return ExitCode::from(1);
            }
        };
        // The digest is an outbox: a mail sender consumes this output. No SMTP
        // client is bundled; delivery is a deployment concern.
        let grouped: Vec<_> = alerts
            .iter()
            .map(|alert| {
                serde_json::json!({
                    "email": alert.email,
                    "id": alert.id,
                    "route": alert.route,
                    "travelDate": alert.travel_date.format("%Y-%m-%d").to_string(),
                    "price": alert.price,
                    "kind": alert.kind,
                    "createdAt": alert.created_at,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&grouped).expect("digest serialises")
        );
        if !alerts.is_empty() {
            let ids: Vec<i64> = alerts.iter().map(|alert| alert.id).collect();
            if let Err(error) = db.mark_alerts_delivered(&ids, &Utc::now().to_rfc3339()) {
                eprintln!("could not mark the digest delivered: {error}");
                return ExitCode::from(1);
            }
        }
    }

    ExitCode::SUCCESS
}
