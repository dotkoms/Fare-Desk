//! Kiwi.com fare search through Kiwi's official public MCP server.
//!
//! No key, no signup: the server is published by Kiwi.com for agents and takes a
//! plain JSON-RPC `tools/call` over HTTP. That is what lets Fare Desk show live
//! fares on first run. Prices come back for the whole party including the checked
//! and cabin bags requested, and every itinerary carries a Kiwi.com booking link.
//!
//! Kiwi does not mark which itineraries are self-transfers, so the adapter asks
//! for protected connections only rather than show a risk it cannot label.

use std::collections::HashMap;

use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use faredesk_core::{Bags, Cabin, FareRules, Offer, Place, Segment};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{mcp, ResolvedQuery, SourceError};

pub const DEFAULT_URL: &str = "https://mcp.kiwi.com/";

pub struct KiwiSource {
    client: reqwest::Client,
    url: String,
    places: HashMap<String, Place>,
}

impl KiwiSource {
    pub fn id(&self) -> &'static str {
        "kiwi"
    }

    pub fn data_card_id(&self) -> &'static str {
        "fares-kiwi"
    }

    pub fn new(client: reqwest::Client, places: HashMap<String, Place>) -> Self {
        Self {
            client,
            url: DEFAULT_URL.to_string(),
            places,
        }
    }

    /// Point the adapter at another endpoint. Used by tests and local mocks.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub async fn search(&self, query: &ResolvedQuery<'_>) -> Result<Vec<Offer>, SourceError> {
        let structured = mcp::call_tool(
            &self.client,
            &self.url,
            "kiwi",
            "search-flight",
            arguments(query),
            60,
        )
        .await?;
        let result: KiwiResult = serde_json::from_value(structured)
            .map_err(|error| SourceError::Parse(format!("kiwi itineraries: {error}")))?;
        if let Some(error) = result.error.filter(|error| !error.is_empty()) {
            return Err(SourceError::InvalidRequest(format!("kiwi: {error}")));
        }
        if !result
            .currency
            .eq_ignore_ascii_case(&query.request.currency)
        {
            return Err(SourceError::Parse(format!(
                "kiwi priced in {} instead of {}",
                result.currency, query.request.currency
            )));
        }

        let travellers = f64::from(query.request.travellers());
        Ok(result
            .itineraries
            .iter()
            .filter_map(|itinerary| self.map_itinerary(itinerary, query, travellers))
            .collect())
    }

    fn map_itinerary(
        &self,
        itinerary: &KiwiItinerary,
        query: &ResolvedQuery<'_>,
        travellers: f64,
    ) -> Option<Offer> {
        let leg = &itinerary.outbound;
        if leg.segments.is_empty() || itinerary.price <= 0.0 {
            return None;
        }
        let mut segments = Vec::with_capacity(leg.segments.len());
        for flight in &leg.segments {
            let (Some(from), Some(to)) = (
                self.lookup_place(&flight.from, query),
                self.lookup_place(&flight.to, query),
            ) else {
                tracing::warn!(from = %flight.from, to = %flight.to, "kiwi airport is not in the reference set");
                return None;
            };
            let depart = local_rfc3339(&from, &flight.departure_time).ok()?;
            let arrive = local_rfc3339(&to, &flight.arrival_time).ok()?;
            segments.push(Segment {
                id: format!("{}-{}", flight.flight_number, depart),
                marketing_carrier: flight.carrier_name.clone(),
                operating_carrier: flight.carrier_name.clone(),
                service_number: spaced_flight_number(&flight.carrier, &flight.flight_number),
                from,
                to,
                depart_terminal: None,
                arrive_terminal: None,
                depart_local: depart,
                arrive_local: arrive,
                duration_minutes: flight.duration_seconds / 60,
                aircraft: None,
                cabin: cabin_from(&flight.cabin_class),
                fare_class: None,
            });
        }

        let request = query.request;
        Some(Offer {
            segments,
            total_duration_minutes: Some(leg.duration_seconds / 60),
            // Kiwi prices the whole party with the requested bags already in.
            price_per_traveller: itinerary.price / travellers,
            currency: request.currency.to_uppercase(),
            taxes_and_fees: None,
            bags_included: Bags {
                carry_on: request.carry_on_bags.max(itinerary.baggage.cabin_bag),
                checked: request.checked_bags.max(itinerary.baggage.checked_bag),
            },
            bag_fee_per_checked_bag: None,
            fare_family: None,
            // Kiwi reports no fare rules; the all-false default means "unknown".
            fare_rules: FareRules::default(),
            provider_url: itinerary.booking_url.clone(),
            provider_name: "Kiwi.com".to_string(),
            source: self.id().to_string(),
            verified_at: Some(Utc::now()),
            risk: Vec::new(),
            warnings: vec![
                "Sold by Kiwi.com; fare rules and the final price are confirmed there.".to_string(),
            ],
        })
    }

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

/// The `search-flight` arguments for one direction of a query.
fn arguments(query: &ResolvedQuery<'_>) -> Value {
    let request = query.request;
    let adults = usize::from(request.adults.max(1));
    let children = usize::from(request.children);
    let mut arguments = json!({
        "flyFrom": query.origin.code,
        "flyTo": query.destination.code,
        "departureDate": request.depart_date.format("%d/%m/%Y").to_string(),
        "adults": adults,
        "children": children,
        "currency": request.currency.to_uppercase(),
        "locale": "en",
        "sort": "price",
        "cabinClass": cabin_code(request.cabin),
        "adults_hold_bags": vec![request.checked_bags.min(2); adults],
        "adults_hand_bags": vec![request.carry_on_bags.min(1); adults],
        // Protected connections only: Kiwi does not flag self-transfers per result.
        "allow_self_transfer": false,
        "allow_diff_airport_connection": false,
    });
    if children > 0 {
        arguments["children_hold_bags"] = json!(vec![request.checked_bags.min(2); children]);
        arguments["children_hand_bags"] = json!(vec![request.carry_on_bags.min(1); children]);
    }
    if let Some(max) = request.max_stops {
        arguments["max_sector_stopovers"] = json!(max);
    }
    arguments
}

/// Parse a whole response body into Kiwi's result (tests feed recorded bodies).
#[cfg(test)]
fn parse_rpc(body: &str) -> Result<KiwiResult, SourceError> {
    let structured = mcp::structured_result(body, "kiwi")?;
    serde_json::from_value(structured)
        .map_err(|error| SourceError::Parse(format!("kiwi itineraries: {error}")))
}

fn local_rfc3339(place: &Place, value: &str) -> Result<String, SourceError> {
    let naive = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
        .map_err(|error| SourceError::Parse(format!("kiwi time {value:?}: {error}")))?;
    let tz: Tz = place
        .timezone
        .parse()
        .map_err(|_| SourceError::Parse(format!("unknown timezone {}", place.timezone)))?;
    let local = tz.from_local_datetime(&naive).earliest().ok_or_else(|| {
        SourceError::Parse(format!("{value} does not exist in {}", place.timezone))
    })?;
    Ok(local.to_rfc3339())
}

/// "B6" + "B61301" → "B6 1301".
fn spaced_flight_number(carrier: &str, number: &str) -> String {
    match number.strip_prefix(carrier) {
        Some(rest) if !rest.is_empty() => format!("{carrier} {rest}"),
        _ => number.to_string(),
    }
}

fn cabin_code(cabin: Cabin) -> &'static str {
    match cabin {
        Cabin::Economy => "M",
        Cabin::Premium => "W",
        Cabin::Business => "C",
        Cabin::First => "F",
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KiwiResult {
    #[serde(default)]
    currency: String,
    #[serde(default)]
    itineraries: Vec<KiwiItinerary>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KiwiItinerary {
    price: f64,
    #[serde(default)]
    booking_url: String,
    #[serde(default)]
    baggage: KiwiBaggage,
    outbound: KiwiLeg,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KiwiBaggage {
    #[serde(default)]
    cabin_bag: u8,
    #[serde(default)]
    checked_bag: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KiwiLeg {
    #[serde(default)]
    duration_seconds: u32,
    #[serde(default)]
    segments: Vec<KiwiSegment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KiwiSegment {
    from: String,
    to: String,
    departure_time: String,
    arrival_time: String,
    #[serde(default)]
    duration_seconds: u32,
    #[serde(default)]
    carrier: String,
    #[serde(default)]
    carrier_name: String,
    #[serde(default)]
    flight_number: String,
    #[serde(default)]
    cabin_class: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use faredesk_core::SearchQuery;

    fn places() -> HashMap<String, Place> {
        let reference = faredesk_store::Reference::load().unwrap();
        ["JFK", "LAX", "FLL"]
            .iter()
            .map(|code| (code.to_string(), reference.by_code(code).unwrap().clone()))
            .collect()
    }

    /// The shape Kiwi's server returns, trimmed to one itinerary.
    fn sse_body(currency: &str) -> String {
        let structured = json!({
            "currency": currency,
            "itineraries": [{
                "price": 379.0,
                "bookingUrl": "https://kiwi.com/u/abc",
                "baggage": { "personalItem": 1, "cabinBag": 0, "checkedBag": 0 },
                "outbound": {
                    "durationSeconds": 35760,
                    "segments": [
                        { "from": "JFK", "to": "FLL", "departureTime": "2026-11-23T16:54:00",
                          "arrivalTime": "2026-11-23T20:05:00", "durationSeconds": 11460,
                          "carrier": "B6", "carrierName": "JetBlue Airways",
                          "flightNumber": "B61301", "cabinClass": "Economy" },
                        { "from": "FLL", "to": "LAX", "departureTime": "2026-11-23T21:10:00",
                          "arrivalTime": "2026-11-23T23:50:00", "durationSeconds": 20400,
                          "carrier": "B6", "carrierName": "JetBlue Airways",
                          "flightNumber": "B6905", "cabinClass": "Economy" }
                    ]
                }
            }],
            "error": null
        });
        let envelope = json!({ "jsonrpc": "2.0", "id": 1,
            "result": { "content": [], "isError": false, "structuredContent": structured } });
        format!("event: message\ndata: {envelope}\n\n")
    }

    #[test]
    fn maps_an_event_stream_answer_onto_offers() {
        let places = places();
        let source = KiwiSource::new(reqwest::Client::new(), places.clone());
        let request = SearchQuery {
            origin: "JFK".into(),
            destination: "LAX".into(),
            depart_date: NaiveDate::from_ymd_opt(2026, 11, 23).unwrap(),
            adults: 2,
            currency: "USD".into(),
            ..Default::default()
        };
        let query = ResolvedQuery {
            request: &request,
            origin: &places["JFK"],
            destination: &places["LAX"],
        };
        let result = parse_rpc(&sse_body("USD")).unwrap();
        let offer = source
            .map_itinerary(&result.itineraries[0], &query, 2.0)
            .unwrap();
        assert_eq!(offer.price_per_traveller, 189.5);
        assert_eq!(offer.segments.len(), 2);
        assert_eq!(offer.segments[0].service_number, "B6 1301");
        assert_eq!(offer.segments[1].from.code, "FLL");
        assert!(offer.segments[0]
            .depart_local
            .starts_with("2026-11-23T16:54:00-05:00"));
        assert_eq!(offer.provider_name, "Kiwi.com");
        assert_eq!(offer.provider_url, "https://kiwi.com/u/abc");
        assert!(offer.verified_at.is_some());
    }

    #[test]
    fn asks_for_protected_connections_and_the_party_bags() {
        let places = places();
        let request = SearchQuery {
            origin: "JFK".into(),
            destination: "LAX".into(),
            depart_date: NaiveDate::from_ymd_opt(2026, 11, 23).unwrap(),
            adults: 2,
            checked_bags: 1,
            max_stops: Some(0),
            currency: "usd".into(),
            ..Default::default()
        };
        let query = ResolvedQuery {
            request: &request,
            origin: &places["JFK"],
            destination: &places["LAX"],
        };
        let arguments = arguments(&query);
        assert_eq!(arguments["departureDate"], "23/11/2026");
        assert_eq!(arguments["currency"], "USD");
        assert_eq!(arguments["adults_hold_bags"], json!([1, 1]));
        assert_eq!(arguments["allow_self_transfer"], false);
        assert_eq!(arguments["max_sector_stopovers"], 0);
        assert!(arguments.get("returnDate").is_none());
    }

    #[test]
    fn a_tool_error_is_a_rejected_request() {
        let body = r#"data: {"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Unknown location"}],"isError":true}}"#;
        assert!(matches!(
            parse_rpc(body),
            Err(SourceError::InvalidRequest(_))
        ));
    }
}
