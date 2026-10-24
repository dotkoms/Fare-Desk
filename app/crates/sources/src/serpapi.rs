//! SerpAPI Google Flights adapter.
//!
//! Live prices only: Google is queried through SerpAPI, never scraped. The
//! adapter maps SerpAPI's itinerary shape onto the Fare Desk contract and
//! refuses round trips, because a round-trip response carries only the
//! outbound segment detail while pricing the pair — showing that as one
//! itinerary would mislabel what the traveller is buying.

use std::collections::HashMap;

use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use faredesk_core::{Bags, Cabin, FareRules, Offer, Place, Provider, Segment};
use serde::Deserialize;

use crate::{fill_template, ResolvedQuery, SourceError};

pub struct SerpApiSource {
    client: reqwest::Client,
    key: String,
    base_url: String,
    providers: Vec<Provider>,
    places: HashMap<String, Place>,
}

impl SerpApiSource {
    pub fn id(&self) -> &'static str {
        "serpapi"
    }

    pub fn data_card_id(&self) -> &'static str {
        "fares-serpapi"
    }

    pub fn new(
        client: reqwest::Client,
        key: impl Into<String>,
        providers: Vec<Provider>,
        places: HashMap<String, Place>,
    ) -> Self {
        Self {
            client,
            key: key.into(),
            base_url: "https://serpapi.com/search.json".to_string(),
            providers,
            places,
        }
    }

    /// Point the adapter at a different endpoint. Used by tests, local mocks
    /// and self-hosted proxies; production uses the SerpAPI default.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub async fn search(&self, query: &ResolvedQuery<'_>) -> Result<Vec<Offer>, SourceError> {
        self.search_with_market(query, None, None).await
    }

    /// Price the same search in another country storefront (`gl`) and currency.
    pub async fn search_in_market(
        &self,
        query: &ResolvedQuery<'_>,
        market: &str,
        currency: &str,
    ) -> Result<Vec<Offer>, SourceError> {
        self.search_with_market(query, Some(market), Some(currency))
            .await
    }

    async fn search_with_market(
        &self,
        query: &ResolvedQuery<'_>,
        market: Option<&str>,
        currency: Option<&str>,
    ) -> Result<Vec<Offer>, SourceError> {
        if query.request.return_date.is_some() {
            return Err(SourceError::InvalidRequest(
                "the live source prices one direction at a time; search each leg separately".into(),
            ));
        }

        let response = self
            .client
            .get(&self.base_url)
            .query(&self.params(query, market, currency))
            .timeout(std::time::Duration::from_secs(25))
            .send()
            .await
            .map_err(|error| {
                SourceError::Unavailable(format!("serpapi request failed: {error}"))
            })?;

        let status = response.status();
        // A rejected key is a configuration problem, not an outage: say so plainly
        // instead of serving it as a transient failure.
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(SourceError::NotConfigured(
                "SerpAPI rejected the API key; check SERPAPI_KEY".into(),
            ));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let snippet: String = body.chars().take(200).collect();
            return Err(SourceError::Unavailable(format!(
                "serpapi returned {status}: {snippet}"
            )));
        }

        let payload: SerpResponse = response
            .json()
            .await
            .map_err(|error| SourceError::Parse(format!("serpapi response: {error}")))?;

        if let Some(error) = payload.error {
            return Err(SourceError::Unavailable(format!("serpapi error: {error}")));
        }

        let mut offers = Vec::new();
        for itinerary in payload
            .best_flights
            .iter()
            .chain(payload.other_flights.iter())
        {
            if let Some(offer) = self.map_itinerary(itinerary, query)? {
                offers.push(offer);
            }
        }
        Ok(offers)
    }

    fn params(
        &self,
        query: &ResolvedQuery<'_>,
        market: Option<&str>,
        currency: Option<&str>,
    ) -> Vec<(&'static str, String)> {
        let request = query.request;
        vec![
            ("engine", "google_flights".to_string()),
            ("departure_id", query.origin.code.clone()),
            ("arrival_id", query.destination.code.clone()),
            (
                "outbound_date",
                request.depart_date.format("%Y-%m-%d").to_string(),
            ),
            ("type", "2".to_string()),
            ("travel_class", travel_class(request.cabin).to_string()),
            ("adults", request.adults.max(1).to_string()),
            ("children", request.children.to_string()),
            (
                "currency",
                currency.unwrap_or(&request.currency).to_uppercase(),
            ),
            ("hl", "en".to_string()),
            ("gl", market.unwrap_or("de").to_lowercase()),
            ("deep_search", "false".to_string()),
            ("api_key", self.key.clone()),
        ]
    }

    fn map_itinerary(
        &self,
        itinerary: &SerpItinerary,
        query: &ResolvedQuery<'_>,
    ) -> Result<Option<Offer>, SourceError> {
        if itinerary.flights.is_empty() {
            return Ok(None);
        }

        let mut segments = Vec::with_capacity(itinerary.flights.len());
        for flight in &itinerary.flights {
            let Some(from) = self.lookup_place(flight.departure_airport.id.as_str(), query) else {
                tracing::warn!(airport = %flight.departure_airport.id, "serpapi airport is not in the reference set");
                return Ok(None);
            };
            let Some(to) = self.lookup_place(flight.arrival_airport.id.as_str(), query) else {
                tracing::warn!(airport = %flight.arrival_airport.id, "serpapi airport is not in the reference set");
                return Ok(None);
            };

            let depart = local_datetime(&from, &flight.departure_airport.time)?;
            let arrive = local_datetime(&to, &flight.arrival_airport.time)?;

            segments.push(Segment {
                id: format!("{}-{}", flight.flight_number.replace(' ', ""), depart),
                marketing_carrier: flight.airline.clone(),
                operating_carrier: flight
                    .operating_airline
                    .clone()
                    .unwrap_or_else(|| flight.airline.clone()),
                service_number: flight.flight_number.clone(),
                from,
                to,
                depart_terminal: None,
                arrive_terminal: None,
                depart_local: depart,
                arrive_local: arrive,
                duration_minutes: flight.duration,
                aircraft: flight.airplane.clone(),
                cabin: cabin_from(&flight.travel_class),
                fare_class: None,
            });
        }

        let bags = bags_from_extensions(&itinerary.extensions);
        let provider = self
            .providers
            .iter()
            .find(|provider| provider.id == "google-flights");
        let provider_url = provider
            .map(|provider| {
                fill_template(
                    &provider.deep_link_template,
                    query.origin.code.as_str(),
                    query.destination.code.as_str(),
                    query.request.depart_date,
                    query.request.adults,
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "https://www.google.com/travel/flights?q=Flights%20from%20{}%20to%20{}%20on%20{}",
                    query.origin.code,
                    query.destination.code,
                    query.request.depart_date
                )
            });

        let mut warnings = Vec::new();
        if itinerary
            .extensions
            .iter()
            .any(|extension| extension.to_lowercase().contains("bag"))
        {
            warnings.push(
                "Bag terms come from the source's summary; confirm the fee before booking."
                    .to_string(),
            );
        }

        Ok(Some(Offer {
            segments,
            total_duration_minutes: itinerary.total_duration,
            price_per_traveller: itinerary.price,
            currency: query.request.currency.to_uppercase(),
            taxes_and_fees: None,
            bags_included: bags,
            // SerpAPI reports that a bag costs extra but not how much.
            bag_fee_per_checked_bag: None,
            fare_family: None,
            fare_rules: FareRules::default(),
            provider_url,
            provider_name: "Google Flights".to_string(),
            source: self.id().to_string(),
            verified_at: Some(Utc::now()),
            risk: Vec::new(),
            warnings,
        }))
    }
}

impl SerpApiSource {
    fn lookup_place(&self, code: &str, query: &ResolvedQuery<'_>) -> Option<Place> {
        if code.eq_ignore_ascii_case(&query.origin.code) {
            return Some(query.origin.clone());
        }
        if code.eq_ignore_ascii_case(&query.destination.code) {
            return Some(query.destination.clone());
        }
        self.places.get(&code.to_ascii_uppercase()).cloned()
    }
}

fn local_datetime(place: &Place, value: &str) -> Result<String, SourceError> {
    let naive = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M")
        .map_err(|error| SourceError::Parse(format!("serpapi time {value:?}: {error}")))?;
    let tz: Tz = place
        .timezone
        .parse()
        .map_err(|_| SourceError::Parse(format!("unknown timezone {}", place.timezone)))?;
    let local = tz.from_local_datetime(&naive).earliest().ok_or_else(|| {
        SourceError::Parse(format!("{value} does not exist in {}", place.timezone))
    })?;
    Ok(local.to_rfc3339())
}

fn travel_class(cabin: Cabin) -> u8 {
    match cabin {
        Cabin::Economy => 1,
        Cabin::Premium => 2,
        Cabin::Business => 3,
        Cabin::First => 4,
    }
}

fn cabin_from(value: &str) -> Cabin {
    let lower = value.to_lowercase();
    if lower.contains("premium") {
        Cabin::Premium
    } else if lower.contains("business") {
        Cabin::Business
    } else if lower.contains("first") {
        Cabin::First
    } else {
        Cabin::Economy
    }
}

fn bags_from_extensions(extensions: &[String]) -> Bags {
    let mut checked = 0;
    for extension in extensions {
        let lower = extension.to_lowercase();
        if (lower.contains("checked bag") || lower.contains("checked baggage"))
            && lower.contains("included")
        {
            checked |= 1;
        }
    }
    Bags {
        carry_on: 1,
        checked,
    }
}

#[derive(Debug, Deserialize)]
struct SerpResponse {
    #[serde(default)]
    best_flights: Vec<SerpItinerary>,
    #[serde(default)]
    other_flights: Vec<SerpItinerary>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerpItinerary {
    #[serde(default)]
    flights: Vec<SerpFlight>,
    #[serde(default)]
    price: f64,
    #[serde(default)]
    total_duration: Option<u32>,
    #[serde(default)]
    extensions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SerpFlight {
    departure_airport: SerpAirport,
    arrival_airport: SerpAirport,
    #[serde(default)]
    duration: u32,
    #[serde(default)]
    airplane: Option<String>,
    #[serde(default)]
    airline: String,
    #[serde(default)]
    operating_airline: Option<String>,
    #[serde(default)]
    travel_class: String,
    #[serde(default)]
    flight_number: String,
}

#[derive(Debug, Deserialize)]
struct SerpAirport {
    #[serde(default)]
    id: String,
    #[serde(default)]
    time: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use faredesk_core::SearchQuery;

    const RESPONSE: &str = include_str!("../tests/fixtures/serpapi_ber_lis.json");

    fn places() -> (Place, Place) {
        let reference = faredesk_store::Reference::load().unwrap();
        (
            reference.by_code("BER").unwrap().clone(),
            reference.by_code("LIS").unwrap().clone(),
        )
    }

    fn place_map() -> HashMap<String, Place> {
        faredesk_store::Reference::load().unwrap().place_map()
    }

    fn query<'a>(
        request: &'a SearchQuery,
        origin: &'a Place,
        destination: &'a Place,
    ) -> ResolvedQuery<'a> {
        ResolvedQuery {
            request,
            origin,
            destination,
        }
    }

    #[test]
    fn maps_best_and_other_flights() {
        let payload: SerpResponse = serde_json::from_str(RESPONSE).unwrap();
        let (berlin, lisbon) = places();
        let request = SearchQuery {
            origin: "BER".into(),
            destination: "LIS".into(),
            depart_date: NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
            ..Default::default()
        };
        let source = SerpApiSource {
            client: reqwest::Client::new(),
            key: "test".into(),
            base_url: String::new(),
            providers: faredesk_store::load_providers().unwrap(),
            places: place_map(),
        };
        let resolved = query(&request, &berlin, &lisbon);
        let mut count = 0;
        for itinerary in payload
            .best_flights
            .iter()
            .chain(payload.other_flights.iter())
        {
            if let Some(offer) = source.map_itinerary(itinerary, &resolved).unwrap() {
                count += 1;
                assert!(!offer.segments.is_empty());
                assert!(offer.price_per_traveller > 0.0);
                assert!(offer.segments[0].depart_local.ends_with("+01:00"));
                assert!(offer.provider_url.contains("BER"));
            }
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn connection_airports_resolve_from_the_reference_set() {
        let payload: SerpResponse = serde_json::from_str(RESPONSE).unwrap();
        let (berlin, lisbon) = places();
        let request = SearchQuery {
            origin: "BER".into(),
            destination: "LIS".into(),
            depart_date: NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
            ..Default::default()
        };
        let source = SerpApiSource {
            client: reqwest::Client::new(),
            key: "test".into(),
            base_url: String::new(),
            providers: Vec::new(),
            places: place_map(),
        };
        let offer = source
            .map_itinerary(
                &payload.other_flights[1],
                &query(&request, &berlin, &lisbon),
            )
            .unwrap()
            .expect("one-stop offer");
        assert_eq!(offer.segments.len(), 2);
        assert_eq!(offer.segments[1].from.code, "STN");
        assert_eq!(offer.segments[0].marketing_carrier, "Ryanair");
    }

    #[test]
    fn unknown_airports_drop_the_offer_instead_of_inventing_a_place() {
        let mut payload: SerpResponse = serde_json::from_str(RESPONSE).unwrap();
        let (berlin, lisbon) = places();
        let request = SearchQuery {
            origin: "BER".into(),
            destination: "LIS".into(),
            depart_date: NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
            ..Default::default()
        };
        let source = SerpApiSource {
            client: reqwest::Client::new(),
            key: "test".into(),
            base_url: String::new(),
            providers: Vec::new(),
            places: place_map(),
        };
        payload.best_flights[0].flights[0].arrival_airport.id = "ZZZ".into();
        let offer = source
            .map_itinerary(&payload.best_flights[0], &query(&request, &berlin, &lisbon))
            .unwrap();
        assert!(offer.is_none());
    }

    #[test]
    fn baggage_extensions_are_read_conservatively() {
        assert_eq!(bags_from_extensions(&[]).checked, 0);
        assert_eq!(
            bags_from_extensions(&["Checked baggage for a fee".into()]).checked,
            0
        );
        assert_eq!(
            bags_from_extensions(&["1 checked bag included".into()]).checked,
            1
        );
    }

    #[test]
    fn cabins_map_both_ways() {
        assert_eq!(travel_class(Cabin::Economy), 1);
        assert_eq!(cabin_from("Premium economy"), Cabin::Premium);
        assert_eq!(cabin_from("Business"), Cabin::Business);
        assert_eq!(cabin_from("First"), Cabin::First);
    }

    #[test]
    fn round_trips_are_refused_with_a_clear_message() {
        let (berlin, lisbon) = places();
        let request = SearchQuery {
            return_date: Some(NaiveDate::from_ymd_opt(2027, 3, 8).unwrap()),
            ..Default::default()
        };
        let source = SerpApiSource::new(reqwest::Client::new(), "test", Vec::new(), place_map());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let error = runtime
            .block_on(source.search(&query(&request, &berlin, &lisbon)))
            .unwrap_err();
        assert!(error.to_string().contains("one direction"));
    }

    #[tokio::test]
    async fn adapter_calls_the_http_endpoint_with_the_contract_params() {
        use std::collections::HashMap;
        use std::sync::{Arc, Mutex};

        use axum::extract::Query;
        use axum::response::Json;
        use axum::routing::get;
        use axum::Router;

        let captured: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let handler = {
            let captured = captured.clone();
            move |Query(params): Query<HashMap<String, String>>| {
                let captured = captured.clone();
                async move {
                    captured.lock().unwrap().extend(params);
                    Json(serde_json::from_str::<serde_json::Value>(RESPONSE).unwrap())
                }
            }
        };
        let app = Router::new().route("/search.json", get(handler));
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let (berlin, lisbon) = places();
        let request = SearchQuery {
            origin: "BER".into(),
            destination: "LIS".into(),
            depart_date: NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
            adults: 2,
            checked_bags: 1,
            ..Default::default()
        };
        let source = SerpApiSource::new(
            reqwest::Client::new(),
            "test-key",
            faredesk_store::load_providers().unwrap(),
            place_map(),
        )
        .with_base_url(format!("http://{address}/search.json"));

        let offers = source
            .search(&query(&request, &berlin, &lisbon))
            .await
            .unwrap();
        assert_eq!(offers.len(), 3);
        assert!(offers.iter().all(|offer| offer.verified_at.is_some()));
        assert!(offers
            .iter()
            .all(|offer| offer.provider_url.contains("BER")));

        let params = captured.lock().unwrap().clone();
        assert_eq!(params["engine"], "google_flights");
        assert_eq!(params["departure_id"], "BER");
        assert_eq!(params["arrival_id"], "LIS");
        assert_eq!(params["outbound_date"], "2027-03-01");
        assert_eq!(params["type"], "2");
        assert_eq!(params["adults"], "2");
        assert_eq!(params["currency"], "EUR");
        assert_eq!(params["api_key"], "test-key");
        assert!(!params.contains_key("return_date"));
    }

    #[test]
    fn base_url_is_configurable_for_tests() {
        let source = SerpApiSource::new(reqwest::Client::new(), "test", Vec::new(), HashMap::new())
            .with_base_url("http://127.0.0.1:9/search");
        assert_eq!(source.base_url, "http://127.0.0.1:9/search");
    }
}
