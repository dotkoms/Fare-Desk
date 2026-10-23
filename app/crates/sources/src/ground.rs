//! OSRM driving legs for airport access. OSRM is a routing engine over
//! OpenStreetMap; the public demo server is rate-limited, so callers cache.

use faredesk_core::GroundLeg;
use serde::Deserialize;

use crate::SourceError;

pub struct GroundClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for GroundClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GroundClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            base_url: std::env::var("OSRM_BASE_URL")
                .unwrap_or_else(|_| "https://router.project-osrm.org/route/v1/driving".to_string()),
        }
    }

    /// Driving time and distance from one coordinate to another.
    pub async fn drive(&self, from: (f64, f64), to: (f64, f64)) -> Result<GroundLeg, SourceError> {
        let url = format!(
            "{}/{:.5},{:.5};{:.5},{:.5}?overview=false",
            self.base_url, from.1, from.0, to.1, to.0
        );
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| SourceError::Unavailable(format!("osrm: {error}")))?;
        if !response.status().is_success() {
            return Err(SourceError::Unavailable(format!(
                "osrm returned {}",
                response.status()
            )));
        }
        let payload: OsrmResponse = response
            .json()
            .await
            .map_err(|error| SourceError::Parse(format!("osrm: {error}")))?;
        let route = payload
            .routes
            .first()
            .ok_or_else(|| SourceError::Parse("osrm returned no route".into()))?;
        Ok(GroundLeg {
            minutes: (route.duration / 60.0).ceil().max(1.0) as u32,
            km: (route.distance / 100.0).round() / 10.0,
            source: "OSRM".to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct OsrmResponse {
    #[serde(default)]
    routes: Vec<OsrmRoute>,
}

#[derive(Debug, Deserialize)]
struct OsrmRoute {
    #[serde(default)]
    duration: f64,
    #[serde(default)]
    distance: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_an_osrm_route() {
        let raw = r#"{"code":"Ok","routes":[{"duration":1860.4,"distance":25431.2}]}"#;
        let payload: OsrmResponse = serde_json::from_str(raw).unwrap();
        let route = payload.routes.first().unwrap();
        assert_eq!((route.duration / 60.0).ceil() as u32, 32);
        assert_eq!((route.distance / 100.0).round() / 10.0, 25.4);
    }
}
