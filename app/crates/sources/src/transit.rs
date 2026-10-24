//! Transitous (MOTIS) public-transit routing for airport access.

use chrono::{DateTime, Utc};
use faredesk_core::TransitLeg;
use serde::Deserialize;

use crate::SourceError;

pub struct TransitClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for TransitClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TransitClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("FareDesk/0.1 (+https://github.com/dotkoms/Fare-Desk)")
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .unwrap_or_default(),
            base_url: std::env::var("TRANSITOUS_BASE_URL")
                .unwrap_or_else(|_| "https://api.transitous.org/api/v1/plan".to_string()),
        }
    }

    /// Fastest public-transit itinerary between two coordinates at `depart_at`.
    pub async fn fastest_leg(
        &self,
        from: (f64, f64),
        to: (f64, f64),
        depart_at: DateTime<Utc>,
    ) -> Result<TransitLeg, SourceError> {
        let response = self
            .client
            .get(&self.base_url)
            .query(&[
                ("fromPlace", format!("{:.5},{:.5}", from.0, from.1)),
                ("toPlace", format!("{:.5},{:.5}", to.0, to.1)),
                ("time", depart_at.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                ("numItineraries", "1".to_string()),
            ])
            .send()
            .await
            .map_err(|error| SourceError::Unavailable(format!("transitous: {error}")))?;

        if !response.status().is_success() {
            return Err(SourceError::Unavailable(format!(
                "transitous returned {}",
                response.status()
            )));
        }
        let payload: TransitResponse = response
            .json()
            .await
            .map_err(|error| SourceError::Parse(format!("transitous: {error}")))?;
        let itinerary = payload
            .itineraries
            .first()
            // Transitous answers but has no route: its feeds do not cover this area.
            .ok_or_else(|| {
                SourceError::Unavailable("Transitous has no public-transit route here".into())
            })?;
        Ok(TransitLeg {
            minutes: ((itinerary.duration / 60.0).ceil() as u32).max(1),
            transfers: itinerary.transfers,
            source: "Transitous".to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct TransitResponse {
    #[serde(default)]
    itineraries: Vec<TransitItinerary>,
}

#[derive(Debug, Deserialize)]
struct TransitItinerary {
    #[serde(default)]
    duration: f64,
    #[serde(default)]
    transfers: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_transitous_itinerary() {
        let raw = r#"{"itineraries":[{"duration":2880,"transfers":2}]}"#;
        let payload: TransitResponse = serde_json::from_str(raw).unwrap();
        let itinerary = payload.itineraries.first().unwrap();
        assert_eq!((itinerary.duration / 60.0).ceil() as u32, 48);
        assert_eq!(itinerary.transfers, 2);
    }
}
