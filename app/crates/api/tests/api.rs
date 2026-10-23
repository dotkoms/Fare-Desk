use std::collections::HashMap;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use serde_json::{json, Value};
use tower::ServiceExt;

use faredesk_api::{app, search_cache_key, AppState, Config};
use faredesk_core::{ClimateMonth, SearchQuery, SearchSnapshot};
use faredesk_sources::fixtures::FixtureSource;
use faredesk_sources::Source;

fn depart_date() -> String {
    (Utc::now().date_naive() + ChronoDuration::days(60))
        .format("%Y-%m-%d")
        .to_string()
}

fn state() -> AppState {
    // Ground routing is off so tests never call OSRM.
    AppState::in_memory(Config {
        ground_routing: false,
        transit_routing: false,
        ..Config::default()
    })
    .expect("state")
}

async fn call(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    call_as(app, method, uri, body, None).await
}

async fn call_as(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
    client: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(client) = client {
        request = request.header("x-forwarded-for", client);
    }
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

#[tokio::test]
async fn rate_limiter_refuses_after_the_limit_but_leaves_health_alone() {
    let config = Config {
        rate_limit_per_minute: 2,
        ..Config::default()
    };
    let app = app(AppState::in_memory(config).unwrap());

    for _ in 0..2 {
        let (status, _) = call_as(
            app.clone(),
            "GET",
            "/api/places?q=lis",
            None,
            Some("203.0.113.7"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let (status, body) = call_as(
        app.clone(),
        "GET",
        "/api/places?q=lis",
        None,
        Some("203.0.113.7"),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "rate_limited");

    let (status, _) = call_as(app.clone(), "GET", "/healthz", None, Some("203.0.113.7")).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = call_as(app, "GET", "/api/places?q=lis", None, Some("198.51.100.4")).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn unknown_endpoints_return_a_json_error() {
    let (status, body) = call(app(state()), "GET", "/api/nope", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("data-card"));
}

#[tokio::test]
async fn route_intel_says_insufficient_without_history() {
    let (status, body) = call(
        app(state()),
        "GET",
        "/api/route-intel?from=BER&to=LIS&date=2027-03-01",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["intel"]["route"], "BER-LIS");
    assert_eq!(body["intel"]["sampleSize"], 0);
    assert_eq!(body["intel"]["recommendation"], "insufficient");
    assert_eq!(body["intel"]["history"].as_array().unwrap().len(), 0);
    assert_eq!(body["provenance"]["dataCardId"], "fare-history");
    assert!(body["provenance"]["notes"][0]
        .as_str()
        .unwrap()
        .contains("Collecting baseline — 0 of 14"));
}

#[tokio::test]
async fn route_intel_returns_percentiles_at_fourteen_days() {
    let state = state();
    let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
    let first_day = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
    for offset in 0..14 {
        state
            .store()
            .db
            .record_daily_minimum(
                "BER-LIS",
                travel,
                first_day + ChronoDuration::days(offset),
                100.0 + offset as f64,
                "EUR",
                "serpapi",
                "TP-1041",
            )
            .unwrap();
    }

    let (status, body) = call(
        app(state),
        "GET",
        "/api/route-intel?from=BER&to=LIS&date=2027-03-01",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let intel = &body["intel"];
    assert_eq!(intel["sampleSize"], 14);
    assert_eq!(intel["currentPrice"], 113.0);
    assert_eq!(intel["median"], 106.5);
    assert_eq!(intel["p10"], 101.3);
    assert_eq!(intel["p90"], 111.7);
    assert_eq!(intel["percentile"], 100.0);
    assert_eq!(intel["window"]["days"], 14);
    // Above the median with months to go: wait, with reasons and a 7-day range.
    assert_eq!(intel["recommendation"], "wait");
    assert!(intel["expected7"]["low"].as_f64().unwrap() > 0.0);
    assert!(
        intel["expected7"]["high"].as_f64().unwrap() >= intel["expected7"]["low"].as_f64().unwrap()
    );
    assert!(intel["expected21"].is_null());
    assert_eq!(intel["curve"].as_array().unwrap().len(), 1);
    assert!(intel["backtest"].is_null());
    assert!(!intel["reasons"].as_array().unwrap().is_empty());
    assert!(intel["changeMyMind"]
        .as_str()
        .unwrap()
        .contains("sale bucket"));
    assert_eq!(intel["history"].as_array().unwrap().len(), 14);
    assert!(body["provenance"]["notes"][0]
        .as_str()
        .unwrap()
        .contains("Track record is not available yet"));
}

#[tokio::test]
async fn route_intel_requires_a_date() {
    let (status, body) = call(
        app(state()),
        "GET",
        "/api/route-intel?from=BER&to=LIS",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "missing_date");
}

#[tokio::test]
async fn route_intel_books_when_the_fare_is_below_p10() {
    let state = state();
    let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
    let first_day = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
    // 25 descending daily minima: today's price is the lowest in the series.
    for offset in 0..25 {
        state
            .store()
            .db
            .record_daily_minimum(
                "BER-LIS",
                travel,
                first_day + ChronoDuration::days(offset),
                200.0 - offset as f64,
                "EUR",
                "serpapi",
                "TP-1041",
            )
            .unwrap();
    }

    let (status, body) = call(
        app(state),
        "GET",
        "/api/route-intel?from=BER&to=LIS&date=2027-03-01",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let intel = &body["intel"];
    assert_eq!(intel["sampleSize"], 25);
    assert_eq!(intel["recommendation"], "buy");
    assert_eq!(intel["percentile"], 4.0);
    assert!(intel["reasons"][0]
        .as_str()
        .unwrap()
        .contains("10th percentile"));
    // 25 points give enough 21-day moves for both ranges.
    assert!(intel["expected7"]["low"].is_number());
    assert!(intel["expected21"]["low"].is_number());
    // All observations are months before departure, so no checkpoint has
    // completed its wait yet; the backtest stays honest and null.
    assert!(intel["backtest"].is_null());
}

fn price_check_body(itinerary_id: &str) -> Value {
    json!({
        "origin": "BER",
        "destination": "LIS",
        "departDate": depart_date(),
        "itineraryId": itinerary_id,
        "adults": 1,
        "checkedBags": 1,
        "homeCurrency": "EUR",
        "fxFeePercent": 3,
        "profile": {
            "residency": "DE",
            "nationality": "DE",
            "cardCountries": ["DE"],
            "paymentMethods": ["card"]
        }
    })
}

#[tokio::test]
async fn price_check_screens_markets_and_keeps_home_for_flat_fixtures() {
    let state = state();
    let (status, search) = call(
        app(state.clone()),
        "POST",
        "/api/search",
        Some(search_body("BER", "LIS")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let itinerary_id = search["itineraries"][0]["id"].as_str().unwrap().to_string();

    let (status, body) = call(
        app(state),
        "POST",
        "/api/price-check",
        Some(price_check_body(&itinerary_id)),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["homeMarket"], "DE");
    assert_eq!(body["homeCurrency"], "EUR");
    let quotes = body["quotes"].as_array().unwrap();
    assert_eq!(quotes.len(), 5);
    assert_eq!(quotes[0]["market"], "DE");
    assert_eq!(quotes[0]["status"], "eligible");
    assert!(quotes
        .iter()
        .any(|q| q["market"] == "PT" && q["status"] == "listed"));
    // Fixtures are market-agnostic: every storefront shows the same total.
    assert!(quotes
        .iter()
        .all(|quote| quote["netTotal"].as_f64() == quotes[0]["netTotal"].as_f64()));
    assert_eq!(body["recommendation"]["choice"], "home");
    assert_eq!(body["recommendation"]["saving"], 0.0);
    assert_eq!(body["provenance"]["dataCardId"], "pos-quotes");
    assert_eq!(body["provenance"]["freshness"], "fixture");
    assert!(body["provenance"]["notes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|note| note.as_str().unwrap().contains("Fixture source")));
}

#[tokio::test]
async fn price_check_rejects_an_unknown_itinerary() {
    let (status, body) = call(
        app(state()),
        "POST",
        "/api/price-check",
        Some(price_check_body("NOPE-1-2027-03-01")),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "itinerary_not_found");
}

#[tokio::test]
async fn price_check_validates_the_profile() {
    let mut body = price_check_body("TAPAirPortugal-TP1041-2027-03-01");
    body["profile"]["residency"] = json!("DEU");
    let (status, body) = call(app(state()), "POST", "/api/price-check", Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_profile");
}

#[tokio::test]
async fn seasonality_returns_monthly_medians_and_omits_thin_months() {
    let state = state();
    // Six March observations across six travel dates, four April ones.
    for offset in 0..6 {
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap() + ChronoDuration::days(offset);
        let observed = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap() + ChronoDuration::days(offset);
        state
            .store()
            .db
            .record_daily_minimum(
                "BER-LIS",
                travel,
                observed,
                100.0 + offset as f64,
                "EUR",
                "serpapi",
                "TP-1",
            )
            .unwrap();
    }
    for offset in 0..4 {
        let travel = NaiveDate::from_ymd_opt(2027, 4, 1).unwrap() + ChronoDuration::days(offset);
        let observed =
            NaiveDate::from_ymd_opt(2026, 10, 10).unwrap() + ChronoDuration::days(offset);
        state
            .store()
            .db
            .record_daily_minimum(
                "BER-LIS",
                travel,
                observed,
                200.0 + offset as f64,
                "EUR",
                "serpapi",
                "TP-1",
            )
            .unwrap();
    }

    let (status, body) = call(app(state), "GET", "/api/seasonality?from=BER&to=LIS", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["route"], "BER-LIS");
    let points = body["points"].as_array().unwrap();
    assert_eq!(points.len(), 1);
    assert_eq!(points[0]["month"], "2027-03");
    assert_eq!(points[0]["sample"], 6);
    assert_eq!(points[0]["median"], 103.0);
    assert_eq!(body["sampleSize"], 10);
    assert_eq!(body["provenance"]["dataCardId"], "fare-history");
}

#[tokio::test]
async fn fare_weather_reports_volatility_from_daily_minima() {
    let state = state();
    let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
    for offset in 0..6 {
        let observed = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap() + ChronoDuration::days(offset);
        state
            .store()
            .db
            .record_daily_minimum(
                "BER-LIS",
                travel,
                observed,
                100.0 + offset as f64,
                "EUR",
                "serpapi",
                "TP-1",
            )
            .unwrap();
    }

    let (status, body) = call(
        app(state),
        "GET",
        "/api/fare-weather?from=BER&to=LIS&date=2027-03-01",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let weather = &body["weather"];
    assert_eq!(weather["route"], "BER-LIS");
    assert_eq!(weather["classification"], "stable");
    assert_eq!(weather["sampleSize"], 6);
    assert_eq!(weather["range7"]["low"], 100.0);
    assert_eq!(weather["range7"]["high"], 105.0);
    assert!(weather["cv"].as_f64().unwrap() < 0.12);
}

#[tokio::test]
async fn fare_weather_is_insufficient_without_history() {
    let (status, body) = call(
        app(state()),
        "GET",
        "/api/fare-weather?from=BER&to=LIS&date=2027-03-01",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["weather"]["classification"], "insufficient");
    assert_eq!(body["weather"]["sampleSize"], 0);
    assert!(body["weather"]["range7"].is_null());
}

#[tokio::test]
async fn explore_returns_a_sorted_validated_feed() {
    let (status, body) = call(
        app(state()),
        "GET",
        "/api/explore?home=BER&month=2027-03",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["home"]["code"], "BER");
    assert_eq!(body["month"], "2027-03");
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0]["destination"]["code"], "BCN");
    assert_eq!(rows[0]["totalPrice"], 69.0);
    assert_eq!(rows[0]["travelDate"], "2027-03-15");
    assert_eq!(rows[3]["destination"]["code"], "JFK");
    assert_eq!(body["coverage"]["priced"], 4);
    assert_eq!(body["coverage"]["attempted"], 4);
    assert_eq!(body["provenance"]["dataCardId"], "explore-feed");
    assert_eq!(body["provenance"]["freshness"], "fixture");
    assert!(body["provenance"]["notes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|note| note.as_str().unwrap().contains("demo route graph")));
}

#[tokio::test]
async fn explore_rejects_a_bad_month() {
    let (status, body) = call(
        app(state()),
        "GET",
        "/api/explore?home=BER&month=March",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_month");
}

#[tokio::test]
async fn airport_access_returns_transfers_and_mct() {
    let (status, body) = call(app(state()), "GET", "/api/airport-access?airport=BER", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["place"]["code"], "BER");
    assert_eq!(body["cityCenter"]["lat"], 52.52);
    let transfers = body["transfers"].as_array().unwrap();
    assert_eq!(transfers.len(), 2);
    assert_eq!(transfers[0]["mode"], "train");
    assert_eq!(body["mct"]["domestic"], 30);
    assert_eq!(body["mct"]["international"], 45);
    assert_eq!(body["provenance"]["dataCardId"], "airport-access");
}

#[tokio::test]
async fn airport_access_is_empty_but_valid_for_uncurated_airports() {
    let (status, body) = call(app(state()), "GET", "/api/airport-access?airport=AKL", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["transfers"].as_array().unwrap().len(), 0);
    assert!(body["provenance"]["notes"][1]
        .as_str()
        .unwrap()
        .contains("No curated access data"));
}

#[tokio::test]
async fn airport_access_rejects_unknown_places() {
    let (status, body) = call(app(state()), "GET", "/api/airport-access?airport=ZZZ", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "unknown_place");
}

#[tokio::test]
async fn climate_uses_the_cached_normals() {
    let state = state();
    let year = Utc::now().year() - 1;
    state.store().climate_cache.insert(
        format!("climate:BER:{year}"),
        vec![
            ClimateMonth {
                month: 1,
                mean_high: 5.0,
                mean_low: -1.0,
                rain_days: 10,
                comfort: 0,
            },
            ClimateMonth {
                month: 7,
                mean_high: 26.0,
                mean_low: 16.0,
                rain_days: 5,
                comfort: 80,
            },
        ],
        Duration::from_secs(60),
    );

    let (status, body) = call(app(state), "GET", "/api/reference/climate?place=BER", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["place"]["code"], "BER");
    assert_eq!(body["year"], year);
    assert_eq!(body["months"].as_array().unwrap().len(), 2);
    assert_eq!(body["months"][1]["comfort"], 80);
    assert_eq!(body["provenance"]["freshness"], "cached");
    assert_eq!(body["provenance"]["dataCardId"], "climate-open-meteo");
}

#[tokio::test]
async fn door_to_door_adds_the_cheapest_transfer_at_each_end() {
    let (status, body) = call(
        app(state()),
        "GET",
        "/api/door-to-door?from=BER&to=LIS&date=2027-03-01&checkedBags=0",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["route"], "BER-LIS");
    assert_eq!(body["currency"], "EUR");
    // Cheapest fixture fare with no checked bag is the €104 Ryanair connection.
    assert_eq!(body["fare"], 104.0);
    // Berlin train €4.40 plus Lisbon metro €2.00.
    assert_eq!(body["transferCost"], 6.4);
    assert_eq!(body["total"], 110.4);
    assert_eq!(body["flightMinutes"], 360);
    assert_eq!(body["transferMinutes"], 55);
    assert_eq!(body["totalMinutes"], 415);
    assert_eq!(body["origin"]["mode"], "train");
    assert_eq!(body["destination"]["mode"], "metro");
    assert_eq!(body["provenance"]["dataCardId"], "fares-fixture");
    assert!(body["provenance"]["notes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|note| note.as_str().unwrap().contains("Reliability-adjusted")));
}

#[tokio::test]
async fn door_to_door_requires_a_date() {
    let (status, body) = call(
        app(state()),
        "GET",
        "/api/door-to-door?from=BER&to=LIS",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "missing_date");
}

#[tokio::test]
async fn fixture_return_search_prices_both_directions() {
    let mut body = search_body("BER", "LIS");
    let back = (Utc::now().date_naive() + ChronoDuration::days(67))
        .format("%Y-%m-%d")
        .to_string();
    body["returnDate"] = json!(back);
    let (status, body) = call(app(state()), "POST", "/api/search", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["query"]["returnDate"], json!(back));
    assert_eq!(body["itineraries"].as_array().unwrap().len(), 5);
    let returns = body["returnItineraries"].as_array().expect("return flights");
    assert_eq!(returns.len(), 5);
    for itinerary in returns {
        let segments = itinerary["segments"].as_array().unwrap();
        assert_eq!(segments[0]["from"]["code"], "LIS");
        assert_eq!(segments.last().unwrap()["to"]["code"], "BER");
        assert!(segments[0]["departLocal"]
            .as_str()
            .unwrap()
            .starts_with(&back));
    }
    assert!(body["provenance"]["notes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|note| note.as_str().unwrap().contains("two one-way tickets")));
}

/// A stand-in for SerpAPI: one nonstop each way, priced by direction.
async fn mock_serpapi() -> String {
    use axum::extract::Query as AxumQuery;
    use axum::routing::get;

    async fn handler(AxumQuery(params): AxumQuery<HashMap<String, String>>) -> axum::Json<Value> {
        let from = params["departure_id"].clone();
        let to = params["arrival_id"].clone();
        let date = params["outbound_date"].clone();
        let price = if from == "BER" { 111.0 } else { 99.0 };
        axum::Json(json!({
            "best_flights": [{
                "flights": [{
                    "departure_airport": { "id": from, "time": format!("{date} 08:00") },
                    "arrival_airport": { "id": to, "time": format!("{date} 10:40") },
                    "duration": 220,
                    "airline": "TAP Air Portugal",
                    "travel_class": "Economy",
                    "flight_number": "TP 1041"
                }],
                "price": price,
                "total_duration": 220,
                "extensions": []
            }]
        }))
    }

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, axum::Router::new().route("/search.json", get(handler)))
            .await
            .unwrap();
    });
    format!("http://{addr}/search.json")
}

#[tokio::test]
async fn live_return_search_is_two_metered_one_way_searches() {
    let base = mock_serpapi().await;
    let state = AppState::in_memory(Config {
        ground_routing: false,
        transit_routing: false,
        serpapi_key: Some("test-key".into()),
        serpapi_base_url: Some(base),
        ..Config::default()
    })
    .expect("state");
    let mut body = search_body("BER", "LIS");
    let back = (Utc::now().date_naive() + ChronoDuration::days(67))
        .format("%Y-%m-%d")
        .to_string();
    body["returnDate"] = json!(back);

    let (status, body) = call(app(state.clone()), "POST", "/api/search", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["provenance"]["freshness"], "live");
    assert_eq!(body["itineraries"][0]["totalPrice"], 111.0);
    assert_eq!(body["returnItineraries"][0]["totalPrice"], 99.0);
    assert_eq!(body["returnItineraries"][0]["segments"][0]["from"]["code"], "LIS");

    let (_, health) = call(app(state), "GET", "/healthz", None).await;
    assert_eq!(health["sources"][1]["usedToday"], 2, "{health}");
}
