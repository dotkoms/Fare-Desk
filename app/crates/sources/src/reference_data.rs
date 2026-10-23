//! Free reference-data clients: ECB rates via Frankfurter and public holidays
//! via Nager.Date. Both are cached by the API for a day.

use chrono::NaiveDate;
use faredesk_core::{FxRate, Holiday};
use serde::Deserialize;

use crate::SourceError;

pub struct ReferenceClient {
    client: reqwest::Client,
}

impl Default for ReferenceClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ReferenceClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// GET with one retry on a transport error or a 5xx. Reference data is
    /// free to retry; paid fare sources are not.
    async fn get(
        &self,
        url: &str,
        query: &[(&str, &str)],
    ) -> Result<reqwest::Response, SourceError> {
        let attempt = |query: &[(&str, &str)]| {
            let client = self.client.clone();
            let url = url.to_string();
            let query: Vec<(String, String)> = query
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect();
            async move {
                client
                    .get(url)
                    .query(&query)
                    .send()
                    .await
                    .map_err(|error| SourceError::Unavailable(error.to_string()))
            }
        };

        match attempt(query).await {
            Ok(response) if response.status().is_server_error() => {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                attempt(query).await
            }
            Ok(response) => Ok(response),
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                attempt(query).await
            }
        }
    }

    pub async fn fx_rate(&self, base: &str, quote: &str) -> Result<FxRate, SourceError> {
        let base = base.to_uppercase();
        let quote = quote.to_uppercase();
        if base == quote {
            return Ok(FxRate {
                date: chrono::Utc::now()
                    .date_naive()
                    .format("%Y-%m-%d")
                    .to_string(),
                base,
                quote,
                rate: 1.0,
                source: "identity".to_string(),
            });
        }

        let response = self
            .get(
                "https://api.frankfurter.app/latest",
                &[("from", base.as_str()), ("to", quote.as_str())],
            )
            .await
            .map_err(|error| SourceError::Unavailable(format!("frankfurter: {error}")))?;

        if !response.status().is_success() {
            return Err(SourceError::Unavailable(format!(
                "frankfurter returned {}",
                response.status()
            )));
        }

        let payload: FrankfurterResponse = response
            .json()
            .await
            .map_err(|error| SourceError::Parse(format!("frankfurter: {error}")))?;

        let rate = payload
            .rates
            .get(&quote)
            .copied()
            .ok_or_else(|| SourceError::Parse(format!("frankfurter did not return {quote}")))?;

        Ok(FxRate {
            date: payload.date,
            base: payload.base,
            quote,
            rate,
            source: "ECB".to_string(),
        })
    }

    pub async fn holidays(&self, country: &str, year: i32) -> Result<Vec<Holiday>, SourceError> {
        let country = country.to_uppercase();
        let url = format!("https://date.nager.at/api/v3/PublicHolidays/{year}/{country}");
        let response = self
            .get(&url, &[])
            .await
            .map_err(|error| SourceError::Unavailable(format!("nager.date: {error}")))?;

        if !response.status().is_success() {
            return Err(SourceError::Unavailable(format!(
                "nager.date returned {}",
                response.status()
            )));
        }

        let payload: Vec<NagerHoliday> = response
            .json()
            .await
            .map_err(|error| SourceError::Parse(format!("nager.date: {error}")))?;

        Ok(payload
            .into_iter()
            .filter_map(|holiday| {
                let date = NaiveDate::parse_from_str(&holiday.date, "%Y-%m-%d").ok()?;
                Some(Holiday {
                    date,
                    name: holiday.name,
                    local_name: Some(holiday.local_name),
                    country_code: holiday.country_code,
                    global: holiday.global,
                })
            })
            .collect())
    }
}

/// Bundled EUR snapshot used when Frankfurter is unreachable. Rates are
/// approximate and the response is marked degraded so the UI can say so.
const FALLBACK_PER_EUR: &[(&str, f64)] = &[
    ("EUR", 1.0),
    ("USD", 1.16),
    ("GBP", 0.86),
    ("CHF", 0.94),
    ("SEK", 11.0),
    ("NOK", 11.7),
    ("DKK", 7.46),
    ("PLN", 4.28),
    ("CZK", 25.2),
    ("HUF", 390.0),
    ("BRL", 5.9),
    ("JPY", 168.0),
    ("AUD", 1.64),
    ("CAD", 1.47),
    ("SGD", 1.51),
    ("HKD", 9.05),
    ("TRY", 24.0),
    ("ZAR", 20.5),
];

pub fn fallback_fx(base: &str, quote: &str) -> Option<FxRate> {
    let base = base.to_uppercase();
    let quote = quote.to_uppercase();
    let base_rate = FALLBACK_PER_EUR
        .iter()
        .find(|(code, _)| *code == base)
        .map(|(_, rate)| *rate)?;
    let quote_rate = FALLBACK_PER_EUR
        .iter()
        .find(|(code, _)| *code == quote)
        .map(|(_, rate)| *rate)?;
    Some(FxRate {
        date: chrono::Utc::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
        rate: (quote_rate / base_rate * 10_000.0).round() / 10_000.0,
        base,
        quote,
        source: "bundled-snapshot".to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    date: String,
    base: String,
    rates: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct NagerHoliday {
    date: String,
    #[serde(default)]
    name: String,
    #[serde(default, rename = "localName")]
    local_name: String,
    #[serde(default, rename = "countryCode")]
    country_code: String,
    #[serde(default)]
    global: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_uses_cross_rates() {
        let eur_usd = fallback_fx("EUR", "USD").unwrap();
        assert!((eur_usd.rate - 1.16).abs() < 0.001);
        let usd_eur = fallback_fx("usd", "EUR").unwrap();
        assert!((usd_eur.rate - 1.0 / 1.16).abs() < 0.001);
        assert_eq!(usd_eur.source, "bundled-snapshot");
    }

    #[test]
    fn unknown_currency_has_no_fallback() {
        assert!(fallback_fx("EUR", "XYZ").is_none());
    }

    #[test]
    fn identity_is_one() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let rate = runtime
            .block_on(ReferenceClient::new().fx_rate("EUR", "EUR"))
            .unwrap();
        assert_eq!(rate.rate, 1.0);
        assert_eq!(rate.source, "identity");
    }

    #[test]
    fn holiday_payload_shape_parses() {
        let raw = r#"[
            {"date":"2027-01-01","localName":"Neujahr","name":"New Year's Day","countryCode":"DE","global":true,"counties":null,"types":["Public"]},
            {"date":"2027-10-03","localName":"Tag der Deutschen Einheit","name":"German Unity Day","countryCode":"DE","global":true}
        ]"#;
        let parsed: Vec<NagerHoliday> = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].country_code, "DE");
        assert!(parsed[0].global);
    }
}
