use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use faredesk_core::Place;

const AIRPORTS_JSON: &str = include_str!("../../../data/airports.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AirportFile {
    generated_at: DateTime<Utc>,
    sources: Vec<AirportSource>,
    places: Vec<Place>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AirportSource {
    name: String,
    licence: String,
    fields: String,
}

#[derive(Debug)]
pub struct Reference {
    places: Vec<Place>,
    by_code: HashMap<String, Place>,
    generated_at: DateTime<Utc>,
    sources: Vec<AirportSource>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReferenceError {
    #[error("airport dataset is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("airport dataset is empty")]
    Empty,
}

impl Reference {
    pub fn load() -> Result<Self, ReferenceError> {
        let file: AirportFile = serde_json::from_str(AIRPORTS_JSON)?;
        if file.places.is_empty() {
            return Err(ReferenceError::Empty);
        }
        let by_code = file
            .places
            .iter()
            .map(|place| (place.code.clone(), place.clone()))
            .collect();
        Ok(Self {
            places: file.places,
            by_code,
            generated_at: file.generated_at,
            sources: file.sources,
        })
    }

    pub fn len(&self) -> usize {
        self.places.len()
    }

    pub fn is_empty(&self) -> bool {
        self.places.is_empty()
    }

    pub fn generated_at(&self) -> DateTime<Utc> {
        self.generated_at
    }

    pub fn source_summary(&self) -> String {
        self.sources
            .iter()
            .map(|source| format!("{} ({}, {})", source.name, source.fields, source.licence))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// A snapshot of every airport keyed by IATA code. Used by adapters that
    /// must name connection airports the request did not mention.
    pub fn place_map(&self) -> HashMap<String, Place> {
        self.by_code.clone()
    }

    pub fn by_code(&self, code: &str) -> Option<&Place> {
        self.by_code.get(&code.to_ascii_uppercase())
    }

    /// Resolve a query that may be an IATA code or a place id.
    pub fn resolve(&self, code_or_id: &str) -> Option<&Place> {
        let trimmed = code_or_id.trim();
        if let Some(code) = trimmed.strip_prefix("airport:") {
            return self.by_code(code);
        }
        self.by_code(trimmed)
    }

    /// Airports in the same metro area as `code`, including the airport itself.
    pub fn metro_airports(&self, code: &str) -> Vec<&Place> {
        let Some(place) = self.by_code(code) else {
            return Vec::new();
        };
        match &place.metro {
            Some(metro) => self
                .places
                .iter()
                .filter(|candidate| candidate.metro.as_deref() == Some(metro.as_str()))
                .collect(),
            None => vec![place],
        }
    }

    /// Search by IATA code, city, airport name or country. Code and city
    /// prefix matches rank above name and country matches.
    pub fn search(&self, query: &str, limit: usize) -> Vec<Place> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<(u8, &Place)> = self
            .places
            .iter()
            .filter_map(|place| {
                let code = place.code.to_lowercase();
                let city = place.city.to_lowercase();
                let name = place.name.to_lowercase();
                let country = place.country_name.to_lowercase();
                let rank = if code == needle {
                    0
                } else if code.starts_with(&needle) {
                    1
                } else if city == needle {
                    2
                } else if city.starts_with(&needle) {
                    3
                } else if city.contains(&needle) {
                    4
                } else if name.contains(&needle) {
                    5
                } else if country.contains(&needle) {
                    6
                } else {
                    return None;
                };
                Some((rank, place))
            })
            .collect();
        scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.code.cmp(&b.1.code)));
        scored
            .into_iter()
            .take(limit)
            .map(|(_, place)| place.clone())
            .collect()
    }
}

impl Reference {
    /// The fixture network the demo app ships with.
    pub const FIXTURE_ROUTES: &'static [(&'static str, &'static str)] = &[
        ("BER", "LIS"),
        ("BER", "LHR"),
        ("BER", "BCN"),
        ("BER", "JFK"),
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_loads_with_fixture_airports() {
        let reference = Reference::load().expect("dataset");
        assert!(reference.len() > 2_000, "{}", reference.len());
        for code in ["BER", "LIS", "LHR", "BCN", "JFK", "IST", "GRU"] {
            let place = reference.by_code(code).unwrap_or_else(|| panic!("{code}"));
            assert!(!place.timezone.is_empty());
            assert_eq!(place.id, format!("airport:{code}"));
        }
    }

    #[test]
    fn resolves_ids_and_codes() {
        let reference = Reference::load().unwrap();
        assert_eq!(reference.resolve("airport:BER").unwrap().code, "BER");
        assert_eq!(reference.resolve("ber").unwrap().code, "BER");
        assert!(reference.resolve("NOPE").is_none());
    }

    #[test]
    fn search_ranks_exact_code_first() {
        let reference = Reference::load().unwrap();
        let results = reference.search("lis", 5);
        assert_eq!(results[0].code, "LIS");
        let london = reference.search("london", 10);
        assert!(london.iter().any(|place| place.code == "LHR"));
        assert!(london.iter().any(|place| place.code == "STN"));
    }

    #[test]
    fn metro_expands_to_siblings() {
        let reference = Reference::load().unwrap();
        let london = reference.metro_airports("LHR");
        assert!(london.len() >= 4, "{}", london.len());
        assert!(london
            .iter()
            .all(|place| place.metro.as_deref() == Some("LON")));
    }
}
