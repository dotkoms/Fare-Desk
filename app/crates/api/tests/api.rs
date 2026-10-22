use std::collections::HashMap;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::{json, Value};
use tower::ServiceExt;

use faredesk_api::{app, search_cache_key, AppState, Config};
use faredesk_core::{SearchQuery, SearchSnapshot};
use faredesk_sources::fixtures::FixtureSource;
use faredesk_sources::Source;

fn depart_date() -> String {
    (Utc::now().date_naive() + ChronoDuration::days(60))
        .format("%Y-%m-%d")
        .to_string()
}

fn state() -> AppState {
    AppState::in_memory(Config::default()).expect("state")
}

async fn call(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    let request = match body {
        Some(body) => request.body(Body::from(body.to_string())).unwrap(),
        None => request.body(Body::empty()).unwrap(),
    };
    let response = app.oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body");
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

fn search_body(origin: &str, destination: &str) -> Value {
    json!({
        "origin": origin,
        "destination": destination,
        "departDate": depart_date(),
        "adults": 1,
        "checkedBags": 0,
        "currency": "EUR"
    })
}

#[tokio::test]
async fn places_returns_reference_airports_with_provenance() {
    let (status, body) = call(app(state()), "GET", "/api/places?q=lis&limit=5", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["places"][0]["code"], "LIS");
    assert_eq!(body["places"][0]["timezone"], "Europe/Lisbon");
    assert_eq!(body["provenance"]["dataCardId"], "airports-reference");
    assert_eq!(body["provenance"]["freshness"], "bundled");
    assert!(body["provenance"]["sampleSize"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn places_rejects_tiny_queries() {
    let (status, body) = call(app(state()), "GET", "/api/places?q=l", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "missing_query");
}

#[tokio::test]
async fn fixture_search_prices_bags_and_party_and_says_it_is_a_fixture() {
    let mut body = search_body("BER", "LIS");
    body["adults"] = json!(2);
    body["checkedBags"] = json!(1);
    let (status, body) = call(app(state()), "POST", "/api/search", Some(body)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["route"], "BER-LIS");
    assert_eq!(body["provenance"]["dataCardId"], "fares-fixture");
    assert_eq!(body["provenance"]["freshness"], "fixture");
    assert_eq!(body["provenance"]["degraded"], false);

    let itineraries = body["itineraries"].as_array().unwrap();
    assert_eq!(itineraries.len(), 5);
    assert!(itineraries
        .iter()
        .all(|itinerary| itinerary["verifiedAt"].is_null()));

    let tap = itineraries
        .iter()
        .find(|itinerary| itinerary["segments"][0]["marketingCarrier"] == "TAP Air Portugal")
        .unwrap();
    // €119 fare each, one bag included: no bag charge.
    assert_eq!(tap["totalPrice"], 238.0);
    assert_eq!(tap["priceBreakdown"]["bags"], 0.0);
    assert_eq!(tap["priceBreakdown"]["travellers"], 2);

    let ryanair = itineraries
        .iter()
        .find(|itinerary| itinerary["segments"][0]["marketingCarrier"] == "Ryanair")
        .unwrap();
    // €104 each plus one €45 bag each; it is a self-transfer.
    assert_eq!(ryanair["totalPrice"], 298.0);
    assert_eq!(ryanair["priceBreakdown"]["bags"], 90.0);
    assert_eq!(ryanair["risk"][0], "self-transfer");
    assert_eq!(ryanair["stops"], 1);

    assert!(body["provenance"]["notes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|note| note.as_str().unwrap().contains("Demo fixture")));
}

#[tokio::test]
async fn identical_searches_hit_the_cache() {
    let state = state();
    let body = search_body("BER", "LIS");
    let (first_status, first) = call(
        app(state.clone()),
        "POST",
        "/api/search",
        Some(body.clone()),
    )
    .await;
    let (second_status, second) = call(app(state), "POST", "/api/search", Some(body)).await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(first["provenance"]["freshness"], "fixture");
    assert_eq!(second["provenance"]["freshness"], "cached");
    assert_eq!(
        first["itineraries"].as_array().unwrap().len(),
        second["itineraries"].as_array().unwrap().len()
    );
}

#[tokio::test]
async fn searches_are_ranked_by_best_by_default() {
    let mut body = search_body("BER", "LIS");
    body["sort"] = json!("cheapest");
    let (_, body) = call(app(state()), "POST", "/api/search", Some(body)).await;
    let totals: Vec<f64> = body["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|itinerary| itinerary["totalPrice"].as_f64().unwrap())
        .collect();
    let mut sorted = totals.clone();
    sorted.sort_by(f64::total_cmp);
    assert_eq!(totals, sorted);
}

#[tokio::test]
async fn unknown_places_are_rejected() {
    let (status, body) = call(
        app(state()),
        "POST",
        "/api/search",
        Some(search_body("BER", "ZZZ")),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "unknown_place");
}

#[tokio::test]
async fn requesting_serpapi_without_a_key_is_a_clear_error() {
    let mut body = search_body("BER", "LIS");
    body["source"] = json!("serpapi");
    let (status, body) = call(app(state()), "POST", "/api/search", Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "source_not_configured");
}

#[tokio::test]
async fn past_dates_and_same_airports_are_rejected() {
    let mut body = search_body("BER", "LIS");
    body["departDate"] = json!("2020-01-01");
    let (status, body) = call(app(state()), "POST", "/api/search", Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "date_in_past");

    let (status, body) = call(
        app(state()),
        "POST",
        "/api/search",
        Some(search_body("BER", "BER")),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "same_place");
}

#[tokio::test]
async fn live_source_with_a_spent_quota_fails_fast() {
    let config = Config {
        serpapi_key: Some("test-key".into()),
        serpapi_daily_limit: 0,
        ..Config::default()
    };
    let mut body = search_body("BER", "LIS");
    body["source"] = json!("serpapi");
    let (status, body) = call(
        app(AppState::in_memory(config).unwrap()),
        "POST",
        "/api/search",
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "quota_exhausted");
    assert!(body["error"]["details"]["resetsOn"].is_string());
}

#[tokio::test]
async fn a_dead_source_serves_the_last_good_answer() {
    let state = AppState::in_memory_with_sources(
        Config::default(),
        Source::Fixtures(FixtureSource::new(HashMap::new(), Vec::new())),
        None,
    )
    .unwrap();

    let query = SearchQuery {
        origin: "BER".into(),
        destination: "LIS".into(),
        depart_date: chrono::NaiveDate::parse_from_str(&depart_date(), "%Y-%m-%d").unwrap(),
        ..Default::default()
    };
    let key = search_cache_key("fixtures", &query, "BER", "LIS");
    state.store().search_cache.insert(
        key,
        SearchSnapshot {
            itineraries: Vec::new(),
            source: "fixtures".into(),
            observed_at: Utc::now() - ChronoDuration::minutes(30),
            sample_size: 0,
            notes: vec!["previous good answer".into()],
        },
        Duration::from_millis(0),
    );

    let (status, body) = call(
        app(state),
        "POST",
        "/api/search",
        Some(search_body("BER", "LIS")),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["provenance"]["freshness"], "stale");
    assert_eq!(body["provenance"]["degraded"], true);
    assert!(body["provenance"]["degradedReason"]
        .as_str()
        .unwrap()
        .contains("last good result"));
}

#[tokio::test]
async fn a_dead_source_without_a_cache_is_a_typed_error() {
    let state = AppState::in_memory_with_sources(
        Config::default(),
        Source::Fixtures(FixtureSource::new(HashMap::new(), Vec::new())),
        None,
    )
    .unwrap();
    let (status, body) = call(
        app(state),
        "POST",
        "/api/search",
        Some(search_body("BER", "LIS")),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(body["error"]["code"], "source_unavailable");
}

#[tokio::test]
async fn routes_list_the_fixture_graph() {
    let (status, body) = call(app(state()), "GET", "/api/routes?origin=BER", None).await;
    assert_eq!(status, StatusCode::OK);
    let destinations: Vec<&str> = body["routes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|route| route["destination"]["code"].as_str().unwrap())
        .collect();
    assert_eq!(destinations, vec!["BCN", "JFK", "LHR", "LIS"]);
    assert_eq!(body["routes"][0]["coverage"], "fixture");
    assert_eq!(body["provenance"]["dataCardId"], "route-coverage");
}

#[tokio::test]
async fn data_cards_expose_every_source() {
    let (status, body) = call(app(state()), "GET", "/api/data-card", None).await;
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<&str> = body["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|card| card["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"fares-serpapi"));
    assert!(ids.contains(&"fares-fixture"));
    assert!(ids.contains(&"route-coverage"));

    let (status, body) = call(app(state()), "GET", "/api/data-card?dataset=fares", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["cards"].as_array().unwrap().len(), 2);

    let (status, _) = call(app(state()), "GET", "/api/data-card?dataset=nope", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn fx_identity_rates_do_not_need_the_network() {
    let (status, body) = call(
        app(state()),
        "GET",
        "/api/reference/fx?base=EUR&quote=EUR",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["rate"]["rate"], 1.0);
    assert_eq!(body["rate"]["source"], "identity");
    assert_eq!(body["provenance"]["dataCardId"], "fx-frankfurter");
}

#[tokio::test]
async fn providers_come_from_the_registry() {
    let (status, body) = call(app(state()), "GET", "/api/reference/providers", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["providers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|provider| provider["id"] == "google-flights"));
}

#[tokio::test]
async fn health_reports_sources_and_reference_size() {
    let (status, body) = call(app(state()), "GET", "/healthz", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert!(body["places"].as_u64().unwrap() > 2_000);
    let serpapi = body["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|source| source["id"] == "serpapi")
        .unwrap();
    assert_eq!(serpapi["configured"], false);
}
