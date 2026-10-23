//! Curated city-centre access and minimum connection times.

use std::collections::HashMap;
use std::sync::OnceLock;

use faredesk_core::{CityCenter, Mct, Transfer};
use serde::Deserialize;

const ACCESS_JSON: &str = include_str!("../../../data/access.json");

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirportAccess {
    #[serde(default)]
    pub city_center: Option<CityCenter>,
    #[serde(default)]
    pub transfers: Vec<Transfer>,
    #[serde(default)]
    pub mct: Option<Mct>,
}

#[derive(Debug, Deserialize)]
struct AccessFile {
    #[serde(default)]
    #[allow(dead_code)]
    note: String,
    airports: HashMap<String, AirportAccess>,
}

static ACCESS: OnceLock<HashMap<String, AirportAccess>> = OnceLock::new();

fn load() -> &'static HashMap<String, AirportAccess> {
    ACCESS.get_or_init(|| {
        let file: AccessFile =
            serde_json::from_str(ACCESS_JSON).expect("embedded access data must be valid JSON");
        file.airports
            .into_iter()
            .map(|(code, access)| (code.to_uppercase(), access))
            .collect()
    })
}

pub fn access_for(code: &str) -> Option<AirportAccess> {
    load().get(&code.to_uppercase()).cloned()
}

pub fn access_note() -> &'static str {
    "Curated estimates: city-centre transfers in typical conditions and airport guidance for minimum connection times. Verify with the operator."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn berlin_has_access_data() {
        let access = access_for("ber").expect("BER");
        assert!(access.transfers.len() >= 2);
        assert_eq!(access.mct.unwrap().international, 45);
        let center = access.city_center.unwrap();
        assert!((center.lat - 52.52).abs() < 0.01);
    }

    #[test]
    fn unknown_airports_return_none() {
        assert!(access_for("ZZZ").is_none());
    }
}
