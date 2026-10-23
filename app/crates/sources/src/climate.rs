//! Open-Meteo climate normals: one recent calendar year of daily temperatures
//! and precipitation, aggregated into monthly normals.

use faredesk_core::ClimateMonth;
use serde::Deserialize;

use crate::SourceError;

pub struct ClimateClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for ClimateClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ClimateClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .unwrap_or_default(),
            base_url: std::env::var("OPEN_METEO_BASE_URL")
                .unwrap_or_else(|_| "https://archive-api.open-meteo.com/v1/archive".to_string()),
        }
    }

    pub async fn monthly_normals(
        &self,
        latitude: f64,
        longitude: f64,
        year: i32,
    ) -> Result<Vec<ClimateMonth>, SourceError> {
        let start = format!("{year}-01-01");
        let end = format!("{year}-12-31");
        let response = self
            .client
            .get(&self.base_url)
            .query(&[
                ("latitude", format!("{latitude:.4}")),
                ("longitude", format!("{longitude:.4}")),
                ("start_date", start),
                ("end_date", end),
                (
                    "daily",
                    "temperature_2m_max,temperature_2m_min,precipitation_sum".to_string(),
                ),
                ("timezone", "auto".to_string()),
            ])
            .send()
            .await
            .map_err(|error| SourceError::Unavailable(format!("open-meteo: {error}")))?;

        if !response.status().is_success() {
            return Err(SourceError::Unavailable(format!(
                "open-meteo returned {}",
                response.status()
            )));
        }

        let payload: OpenMeteoResponse = response
            .json()
            .await
            .map_err(|error| SourceError::Parse(format!("open-meteo: {error}")))?;

        aggregate(&payload.daily)
    }
}

/// Monthly mean high, mean low and rain days (1 mm or more) from daily series.
pub fn aggregate(daily: &DailySeries) -> Result<Vec<ClimateMonth>, SourceError> {
    let length = daily
        .temperature_2m_max
        .len()
        .min(daily.temperature_2m_min.len())
        .min(daily.precipitation_sum.len())
        .min(daily.time.len());
    if length == 0 {
        return Err(SourceError::Parse(
            "open-meteo returned no daily data".into(),
        ));
    }

    let mut highs = [0.0f64; 13];
    let mut lows = [0.0f64; 13];
    let mut rain = [0u32; 13];
    let mut days = [0u32; 13];

    for index in 0..length {
        let Some(month) = month_of(&daily.time[index]) else {
            continue;
        };
        let high = daily.temperature_2m_max[index];
        let low = daily.temperature_2m_min[index];
        if !high.is_finite() || !low.is_finite() {
            continue;
        }
        highs[month] += high;
        lows[month] += low;
        if daily.precipitation_sum[index].is_finite() && daily.precipitation_sum[index] >= 1.0 {
            rain[month] += 1;
        }
        days[month] += 1;
    }

    let mut months = Vec::new();
    for month in 1..=12u8 {
        let count = days[month as usize];
        if count == 0 {
            continue;
        }
        months.push(ClimateMonth {
            month,
            mean_high: (highs[month as usize] / count as f64 * 10.0).round() / 10.0,
            mean_low: (lows[month as usize] / count as f64 * 10.0).round() / 10.0,
            rain_days: rain[month as usize],
            comfort: 0,
        });
    }
    if months.is_empty() {
        return Err(SourceError::Parse(
            "open-meteo returned no usable days".into(),
        ));
    }
    Ok(months)
}

fn month_of(date: &str) -> Option<usize> {
    let month = date.get(5..7)?.parse::<usize>().ok()?;
    (1..=12).contains(&month).then_some(month)
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    daily: DailySeries,
}

#[derive(Debug, Deserialize)]
pub struct DailySeries {
    #[serde(default)]
    pub time: Vec<String>,
    #[serde(default)]
    pub temperature_2m_max: Vec<f64>,
    #[serde(default)]
    pub temperature_2m_min: Vec<f64>,
    #[serde(default)]
    pub precipitation_sum: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_daily_data_into_months() {
        let raw = r#"{
          "daily": {
            "time": ["2025-01-01","2025-01-02","2025-02-01","2025-02-02"],
            "temperature_2m_max": [5.0, 7.0, 10.0, 12.0],
            "temperature_2m_min": [-1.0, 1.0, 2.0, 4.0],
            "precipitation_sum": [0.0, 2.5, 0.2, 0.0]
          }
        }"#;
        let payload: OpenMeteoResponse = serde_json::from_str(raw).unwrap();
        let months = aggregate(&payload.daily).unwrap();
        assert_eq!(months.len(), 2);
        assert_eq!(months[0].month, 1);
        assert_eq!(months[0].mean_high, 6.0);
        assert_eq!(months[0].mean_low, 0.0);
        assert_eq!(months[0].rain_days, 1);
        assert_eq!(months[1].month, 2);
    }

    #[test]
    fn empty_series_is_an_error() {
        let daily = DailySeries {
            time: vec![],
            temperature_2m_max: vec![],
            temperature_2m_min: vec![],
            precipitation_sum: vec![],
        };
        assert!(aggregate(&daily).is_err());
    }

    #[test]
    fn month_parsing_is_defensive() {
        assert_eq!(month_of("2025-03-14"), Some(3));
        assert_eq!(month_of("nope"), None);
        assert_eq!(month_of("2025-13-01"), None);
    }
}
