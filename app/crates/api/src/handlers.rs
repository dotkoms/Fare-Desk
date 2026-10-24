use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::time::Instant;

use axum::extract::{ConnectInfo, Path, Query, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use faredesk_core::{
    adjusted_price, comfort, fare_weather, on_time_rate, seasonality, AirportAccessResponse,
    CarrierReliability, ClimateResponse, DoorToDoorLeg, DoorToDoorResponse, ExploreCache,
    ExploreCoverage, ExploreResponse, ExploreRow, FareWeatherResponse, ReliabilityResponse,
    SeasonalityResponse,
};
use faredesk_core::{
    advise, apply_scores, currency_for_country, fingerprint, net_total, percentile,
    percentile_rank, price_offer, recommend, screening_markets, sort_itineraries, status_for,
    AdviceInput, Cabin, Coverage, DataCard, Dataset, EligibilityProfile, Freshness, FxResponse,
    HistoryPoint, Holiday, HolidaysResponse, Itinerary, Observation, ObservationWindow, Place,
    PlacesResponse, PosQuote, PosStatus, PriceCheckResponse, Provenance, Provider,
    ProvidersResponse, Recommendation, RouteIntel, RouteIntelResponse, RouteSummary,
    RoutesResponse, SearchQuery, SearchResponse, SearchSnapshot, Sort, MIN_OBSERVATIONS,
};
use faredesk_sources::{ResolvedQuery, Source, SourceError};
use faredesk_store::{access_for, access_note};
use faredesk_store::{Cached, QuotaOutcome};

use crate::error::ApiError;
use crate::AppState;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn now() -> DateTime<Utc> {
    Utc::now()
}

fn base_provenance(card_id: &str, source: &str, sample_size: usize) -> Provenance {
    Provenance {
        data_card_id: card_id.to_string(),
        source: source.to_string(),
        observed_at: now(),
        freshness: Freshness::Bundled,
        age_seconds: 0,
        expires_at: None,
        degraded: false,
        degraded_reason: None,
        sample_size,
        notes: Vec::new(),
    }
}

fn resolve_place(state: &AppState, value: &str) -> Result<Place, ApiError> {
    state
        .store()
        .reference
        .resolve(value)
        .cloned()
        .ok_or_else(|| {
            ApiError::not_found(
                "unknown_place",
                format!("Unknown airport or place {value:?}. Try an IATA code such as BER."),
            )
        })
}

fn ttl_duration(ttl: std::time::Duration) -> Duration {
    Duration::from_std(ttl).unwrap_or_else(|_| Duration::zero())
}

pub async fn not_found() -> ApiError {
    ApiError::not_found(
        "not_found",
        "No such endpoint. See /api/data-card for the dataset list.",
    )
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceHealth {
    pub id: &'static str,
    pub configured: bool,
    pub live: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_today: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub status: &'static str,
    pub version: &'static str,
    pub started_at: DateTime<Utc>,
    pub uptime_seconds: i64,
    pub places: usize,
    pub cache_entries: usize,
    pub sources: Vec<SourceHealth>,
}

pub async fn health(State(state): State<AppState>) -> Json<Health> {
    let today = now().date_naive();
    let config = state.config();
    let serpapi_used = state
        .serpapi()
        .and_then(|_| state.store().db.quota_used("serpapi", today).ok());

    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        started_at: state.started_at(),
        uptime_seconds: (now() - state.started_at()).num_seconds(),
        places: state.store().reference.len(),
        cache_entries: state.store().cache_len(),
        sources: vec![
            SourceHealth {
                id: "fixtures",
                configured: state.fixtures().is_some(),
                live: false,
                daily_limit: None,
                used_today: None,
            },
            SourceHealth {
                id: "serpapi",
                configured: state.serpapi().is_some(),
                live: true,
                daily_limit: state.serpapi().map(|_| config.serpapi_daily_limit),
                used_today: serpapi_used,
            },
            SourceHealth {
                id: "skiplagged",
                configured: state.skiplagged().is_some(),
                live: true,
                daily_limit: state.skiplagged().map(|_| config.skiplagged_daily_limit),
                used_today: state
                    .skiplagged()
                    .and_then(|_| state.store().db.quota_used("skiplagged", today).ok()),
            },
            SourceHealth {
                id: "kiwi",
                configured: state.kiwi().is_some(),
                live: true,
                daily_limit: state.kiwi().map(|_| config.kiwi_daily_limit),
                used_today: state
                    .kiwi()
                    .and_then(|_| state.store().db.quota_used("kiwi", today).ok()),
            },
        ],
    })
}

// ---------------------------------------------------------------------------
// Places
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PlacesParams {
    pub q: Option<String>,
    pub limit: Option<usize>,
}

pub async fn places(
    State(state): State<AppState>,
    Query(params): Query<PlacesParams>,
) -> Result<Json<PlacesResponse>, ApiError> {
    let query = params.q.unwrap_or_default();
    if query.trim().len() < 2 {
        return Err(ApiError::bad_request(
            "missing_query",
            "Give at least two characters, e.g. /api/places?q=lis.",
        ));
    }
    let limit = params.limit.unwrap_or(8).clamp(1, 25);
    let results = state.store().reference.search(&query, limit);

    let mut provenance = base_provenance("airports-reference", "reference", results.len());
    provenance.observed_at = state.store().reference.generated_at();
    provenance.age_seconds = (now() - state.store().reference.generated_at())
        .num_seconds()
        .max(0) as u64;
    provenance.notes = vec![format!(
        "{} airports in the reference set; searches cover IATA code, city, airport name and country.",
        state.store().reference.len()
    )];

    Ok(Json(PlacesResponse {
        places: results,
        provenance,
    }))
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RoutesParams {
    pub origin: Option<String>,
    pub limit: Option<usize>,
}

pub async fn routes(
    State(state): State<AppState>,
    Query(params): Query<RoutesParams>,
) -> Result<Json<RoutesResponse>, ApiError> {
    let origin_value = params.origin.unwrap_or_default();
    if origin_value.trim().is_empty() {
        return Err(ApiError::bad_request(
            "missing_origin",
            "Give an origin airport, e.g. /api/routes?origin=BER.",
        ));
    }
    let origin = resolve_place(&state, &origin_value)?;
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let live = state.live_available();
    let observed = state
        .store()
        .db
        .observed_destinations(&origin.code)
        .map_err(|error| ApiError::internal(format!("route history: {error}")))?;

    let mut summaries: Vec<RouteSummary> = observed
        .into_iter()
        .filter_map(|(code, last)| {
            let destination = state.store().reference.by_code(&code)?.clone();
            Some(RouteSummary {
                origin: origin.clone(),
                destination,
                coverage: if live { Coverage::Live } else { Coverage::None },
                last_observed_at: last.and_hms_opt(0, 0, 0).map(|day| day.and_utc()),
            })
        })
        .collect();
    summaries.sort_by(|a, b| a.destination.code.cmp(&b.destination.code));
    summaries.truncate(limit);

    let mut provenance = base_provenance("route-coverage", "fare-history", summaries.len());
    provenance.notes = vec![format!(
        "Routes from {} with recorded live fares; a schedule feed is needed to list routes never priced.",
        origin.code
    )];
    if !live {
        provenance.notes.push(
            "No live fare source is configured, so these routes cannot be priced now.".to_string(),
        );
    }
    if summaries.is_empty() {
        provenance
            .notes
            .push(format!("No route data for {} yet.", origin.code));
    }

    Ok(Json(RoutesResponse {
        routes: summaries,
        provenance,
    }))
}

// ---------------------------------------------------------------------------
// Route intelligence
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RouteIntelParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<NaiveDate>,
}

pub async fn route_intel(
    State(state): State<AppState>,
    Query(params): Query<RouteIntelParams>,
) -> Result<Json<RouteIntelResponse>, ApiError> {
    let from = params.from.unwrap_or_default();
    let to = params.to.unwrap_or_default();
    if from.trim().is_empty() || to.trim().is_empty() {
        return Err(ApiError::bad_request(
            "missing_route",
            "Give from and to, e.g. /api/route-intel?from=BER&to=LIS&date=2027-03-01.",
        ));
    }
    let date = params.date.ok_or_else(|| {
        ApiError::bad_request(
            "missing_date",
            "Give the travel date, e.g. /api/route-intel?from=BER&to=LIS&date=2027-03-01.",
        )
    })?;

    let origin = resolve_place(&state, &from)?;
    let destination = resolve_place(&state, &to)?;
    if origin.code == destination.code {
        return Err(ApiError::bad_request(
            "same_place",
            "Origin and destination are the same airport.",
        ));
    }
    let route = format!("{}-{}", origin.code, destination.code);

    let rows = state
        .store()
        .db
        .daily_minima(&route, date)
        .map_err(|error| ApiError::internal(format!("fare history: {error}")))?;
    let currency = state
        .store()
        .db
        .history_currency(&route, date)
        .ok()
        .flatten()
        .unwrap_or_else(|| "EUR".to_string());

    let mut sorted: Vec<f64> = rows.iter().map(|(_, price)| *price).collect();
    sorted.sort_by(f64::total_cmp);

    let current_price = rows.last().map(|(_, price)| *price);
    let current_percentile = current_price
        .and_then(|price| percentile_rank(&sorted, price))
        .map(|value| (value * 10.0).round() / 10.0);
    let median = percentile(&sorted, 50.0);
    let p10 = percentile(&sorted, 10.0);
    let p90 = percentile(&sorted, 90.0);
    let sample_size = rows.len();
    let window = match (rows.first(), rows.last()) {
        (Some((from_date, _)), Some((to_date, _))) => Some(ObservationWindow {
            from: *from_date,
            to: *to_date,
            days: (*to_date - *from_date).num_days().max(0) as u32 + 1,
        }),
        _ => None,
    };
    let history: Vec<HistoryPoint> = rows
        .iter()
        .map(|(date, price)| HistoryPoint {
            date: *date,
            price: *price,
        })
        .collect();

    let route_observations: Vec<Observation> = state
        .store()
        .db
        .route_observations(&route)
        .map_err(|error| ApiError::internal(format!("fare history: {error}")))?
        .into_iter()
        .map(|(travel_date, observed_on, price)| Observation {
            travel_date,
            observed_on,
            price,
        })
        .collect();

    let advice = current_price.map(|current| {
        advise(&AdviceInput {
            observations: &route_observations,
            travel_date: date,
            today: now().date_naive(),
            current_price: current,
            p10: p10.unwrap_or(current),
            p50: median.unwrap_or(current),
            sample_size,
            currency: &currency,
        })
    });

    let recommendation = advice
        .as_ref()
        .map(|advice| advice.recommendation)
        .or(Some(Recommendation::Insufficient));
    let expected7 = advice.as_ref().and_then(|advice| advice.expected7);
    let expected21 = advice.as_ref().and_then(|advice| advice.expected21);
    let downside = advice.as_ref().and_then(|advice| advice.downside);
    let curve = advice
        .as_ref()
        .map(|advice| advice.curve.clone())
        .unwrap_or_default();
    let backtest = advice.as_ref().and_then(|advice| advice.backtest);
    let reasons = advice
        .as_ref()
        .map(|advice| advice.reasons.clone())
        .unwrap_or_default();
    let change_my_mind = advice
        .as_ref()
        .map(|advice| advice.change_my_mind.clone())
        .unwrap_or_default();

    let mut provenance = base_provenance("fare-history", "fare-history", sample_size);
    provenance.observed_at = rows
        .last()
        .and_then(|(date, _)| {
            date.and_hms_opt(0, 0, 0)
                .map(|time| DateTime::<Utc>::from_naive_utc_and_offset(time, Utc))
        })
        .unwrap_or_else(now);
    provenance.age_seconds = (now() - provenance.observed_at).num_seconds().max(0) as u64;
    provenance.notes.push(if sample_size == 0 {
        format!("Collecting baseline — 0 of {MIN_OBSERVATIONS} daily observations. No observations for this route and travel date yet.")
    } else if sample_size < MIN_OBSERVATIONS as usize {
        format!("Collecting baseline — {sample_size} of {MIN_OBSERVATIONS} daily observations.")
    } else {
        format!(
            "Recommendation from {sample_size} observed days. Track record is not available yet."
        )
    });

    Ok(Json(RouteIntelResponse {
        intel: RouteIntel {
            route,
            travel_date: date,
            currency,
            current_price,
            percentile: current_percentile,
            median,
            p10,
            p90,
            history,
            sample_size,
            window,
            recommendation,
            min_observations: MIN_OBSERVATIONS,
            expected7,
            expected21,
            downside,
            curve,
            backtest,
            track_record: None,
            reasons,
            change_my_mind,
        },
        provenance,
    }))
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

fn one() -> u8 {
    1
}

fn eur() -> String {
    "EUR".to_string()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchBody {
    pub origin: String,
    pub destination: String,
    pub depart_date: NaiveDate,
    #[serde(default)]
    pub return_date: Option<NaiveDate>,
    #[serde(default = "one")]
    pub adults: u8,
    #[serde(default)]
    pub children: u8,
    #[serde(default)]
    pub checked_bags: u8,
    #[serde(default = "one")]
    pub carry_on_bags: u8,
    #[serde(default = "eur")]
    pub currency: String,
    #[serde(default)]
    pub cabin: Cabin,
    #[serde(default)]
    pub max_stops: Option<u8>,
    #[serde(default)]
    pub sort: Sort,
    #[serde(default)]
    pub source: Option<String>,
}

impl SearchBody {
    fn validated(self) -> Result<SearchQuery, ApiError> {
        if !(1..=9).contains(&self.adults) {
            return Err(ApiError::bad_request(
                "invalid_party",
                "adults must be between 1 and 9.",
            ));
        }
        if self.children > 8 {
            return Err(ApiError::bad_request(
                "invalid_party",
                "children must be 8 or fewer.",
            ));
        }
        if self.checked_bags > 5 || self.carry_on_bags > 4 {
            return Err(ApiError::bad_request(
                "invalid_bags",
                "At most 5 checked bags and 4 carry-on bags per traveller.",
            ));
        }
        if self.max_stops.is_some_and(|stops| stops > 3) {
            return Err(ApiError::bad_request(
                "invalid_stops",
                "maxStops must be 3 or fewer.",
            ));
        }

        let today = now().date_naive();
        if self.depart_date < today {
            return Err(ApiError::bad_request(
                "date_in_past",
                "The departure date has already passed.",
            ));
        }
        if self.depart_date > today + Duration::days(730) {
            return Err(ApiError::bad_request(
                "date_too_far",
                "Schedules are only searched up to two years ahead.",
            ));
        }
        if let Some(return_date) = self.return_date {
            if return_date < self.depart_date {
                return Err(ApiError::bad_request(
                    "invalid_dates",
                    "The return date is before the departure date.",
                ));
            }
        }

        let currency = self.currency.trim().to_uppercase();
        if currency.len() != 3
            || !currency
                .chars()
                .all(|character| character.is_ascii_alphabetic())
        {
            return Err(ApiError::bad_request(
                "invalid_currency",
                "currency must be a three-letter ISO code, e.g. EUR.",
            ));
        }

        Ok(SearchQuery {
            origin: self.origin.trim().to_uppercase(),
            destination: self.destination.trim().to_uppercase(),
            depart_date: self.depart_date,
            return_date: self.return_date,
            adults: self.adults,
            children: self.children,
            checked_bags: self.checked_bags,
            carry_on_bags: self.carry_on_bags,
            currency,
            cabin: self.cabin,
            max_stops: self.max_stops,
            sort: self.sort,
        })
    }
}

pub async fn search(
    State(state): State<AppState>,
    Json(body): Json<SearchBody>,
) -> Result<Json<SearchResponse>, ApiError> {
    let requested_source = body.source.clone();
    let query = body.validated()?;
    let origin = resolve_place(&state, &query.origin)?;
    let destination = resolve_place(&state, &query.destination)?;
    if origin.code == destination.code {
        return Err(ApiError::bad_request(
            "same_place",
            "Origin and destination are the same airport.",
        ));
    }

    let source = state.source(requested_source.as_deref())?;
    let live = source.is_live();

    if !live && query.currency != "EUR" {
        return Err(ApiError::bad_request(
            "unsupported_currency",
            "Fixtures are priced in EUR. A live source can quote another currency.",
        ));
    }

    // A return is two one-way searches through the same cached, metered path: the live
    // source prices one direction at a time, and the traveller buys two tickets.
    let outbound_query = SearchQuery {
        return_date: None,
        ..query.clone()
    };
    let mut response =
        search_direction(&state, &source, outbound_query, &origin, &destination).await?;

    if let Some(return_date) = query.return_date {
        let inbound_query = SearchQuery {
            origin: destination.code.clone(),
            destination: origin.code.clone(),
            depart_date: return_date,
            return_date: None,
            ..query.clone()
        };
        match search_direction(&state, &source, inbound_query, &destination, &origin).await {
            Ok(inbound) => {
                response.provenance.sample_size += inbound.itineraries.len();
                if inbound.provenance.degraded {
                    response.provenance.degraded = true;
                    response.provenance.degraded_reason = inbound
                        .provenance
                        .degraded_reason
                        .map(|reason| format!("Return flights: {reason}"));
                }
                response.return_itineraries = inbound.itineraries;
            }
            Err(error) => {
                response.provenance.degraded = true;
                response.provenance.degraded_reason =
                    Some(format!("Return flights are unavailable: {}", error.message));
            }
        }
        response.provenance.notes.push(
            "Outbound and return are priced as two one-way tickets; a round-trip fare on the provider can differ."
                .to_string(),
        );
    }

    response.query = query;
    Ok(Json(response))
}

/// One direction of a search: cache, quota, source, pricing, MCT warnings and history.
async fn search_direction(
    state: &AppState,
    source: &Source,
    query: SearchQuery,
    origin: &Place,
    destination: &Place,
) -> Result<SearchResponse, ApiError> {
    let live = source.is_live();

    let route = format!("{}-{}", origin.code, destination.code);
    let key = crate::search_cache_key(source.id(), &query, &origin.code, &destination.code);

    if let Some(cached) = state.store().search_cache.get(&key) {
        return Ok(response_from_snapshot(
            &query,
            &route,
            cached,
            source.data_card_id(),
            source.id(),
            Freshness::Cached,
            None,
        ));
    }

    if live {
        let today = now().date_naive();
        match state
            .store()
            .db
            .consume_quota(source.id(), today, state.daily_limit(source.id()))
        {
            Ok(QuotaOutcome::Granted { .. }) => {}
            Ok(QuotaOutcome::Exhausted { reset_on, .. }) => {
                if let Some(stale) = state.store().search_cache.get_stale(&key) {
                    return Ok(response_from_snapshot(
                        &query,
                        &route,
                        stale,
                        source.data_card_id(),
                        source.id(),
                        Freshness::Stale,
                        Some(
                            "The daily source quota is spent; showing the last good result.".into(),
                        ),
                    ));
                }
                return Err(ApiError::quota_exhausted(
                    "The daily source quota is spent. Try again tomorrow.",
                    reset_on.format("%Y-%m-%d").to_string(),
                ));
            }
            Err(error) => {
                return Err(ApiError::internal(format!("quota ledger: {error}")));
            }
        }
    }

    let resolved = ResolvedQuery {
        request: &query,
        origin,
        destination,
    };

    // Beside Kiwi.com, ask Skiplagged for the same flights at the same time; the
    // merge keeps the cheaper seller for a service both list.
    let companion = match state.skiplagged() {
        Some(skiplagged) if source.id() == "kiwi" => {
            let today = now().date_naive();
            match state.store().db.consume_quota(
                "skiplagged",
                today,
                state.daily_limit("skiplagged"),
            ) {
                Ok(QuotaOutcome::Granted { .. }) => Some(skiplagged),
                _ => None,
            }
        }
        _ => None,
    };
    let (primary, secondary) = tokio::join!(source.search(&resolved), async {
        match companion {
            Some(skiplagged) => Some(skiplagged.search(&resolved).await),
            None => None,
        }
    });

    let mut source_notes: Vec<String> = Vec::new();
    let mut degraded_reason: Option<String> = None;
    let offers = match (primary, secondary) {
        (Ok(mut offers), secondary) => {
            match secondary {
                Some(Ok(extra)) => {
                    if !extra.is_empty() {
                        source_notes.push(format!(
                            "Also priced on Skiplagged: {} standard nonstop fare{}.",
                            extra.len(),
                            if extra.len() == 1 { "" } else { "s" }
                        ));
                    }
                    offers.extend(extra);
                }
                Some(Err(error)) => {
                    source_notes.push(format!("Skiplagged was not available ({error})."));
                }
                None => {}
            }
            offers
        }
        (Err(error), Some(Ok(extra))) if !extra.is_empty() => {
            degraded_reason = Some(format!(
                "Kiwi.com failed ({error}); showing Skiplagged's nonstop fares only."
            ));
            extra
        }
        (Err(error), _) => {
            if let Some(stale) = state.store().search_cache.get_stale(&key) {
                return Ok(response_from_snapshot(
                    &query,
                    &route,
                    stale,
                    source.data_card_id(),
                    source.id(),
                    Freshness::Stale,
                    Some(format!(
                        "The live source failed ({error}); showing the last good result."
                    )),
                ));
            }
            return Err(match error {
                SourceError::InvalidRequest(message) => {
                    ApiError::bad_request("source_rejected", message)
                }
                SourceError::NotConfigured(message) => {
                    ApiError::bad_request("source_not_configured", message)
                }
                SourceError::Unavailable(message) => ApiError::unavailable(message),
                SourceError::Parse(message) => {
                    ApiError::new(StatusCode::BAD_GATEWAY, "source_bad_response", message)
                }
            });
        }
    };

    let (mut itineraries, dropped) = build_itineraries(offers, &query);
    for itinerary in &mut itineraries {
        for pair in itinerary.segments.windows(2) {
            let connection = &pair[0].to;
            let Some(mct) = access_for(&connection.code).and_then(|access| access.mct) else {
                continue;
            };
            let Some(layover) =
                faredesk_core::layover_minutes(&pair[0].arrive_local, &pair[1].depart_local)
            else {
                continue;
            };
            let required = i64::from(mct.international);
            if layover < required {
                itinerary.connection_warnings.push(format!(
                    "Layover {layover} min at {} is below the {required} min minimum connection time.",
                    connection.code
                ));
            }
        }
    }

    if live {
        let today = now().date_naive();
        for itinerary in &itineraries {
            let _ = state.store().db.record_daily_minimum(
                &route,
                query.depart_date,
                today,
                itinerary.total_price,
                &itinerary.currency,
                &itinerary.source,
                &itinerary.id,
            );
        }
    }

    let mut notes = notes_for(&query, live, dropped, &itineraries);
    notes.extend(source_notes);
    let observed_at = now();
    let ttl = state.config().search_cache_ttl;
    let snapshot = SearchSnapshot {
        itineraries: itineraries.clone(),
        source: source.id().to_string(),
        observed_at,
        sample_size: itineraries.len(),
        notes: notes.clone(),
    };
    state.store().search_cache.insert(key, snapshot, ttl);

    let mut provenance = base_provenance(source.data_card_id(), source.id(), itineraries.len());
    provenance.freshness = if live {
        Freshness::Live
    } else {
        Freshness::Fixture
    };
    provenance.observed_at = observed_at;
    provenance.expires_at = Some(observed_at + ttl_duration(ttl));
    provenance.notes = notes;
    if let Some(reason) = degraded_reason {
        provenance.degraded = true;
        provenance.degraded_reason = Some(reason);
    }

    Ok(SearchResponse {
        query,
        route,
        itineraries,
        return_itineraries: Vec::new(),
        provenance,
    })
}

fn response_from_snapshot(
    query: &SearchQuery,
    route: &str,
    cached: Cached<SearchSnapshot>,
    card_id: &str,
    source_id: &str,
    freshness: Freshness,
    degraded_reason: Option<String>,
) -> SearchResponse {
    let mut provenance = base_provenance(card_id, source_id, cached.value.sample_size);
    provenance.freshness = freshness;
    provenance.observed_at = cached.value.observed_at;
    provenance.age_seconds = (now() - cached.value.observed_at).num_seconds().max(0) as u64;
    provenance.expires_at = Some(cached.expires_at);
    provenance.degraded = degraded_reason.is_some();
    provenance.degraded_reason = degraded_reason;
    provenance.notes = cached.value.notes.clone();

    SearchResponse {
        query: query.clone(),
        route: route.to_string(),
        itineraries: cached.value.itineraries,
        return_itineraries: Vec::new(),
        provenance,
    }
}

fn build_itineraries(
    offers: Vec<faredesk_core::Offer>,
    query: &SearchQuery,
) -> (Vec<Itinerary>, usize) {
    let mut merged: HashMap<String, Itinerary> = HashMap::new();
    let mut dropped = 0;

    for offer in offers {
        let id = fingerprint(&offer.segments);
        let (total, breakdown) = price_offer(&offer, query);
        let duration = offer.duration_minutes();
        let stops = offer.stops();
        let itinerary = Itinerary {
            id: id.clone(),
            segments: offer.segments,
            total_price: total,
            currency: offer.currency,
            total_duration_minutes: duration,
            stops,
            fare_family: offer.fare_family,
            bags_included: offer.bags_included,
            fare_rules: offer.fare_rules,
            provider_url: offer.provider_url,
            provider_name: offer.provider_name,
            source: offer.source,
            verified_at: offer.verified_at,
            risk: offer.risk,
            connection_warnings: Vec::new(),
            price_breakdown: breakdown,
            score: None,
        };

        match merged.get(&id) {
            Some(existing) if existing.total_price <= total => {
                dropped += 1;
            }
            Some(_) => {
                dropped += 1;
                merged.insert(id, itinerary);
            }
            None => {
                merged.insert(id, itinerary);
            }
        }
    }

    let mut itineraries: Vec<Itinerary> = merged.into_values().collect();
    apply_scores(&mut itineraries);
    sort_itineraries(&mut itineraries, query.sort);
    (itineraries, dropped)
}

fn notes_for(
    query: &SearchQuery,
    live: bool,
    dropped: usize,
    itineraries: &[Itinerary],
) -> Vec<String> {
    let mut notes = Vec::new();
    if !live {
        notes.push("Demo fixture prices; they do not vary with the date.".to_string());
    }
    if query.children > 0 {
        notes.push("Children are priced at the adult fare in this release.".to_string());
    }
    if dropped > 0 {
        notes.push(format!(
            "{dropped} duplicate service{} merged, keeping the lowest total.",
            if dropped == 1 { " was" } else { "s were" }
        ));
    }
    if itineraries
        .iter()
        .any(|itinerary| itinerary.price_breakdown.bag_fee_estimated)
    {
        notes.push(
            "At least one checked-bag fee is a Fare Desk estimate because the source did not name the amount."
                .to_string(),
        );
    }
    notes
}

// ---------------------------------------------------------------------------
// Seasonality, fare weather and Explore
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SeasonalityParams {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub async fn seasonality_feed(
    State(state): State<AppState>,
    Query(params): Query<SeasonalityParams>,
) -> Result<Json<SeasonalityResponse>, ApiError> {
    let (route, observations) = route_observations_for(&state, &params.from, &params.to)?;
    let points = seasonality(&observations);
    let included: usize = points.iter().map(|point| point.sample).sum();

    let mut provenance = base_provenance("fare-history", "fare-history", included);
    provenance.notes.push(
        "Median, p10 and p90 per travel month; months with fewer than five observations are omitted."
            .to_string(),
    );
    if points.is_empty() {
        provenance
            .notes
            .push("Not enough history yet to show a seasonal curve.".to_string());
    }

    Ok(Json(SeasonalityResponse {
        route,
        points,
        sample_size: observations.len(),
        provenance,
    }))
}

#[derive(Debug, Deserialize)]
pub struct FareWeatherParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<NaiveDate>,
}

pub async fn fare_weather_feed(
    State(state): State<AppState>,
    Query(params): Query<FareWeatherParams>,
) -> Result<Json<FareWeatherResponse>, ApiError> {
    let date = params.date.ok_or_else(|| {
        ApiError::bad_request(
            "missing_date",
            "Give the travel date, e.g. /api/fare-weather?from=BER&to=LIS&date=2027-03-01.",
        )
    })?;
    let (route, observations) = route_observations_for(&state, &params.from, &params.to)?;
    let mut weather = fare_weather(&observations, date, now().date_naive());
    weather.route = route;

    let mut provenance = base_provenance("fare-history", "fare-history", weather.sample_size);
    provenance.notes.push(if weather.sample_size < 5 {
        format!(
            "Collecting baseline — {} of 5 daily minima for this travel date.",
            weather.sample_size
        )
    } else {
        format!(
            "Volatility from {} daily minima; the range covers the last seven observations.",
            weather.sample_size
        )
    });

    Ok(Json(FareWeatherResponse {
        weather,
        provenance,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ExploreParams {
    pub home: Option<String>,
    pub month: Option<String>,
}

pub async fn explore(
    State(state): State<AppState>,
    Query(params): Query<ExploreParams>,
) -> Result<Json<ExploreResponse>, ApiError> {
    let home_value = params.home.unwrap_or_default();
    if home_value.trim().is_empty() {
        return Err(ApiError::bad_request(
            "missing_home",
            "Give a home airport, e.g. /api/explore?home=BER&month=2027-03.",
        ));
    }
    let month = params.month.unwrap_or_default().trim().to_string();
    let month_start =
        NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d").map_err(|_| {
            ApiError::bad_request("invalid_month", "month must be YYYY-MM, e.g. 2027-03.")
        })?;
    let travel_date = month_start + Duration::days(14);
    if travel_date < now().date_naive() {
        return Err(ApiError::bad_request(
            "month_in_past",
            "Explore only searches future dates; the 15th of that month has passed.",
        ));
    }

    let home = resolve_place(&state, &home_value)?;
    let source = state.source(None)?;
    let key = format!("explore:{}:{}:{}", home.code, month, source.id());

    if let Some(cached) = state.store().explore_cache.get(&key) {
        let mut provenance = base_provenance("explore-feed", source.id(), cached.value.rows.len());
        provenance.freshness = Freshness::Cached;
        provenance.observed_at = cached.value.generated_at;
        provenance.age_seconds = (now() - cached.value.generated_at).num_seconds().max(0) as u64;
        provenance.expires_at = Some(cached.expires_at);
        provenance
            .notes
            .push("Cached Explore feed (six-hour window).".to_string());
        return Ok(Json(ExploreResponse {
            home,
            month,
            rows: cached.value.rows,
            coverage: cached.value.coverage,
            provenance,
        }));
    }

    let mut destinations: Vec<Place> = state
        .store()
        .db
        .observed_destinations(&home.code)
        .map_err(|error| ApiError::internal(format!("route history: {error}")))?
        .into_iter()
        .filter_map(|(code, _)| state.store().reference.by_code(&code).cloned())
        .collect();
    destinations.sort_by(|a, b| a.code.cmp(&b.code));
    destinations.dedup_by(|a, b| a.code == b.code);
    destinations.truncate(12);

    let currency = currency_for_country(&home.country).to_string();
    let live = source.is_live();
    let mut rows: Vec<ExploreRow> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut attempted = 0;
    let mut quota_exhausted = false;

    for destination in &destinations {
        if live {
            match state.store().db.consume_quota(
                source.id(),
                now().date_naive(),
                state.daily_limit(source.id()),
            ) {
                Ok(QuotaOutcome::Granted { .. }) => {}
                Ok(QuotaOutcome::Exhausted { .. }) => {
                    quota_exhausted = true;
                    notes.push(
                        "The daily source quota was spent before the feed was complete."
                            .to_string(),
                    );
                    break;
                }
                Err(error) => return Err(ApiError::internal(format!("quota ledger: {error}"))),
            }
        }
        attempted += 1;

        let query = SearchQuery {
            origin: home.code.clone(),
            destination: destination.code.clone(),
            depart_date: travel_date,
            adults: 1,
            checked_bags: 0,
            carry_on_bags: 1,
            currency: currency.clone(),
            ..SearchQuery::default()
        };
        let resolved = ResolvedQuery {
            request: &query,
            origin: &home,
            destination,
        };

        let (total_price, percentile) = match source.search(&resolved).await {
            Ok(offers) => {
                let best = offers
                    .iter()
                    .map(|offer| price_offer(offer, &query).0)
                    .min_by(f64::total_cmp);
                match best {
                    Some(total) => {
                        let route = format!("{}-{}", home.code, destination.code);
                        let history = state
                            .store()
                            .db
                            .daily_minima(&route, travel_date)
                            .unwrap_or_default();
                        let percentile = if history.len() >= 5 {
                            let mut prices: Vec<f64> =
                                history.iter().map(|(_, price)| *price).collect();
                            prices.sort_by(f64::total_cmp);
                            percentile_rank(&prices, total)
                                .map(|value| (value * 10.0).round() / 10.0)
                        } else {
                            None
                        };
                        (Some((total * 100.0).round() / 100.0), percentile)
                    }
                    None => (None, None),
                }
            }
            Err(error) => {
                notes.push(format!("{}: {error}", destination.code));
                (None, None)
            }
        };

        rows.push(ExploreRow {
            destination: destination.clone(),
            travel_date,
            total_price,
            currency: currency.clone(),
            percentile,
            coverage: match (&total_price, live) {
                (None, _) => Coverage::None,
                (Some(_), true) => Coverage::Live,
                (Some(_), false) => Coverage::Fixture,
            },
            last_checked: Some(now()),
        });
    }

    rows.sort_by(|a, b| match (a.total_price, b.total_price) {
        (Some(left), Some(right)) => left.total_cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.destination.code.cmp(&b.destination.code),
    });
    rows.truncate(12);

    let priced = rows.iter().filter(|row| row.total_price.is_some()).count();
    let coverage = ExploreCoverage { priced, attempted };
    let generated_at = now();
    state.store().explore_cache.insert(
        key,
        ExploreCache {
            rows: rows.clone(),
            coverage,
            generated_at,
        },
        std::time::Duration::from_secs(6 * 60 * 60),
    );

    let mut provenance = base_provenance("explore-feed", source.id(), rows.len());
    provenance.freshness = if live {
        Freshness::Live
    } else {
        Freshness::Fixture
    };
    provenance.observed_at = generated_at;
    provenance.notes.push(
        "Destinations are the routes from this airport with recorded live fares; a schedule feed is required for wider coverage."
            .to_string(),
    );
    provenance
        .notes
        .push("Sampled on the 15th of the month, one adult, carry-on only.".to_string());
    if destinations.is_empty() {
        provenance
            .notes
            .push(format!("No route graph for {} yet.", home.code));
    }
    if quota_exhausted {
        provenance.degraded = true;
        provenance.degraded_reason =
            Some("The daily source quota was spent before the feed was complete.".to_string());
    }
    provenance.notes.extend(notes);

    Ok(Json(ExploreResponse {
        home,
        month,
        rows,
        coverage,
        provenance,
    }))
}

/// Resolve a from/to pair and load every observation for the route.
fn route_observations_for(
    state: &AppState,
    from: &Option<String>,
    to: &Option<String>,
) -> Result<(String, Vec<Observation>), ApiError> {
    let from = from.as_deref().unwrap_or_default();
    let to = to.as_deref().unwrap_or_default();
    if from.trim().is_empty() || to.trim().is_empty() {
        return Err(ApiError::bad_request(
            "missing_route",
            "Give from and to, e.g. ?from=BER&to=LIS.",
        ));
    }
    let origin = resolve_place(state, from)?;
    let destination = resolve_place(state, to)?;
    if origin.code == destination.code {
        return Err(ApiError::bad_request(
            "same_place",
            "Origin and destination are the same airport.",
        ));
    }
    let route = format!("{}-{}", origin.code, destination.code);
    let observations = state
        .store()
        .db
        .route_observations(&route)
        .map_err(|error| ApiError::internal(format!("fare history: {error}")))?
        .into_iter()
        .map(|(travel_date, observed_on, price)| Observation {
            travel_date,
            observed_on,
            price,
        })
        .collect();
    Ok((route, observations))
}

// ---------------------------------------------------------------------------
// Point of sale
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceCheckBody {
    pub origin: String,
    pub destination: String,
    pub depart_date: NaiveDate,
    pub itinerary_id: String,
    pub profile: EligibilityProfile,
    #[serde(default = "one")]
    pub adults: u8,
    #[serde(default)]
    pub checked_bags: u8,
    #[serde(default = "one")]
    pub carry_on_bags: u8,
    #[serde(default)]
    pub home_currency: Option<String>,
    #[serde(default)]
    pub fx_fee_percent: Option<f64>,
    #[serde(default)]
    pub source: Option<String>,
}

fn country_code(value: &str) -> bool {
    let value = value.trim();
    value.len() == 2
        && value
            .chars()
            .all(|character| character.is_ascii_alphabetic())
}

pub async fn price_check(
    State(state): State<AppState>,
    Json(body): Json<PriceCheckBody>,
) -> Result<Json<PriceCheckResponse>, ApiError> {
    if body.itinerary_id.trim().is_empty() {
        return Err(ApiError::bad_request(
            "missing_itinerary",
            "Give the itineraryId from a previous /api/search response.",
        ));
    }
    if !(1..=9).contains(&body.adults) {
        return Err(ApiError::bad_request(
            "invalid_party",
            "adults must be between 1 and 9.",
        ));
    }
    if body.checked_bags > 5 || body.carry_on_bags > 4 {
        return Err(ApiError::bad_request(
            "invalid_bags",
            "At most 5 checked bags and 4 carry-on bags per traveller.",
        ));
    }
    if !country_code(&body.profile.residency) || !country_code(&body.profile.nationality) {
        return Err(ApiError::bad_request(
            "invalid_profile",
            "residency and nationality must be two-letter ISO country codes.",
        ));
    }
    if body
        .profile
        .card_countries
        .iter()
        .any(|country| !country_code(country))
    {
        return Err(ApiError::bad_request(
            "invalid_profile",
            "cardCountries must be two-letter ISO country codes.",
        ));
    }

    let origin = resolve_place(&state, &body.origin)?;
    let destination = resolve_place(&state, &body.destination)?;
    if origin.code == destination.code {
        return Err(ApiError::bad_request(
            "same_place",
            "Origin and destination are the same airport.",
        ));
    }
    let source = state.source(body.source.as_deref())?;
    let route = format!("{}-{}", origin.code, destination.code);
    let home_market = origin.country.to_uppercase();
    let home_currency = body
        .home_currency
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase)
        .unwrap_or_else(|| currency_for_country(&home_market).to_string());
    let fx_fee = body.fx_fee_percent.unwrap_or(3.0).clamp(0.0, 20.0);
    let markets = screening_markets(&origin.country, &destination.country);
    let today = now().date_naive();

    let mut quotes: Vec<PosQuote> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut degraded: Option<String> = None;
    let mut quota_exhausted = false;

    for market in &markets {
        if source.is_live() {
            match state
                .store()
                .db
                .consume_quota(source.id(), today, state.daily_limit(source.id()))
            {
                Ok(QuotaOutcome::Granted { .. }) => {}
                Ok(QuotaOutcome::Exhausted { .. }) => {
                    quota_exhausted = true;
                    notes.push(format!(
                        "The daily source quota was spent before the {market} screen."
                    ));
                    break;
                }
                Err(error) => {
                    return Err(ApiError::internal(format!("quota ledger: {error}")));
                }
            }
        }

        let currency = currency_for_country(market);
        let query = SearchQuery {
            origin: origin.code.clone(),
            destination: destination.code.clone(),
            depart_date: body.depart_date,
            adults: body.adults,
            checked_bags: body.checked_bags,
            carry_on_bags: body.carry_on_bags,
            currency: currency.to_string(),
            ..SearchQuery::default()
        };
        let resolved = ResolvedQuery {
            request: &query,
            origin: &origin,
            destination: &destination,
        };

        let offers = match source.search_in_market(&resolved, market, currency).await {
            Ok(offers) => offers,
            Err(error) => {
                notes.push(format!("{market}: {error}"));
                continue;
            }
        };
        let Some(offer) = offers
            .into_iter()
            .find(|offer| fingerprint(&offer.segments) == body.itinerary_id)
        else {
            notes.push(format!("{market}: this itinerary is not listed."));
            continue;
        };

        let (_, breakdown) = price_offer(&offer, &query);
        let quote_currency = offer.currency.to_uppercase();
        let same_currency = quote_currency == home_currency;
        let (rate, fx_source, fx_degraded) = if same_currency {
            (1.0, "identity".to_string(), false)
        } else {
            match fx_rate_for(&state, &quote_currency, &home_currency).await {
                Some((rate, source, degraded)) => (rate, source, degraded),
                None => {
                    degraded.get_or_insert_with(|| {
                        format!(
                            "No FX rate for {quote_currency}/{home_currency}; net totals are in {quote_currency}."
                        )
                    });
                    (1.0, "unavailable".to_string(), true)
                }
            }
        };
        if fx_degraded && degraded.is_none() {
            degraded = Some(format!(
                "Live FX was unavailable for {quote_currency}/{home_currency}; a bundled rate was used."
            ));
        }

        let net = net_total(breakdown.fare, breakdown.bags, rate, fx_fee, same_currency);
        let fx_fee_amount =
            ((net - (breakdown.fare + breakdown.bags) * rate) * 100.0).round() / 100.0;

        let provider = state
            .store()
            .providers
            .iter()
            .find(|provider| provider.name.eq_ignore_ascii_case(&offer.provider_name));
        let payment_methods: Vec<String> = provider
            .map(|provider| provider.payment_methods.clone())
            .unwrap_or_else(|| vec!["card".to_string()]);
        let status = status_for(&body.profile, market, &payment_methods);
        let note = match status {
            PosStatus::Listed => Some(format!(
                "A card issued in {market} or a local billing address may be required."
            )),
            PosStatus::Ineligible => {
                Some("This provider's payment methods are not in your profile.".to_string())
            }
            PosStatus::Eligible => None,
        };
        let status_label = match status {
            PosStatus::Eligible => "eligible",
            PosStatus::Listed => "listed",
            PosStatus::Ineligible => "ineligible",
        };

        let _ = state.store().db.record_pos_quote(
            &route,
            body.depart_date,
            market,
            &quote_currency,
            breakdown.fare,
            breakdown.bags,
            fx_fee_amount,
            net,
            status_label,
            &offer.provider_name,
            today,
        );

        quotes.push(PosQuote {
            market: market.clone(),
            currency: quote_currency,
            fare: breakdown.fare,
            bags: breakdown.bags,
            fx_fee: fx_fee_amount,
            net_total: net,
            status,
            provider_name: offer.provider_name,
            provider_url: offer.provider_url,
            note,
        });

        if fx_source == "bundled-snapshot" {
            notes.push(format!("{market}: bundled FX snapshot used."));
        }
    }

    let Some(home_quote) = quotes.iter().find(|quote| quote.market == home_market) else {
        return Err(ApiError::not_found(
            "itinerary_not_found",
            if quotes.is_empty() {
                "The itinerary was not listed in any screened market; it may have sold out or changed."
            } else {
                "The itinerary was not listed in the departure market, so there is no home total to compare against."
            },
        ));
    };
    let home_net = home_quote.net_total;
    let threshold = 20.0_f64.max(home_net * 0.05);

    let mut persisted: HashSet<String> = HashSet::new();
    for quote in &quotes {
        if quote.market == home_market || quote.status != PosStatus::Eligible {
            continue;
        }
        if quote.net_total > home_net - threshold {
            continue;
        }
        let seen = state
            .store()
            .db
            .prior_pos_gap(
                &route,
                body.depart_date,
                &quote.market,
                home_net - threshold,
                today - Duration::days(7),
                today,
            )
            .unwrap_or(false);
        if seen {
            persisted.insert(quote.market.clone());
        }
    }
    let recommendation = recommend(&quotes, &home_market, home_net, &persisted, 20.0);

    let mut provenance = base_provenance("pos-quotes", source.id(), quotes.len());
    provenance.freshness = if source.is_live() {
        Freshness::Live
    } else {
        Freshness::Fixture
    };
    provenance
        .notes
        .push(format!("Screened markets: {}.", markets.join(", ")));
    provenance.notes.push(format!(
        "Net totals include the requested bags and a {fx_fee}% card FX fee when the quote is in another currency."
    ));
    provenance.notes.push(
        "A foreign storefront is recommended only above €20 or 5% and after the gap persists; a first check keeps the home storefront."
            .to_string(),
    );
    if !source.is_live() {
        provenance.notes.push(
            "Fixture source: storefronts return identical prices; this demonstrates the contract, not a market gap."
                .to_string(),
        );
    }
    if quota_exhausted {
        provenance.degraded = true;
        provenance.degraded_reason =
            Some("The daily source quota was spent partway through the screen; only some markets are shown.".to_string());
    }
    if let Some(reason) = degraded {
        provenance.degraded = true;
        provenance.degraded_reason = Some(reason);
    }
    provenance.notes.extend(notes);

    Ok(Json(PriceCheckResponse {
        route,
        itinerary_id: body.itinerary_id,
        home_market,
        home_currency,
        quotes,
        recommendation,
        provenance,
    }))
}

async fn fx_rate_for(state: &AppState, base: &str, quote: &str) -> Option<(f64, String, bool)> {
    if base.eq_ignore_ascii_case(quote) {
        return Some((1.0, "identity".to_string(), false));
    }
    let key = format!("fx:{base}:{quote}");
    if let Some(cached) = state.store().fx_cache.get(&key) {
        return Some((cached.value.rate, cached.value.source, false));
    }
    match state.reference_client().fx_rate(base, quote).await {
        Ok(rate) => {
            state
                .store()
                .fx_cache
                .insert(key, rate.clone(), state.config().reference_cache_ttl);
            Some((rate.rate, rate.source, false))
        }
        Err(_) => faredesk_sources::reference_data::fallback_fx(base, quote)
            .map(|rate| (rate.rate, rate.source, true)),
    }
}

// ---------------------------------------------------------------------------
// Reference data
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct FxParams {
    pub base: Option<String>,
    pub quote: Option<String>,
}

pub async fn fx(
    State(state): State<AppState>,
    Query(params): Query<FxParams>,
) -> Result<Json<FxResponse>, ApiError> {
    let base = params
        .base
        .unwrap_or_else(|| "EUR".into())
        .trim()
        .to_uppercase();
    let quote = params
        .quote
        .unwrap_or_else(|| "USD".into())
        .trim()
        .to_uppercase();
    for code in [&base, &quote] {
        if code.len() != 3
            || !code
                .chars()
                .all(|character| character.is_ascii_alphabetic())
        {
            return Err(ApiError::bad_request(
                "invalid_currency",
                "base and quote must be three-letter ISO codes.",
            ));
        }
    }

    let key = format!("fx:{base}:{quote}");
    if let Some(cached) = state.store().fx_cache.get(&key) {
        let mut provenance = base_provenance("fx-frankfurter", &cached.value.source, 1);
        provenance.freshness = Freshness::Cached;
        provenance.age_seconds = (now() - cached.stored_at).num_seconds().max(0) as u64;
        provenance.expires_at = Some(cached.expires_at);
        return Ok(Json(FxResponse {
            rate: cached.value,
            provenance,
        }));
    }

    let result = state.reference_client().fx_rate(&base, &quote).await;
    let ttl = state.config().reference_cache_ttl;

    match result {
        Ok(rate) => {
            state.store().fx_cache.insert(key, rate.clone(), ttl);
            let mut provenance = base_provenance("fx-frankfurter", &rate.source, 1);
            provenance.freshness = Freshness::Live;
            provenance.expires_at = Some(now() + ttl_duration(ttl));
            Ok(Json(FxResponse { rate, provenance }))
        }
        Err(error) => {
            if let Some(rate) = faredesk_sources::reference_data::fallback_fx(&base, &quote) {
                let mut provenance = base_provenance("fx-frankfurter", &rate.source, 1);
                provenance.degraded = true;
                provenance.degraded_reason = Some(format!(
                    "Live ECB rates are unavailable ({error}); using the bundled snapshot."
                ));
                return Ok(Json(FxResponse { rate, provenance }));
            }
            Err(ApiError::unavailable(format!(
                "No FX rate for {base}/{quote}: {error}"
            )))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HolidaysParams {
    pub country: Option<String>,
    pub year: Option<i32>,
}

pub async fn holidays(
    State(state): State<AppState>,
    Query(params): Query<HolidaysParams>,
) -> Result<Json<HolidaysResponse>, ApiError> {
    let country = params.country.unwrap_or_default().trim().to_uppercase();
    if country.len() != 2
        || !country
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        return Err(ApiError::bad_request(
            "invalid_country",
            "country must be a two-letter ISO code, e.g. DE.",
        ));
    }
    let year = params.year.unwrap_or_else(|| {
        now()
            .date_naive()
            .format("%Y")
            .to_string()
            .parse()
            .unwrap_or(2026)
    });
    if !(2020..=2100).contains(&year) {
        return Err(ApiError::bad_request(
            "invalid_year",
            "year must be between 2020 and 2100.",
        ));
    }

    let key = format!("holidays:{country}:{year}");
    if let Some(cached) = state.store().holidays_cache.get(&key) {
        let mut provenance = base_provenance("holidays-nager", "Nager.Date", cached.value.len());
        provenance.freshness = Freshness::Cached;
        provenance.age_seconds = (now() - cached.stored_at).num_seconds().max(0) as u64;
        provenance.expires_at = Some(cached.expires_at);
        return Ok(Json(HolidaysResponse {
            holidays: cached.value,
            provenance,
        }));
    }

    let ttl = state.config().reference_cache_ttl;
    match state.reference_client().holidays(&country, year).await {
        Ok(holidays) => {
            state
                .store()
                .holidays_cache
                .insert(key, holidays.clone(), ttl);
            let mut provenance = base_provenance("holidays-nager", "Nager.Date", holidays.len());
            provenance.freshness = Freshness::Live;
            provenance.expires_at = Some(now() + ttl_duration(ttl));
            Ok(Json(HolidaysResponse {
                holidays,
                provenance,
            }))
        }
        Err(error) => {
            if let Some(stale) = state.store().holidays_cache.get_stale(&key) {
                let mut provenance =
                    base_provenance("holidays-nager", "Nager.Date", stale.value.len());
                provenance.freshness = Freshness::Stale;
                provenance.age_seconds = (now() - stale.stored_at).num_seconds().max(0) as u64;
                provenance.degraded = true;
                provenance.degraded_reason = Some(format!(
                    "Public holidays could not be refreshed ({error}); showing the last good list."
                ));
                return Ok(Json(HolidaysResponse {
                    holidays: stale.value,
                    provenance,
                }));
            }
            let mut provenance = base_provenance("holidays-nager", "Nager.Date", 0);
            provenance.degraded = true;
            provenance.degraded_reason = Some(format!(
                "Public holidays are unavailable ({error}). The rest of the response is unaffected."
            ));
            Ok(Json(HolidaysResponse {
                holidays: Vec::<Holiday>::new(),
                provenance,
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProvidersParams {
    pub airport: Option<String>,
}

pub async fn providers(
    State(state): State<AppState>,
    Query(params): Query<ProvidersParams>,
) -> Result<Json<ProvidersResponse>, ApiError> {
    let providers: Vec<Provider> = state.store().providers.clone();
    let mut provenance = base_provenance("providers-registry", "registry", providers.len());

    if let Some(airport) = params
        .airport
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        let place = resolve_place(&state, airport)?;
        provenance.notes = vec![format!(
            "The registry is global; airline schedules are not joined to {} yet, so every provider is listed.",
            place.code
        )];
    }

    Ok(Json(ProvidersResponse {
        providers,
        provenance,
    }))
}

#[derive(Debug, Deserialize)]
pub struct DataCardParams {
    pub dataset: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataCardsResponse {
    pub cards: Vec<DataCard>,
    pub provenance: Provenance,
}

pub async fn data_cards(
    State(state): State<AppState>,
    Query(params): Query<DataCardParams>,
) -> Result<Json<DataCardsResponse>, ApiError> {
    let dataset = match params.dataset.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        None => None,
        Some(value) => Some(parse_dataset(value).ok_or_else(|| {
            ApiError::bad_request(
                "unknown_dataset",
                format!("Unknown dataset {value:?}. Use fares, airports, routes, fx, holidays, providers, history or meta."),
            )
        })?),
    };

    let cards: Vec<DataCard> = match dataset {
        Some(dataset) => state
            .store()
            .cards()
            .iter()
            .filter(|card| card.dataset == dataset)
            .cloned()
            .collect(),
        None => state.store().cards().to_vec(),
    };

    let mut provenance = base_provenance("data-cards-registry", "registry", cards.len());
    provenance.notes = vec![
        "Every data endpoint returns one of these ids so the source, licence and method can be audited."
            .to_string(),
    ];

    Ok(Json(DataCardsResponse { cards, provenance }))
}

// ---------------------------------------------------------------------------
// Rate limiting
// ---------------------------------------------------------------------------

pub async fn rate_limit(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let key = client_key(&request);
    match state.rate_limiter().check(&key, Instant::now()) {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            let mut response = ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Too many requests in a minute; try again shortly.",
            )
            .into_response();
            if let Ok(value) = retry_after.to_string().parse() {
                response.headers_mut().insert("retry-after", value);
            }
            response
        }
    }
}

fn client_key(request: &Request) -> String {
    for header in ["x-forwarded-for", "x-real-ip"] {
        if let Some(first) = request
            .headers()
            .get(header)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return first.to_string();
        }
    }
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| address.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[derive(Debug, Deserialize)]
pub struct ClimateParams {
    pub place: Option<String>,
}

pub async fn climate(
    State(state): State<AppState>,
    Query(params): Query<ClimateParams>,
) -> Result<Json<ClimateResponse>, ApiError> {
    let place_value = params.place.unwrap_or_default();
    if place_value.trim().is_empty() {
        return Err(ApiError::bad_request(
            "missing_place",
            "Give an airport, e.g. /api/reference/climate?place=BER.",
        ));
    }
    let place = resolve_place(&state, &place_value)?;
    let year = now().year() - 1;
    let key = format!("climate:{}:{year}", place.code);
    let ttl = std::time::Duration::from_secs(30 * 24 * 60 * 60);

    if let Some(cached) = state.store().climate_cache.get(&key) {
        let mut provenance =
            base_provenance("climate-open-meteo", "Open-Meteo", cached.value.len());
        provenance.freshness = Freshness::Cached;
        provenance.observed_at = cached.stored_at;
        provenance.age_seconds = (now() - cached.stored_at).num_seconds().max(0) as u64;
        provenance.expires_at = Some(cached.expires_at);
        provenance
            .notes
            .push(format!("Monthly normals for {year}, cached for 30 days."));
        return Ok(Json(ClimateResponse {
            place,
            year,
            months: cached.value,
            provenance,
        }));
    }

    match state
        .climate_client()
        .monthly_normals(place.lat, place.lng, year)
        .await
    {
        Ok(mut months) => {
            for month in &mut months {
                month.comfort = comfort(month.mean_high, month.mean_low, month.rain_days);
            }
            state.store().climate_cache.insert(key, months.clone(), ttl);
            let mut provenance = base_provenance("climate-open-meteo", "Open-Meteo", months.len());
            provenance.freshness = Freshness::Live;
            provenance.expires_at =
                Some(now() + Duration::from_std(ttl).unwrap_or_else(|_| Duration::zero()));
            provenance.notes.push(format!(
                "Monthly normals for {year} from daily temperatures and precipitation. Comfort is a temperature-and-rain score, not a forecast."
            ));
            Ok(Json(ClimateResponse {
                place,
                year,
                months,
                provenance,
            }))
        }
        Err(error) => {
            let mut provenance = base_provenance("climate-open-meteo", "Open-Meteo", 0);
            provenance.degraded = true;
            provenance.degraded_reason = Some(format!(
                "Climate normals are unavailable ({error}). The rest of the response is unaffected."
            ));
            Ok(Json(ClimateResponse {
                place,
                year,
                months: Vec::new(),
                provenance,
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AccessParams {
    pub airport: Option<String>,
}

pub async fn airport_access(
    State(state): State<AppState>,
    Query(params): Query<AccessParams>,
) -> Result<Json<AirportAccessResponse>, ApiError> {
    let airport_value = params.airport.unwrap_or_default();
    if airport_value.trim().is_empty() {
        return Err(ApiError::bad_request(
            "missing_airport",
            "Give an airport, e.g. /api/airport-access?airport=BER.",
        ));
    }
    let place = resolve_place(&state, &airport_value)?;
    let access = access_for(&place.code);

    let mut provenance = base_provenance(
        "airport-access",
        "curated",
        access
            .as_ref()
            .map(|value| value.transfers.len())
            .unwrap_or(0),
    );
    provenance.notes.push(access_note().to_string());
    if access.is_none() {
        provenance
            .notes
            .push(format!("No curated access data for {} yet.", place.code));
    }

    // A real driving leg from the curated city centre, cached for 30 days.
    let mut drive = None;
    if state.config().ground_routing {
        if let Some(center) = access.as_ref().and_then(|value| value.city_center) {
            let key = format!(
                "drive:{:.4},{:.4}:{:.4},{:.4}",
                center.lat, center.lng, place.lat, place.lng
            );
            let ttl = std::time::Duration::from_secs(30 * 24 * 60 * 60);
            if let Some(cached) = state.store().ground_cache.get(&key) {
                drive = Some(cached.value);
            } else {
                match state
                    .ground_client()
                    .drive((center.lat, center.lng), (place.lat, place.lng))
                    .await
                {
                    Ok(leg) => {
                        drive = Some(leg.clone());
                        state.store().ground_cache.insert(key, leg, ttl);
                    }
                    Err(error) => {
                        provenance.degraded = true;
                        provenance.degraded_reason = Some(format!(
                            "Live driving time is unavailable ({error}); curated transfers only."
                        ));
                    }
                }
            }
        }
    }

    // A real public-transit leg at a representative weekday morning.
    let mut transit = None;
    if state.config().transit_routing {
        if let Some(center) = access.as_ref().and_then(|value| value.city_center) {
            let depart_day = now().date_naive() + Duration::days(1);
            let depart_at = depart_day
                .and_hms_opt(9, 0, 0)
                .map(|time| DateTime::<Utc>::from_naive_utc_and_offset(time, Utc));
            if let Some(depart_at) = depart_at {
                let key = format!(
                    "transit:{:.4},{:.4}:{:.4},{:.4}:{}",
                    center.lat,
                    center.lng,
                    place.lat,
                    place.lng,
                    depart_day.format("%Y-%m-%d")
                );
                let ttl = std::time::Duration::from_secs(24 * 60 * 60);
                if let Some(cached) = state.store().transit_cache.get(&key) {
                    transit = Some(cached.value);
                } else {
                    match state
                        .transit_client()
                        .fastest_leg((center.lat, center.lng), (place.lat, place.lng), depart_at)
                        .await
                    {
                        Ok(leg) => {
                            transit = Some(leg.clone());
                            state.store().transit_cache.insert(key, leg, ttl);
                        }
                        Err(error) => {
                            provenance.degraded = true;
                            provenance.degraded_reason = Some(format!(
                                "Live transit routing is unavailable ({error}); curated transfers only."
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(Json(AirportAccessResponse {
        place,
        city_center: access.as_ref().and_then(|value| value.city_center),
        transfers: access
            .as_ref()
            .map(|value| value.transfers.clone())
            .unwrap_or_default(),
        mct: access.as_ref().and_then(|value| value.mct),
        drive,
        transit,
        provenance,
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoorToDoorParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<NaiveDate>,
    pub checked_bags: Option<u8>,
}

pub async fn door_to_door(
    State(state): State<AppState>,
    Query(params): Query<DoorToDoorParams>,
) -> Result<Json<DoorToDoorResponse>, ApiError> {
    let date = params.date.ok_or_else(|| {
        ApiError::bad_request(
            "missing_date",
            "Give the travel date, e.g. /api/door-to-door?from=BER&to=LIS&date=2027-03-01.",
        )
    })?;
    let origin = resolve_place(&state, params.from.as_deref().unwrap_or_default())?;
    let destination = resolve_place(&state, params.to.as_deref().unwrap_or_default())?;
    if origin.code == destination.code {
        return Err(ApiError::bad_request(
            "same_place",
            "Origin and destination are the same airport.",
        ));
    }
    let checked_bags = params.checked_bags.unwrap_or(0).min(5);
    let currency = currency_for_country(&origin.country).to_string();
    let source = state.source(None)?;
    let route = format!("{}-{}", origin.code, destination.code);

    if source.is_live()
        && !matches!(
            state.store().db.consume_quota(
                source.id(),
                now().date_naive(),
                state.daily_limit(source.id())
            ),
            Ok(QuotaOutcome::Granted { .. })
        )
    {
        return Err(ApiError::quota_exhausted(
            "The daily source quota is spent; try again tomorrow.",
            (now().date_naive() + Duration::days(1))
                .format("%Y-%m-%d")
                .to_string(),
        ));
    }

    let query = SearchQuery {
        origin: origin.code.clone(),
        destination: destination.code.clone(),
        depart_date: date,
        adults: 1,
        checked_bags,
        carry_on_bags: 1,
        currency: currency.clone(),
        sort: Sort::Cheapest,
        ..SearchQuery::default()
    };
    let resolved = ResolvedQuery {
        request: &query,
        origin: &origin,
        destination: &destination,
    };
    let offers = match source.search(&resolved).await {
        Ok(offers) => offers,
        Err(error) => {
            return Err(match error {
                SourceError::InvalidRequest(message) => {
                    ApiError::bad_request("source_rejected", message)
                }
                SourceError::NotConfigured(message) => {
                    ApiError::bad_request("source_not_configured", message)
                }
                SourceError::Unavailable(message) => ApiError::unavailable(message),
                SourceError::Parse(message) => {
                    ApiError::new(StatusCode::BAD_GATEWAY, "source_bad_response", message)
                }
            });
        }
    };
    let (mut itineraries, _) = build_itineraries(offers, &query);
    let Some(itinerary) = itineraries.drain(..).next() else {
        return Err(ApiError::not_found(
            "no_fare",
            "No priced itinerary was returned for this route and date.",
        ));
    };

    let mut notes: Vec<String> = Vec::new();
    let origin_leg = choose_transfer(&state, &origin.code, &currency, &mut notes).await;
    let destination_leg = choose_transfer(&state, &destination.code, &currency, &mut notes).await;
    let transfer_cost = origin_leg.as_ref().map(|leg| leg.cost).unwrap_or(0.0)
        + destination_leg.as_ref().map(|leg| leg.cost).unwrap_or(0.0);
    let transfer_minutes = origin_leg.as_ref().map(|leg| leg.minutes).unwrap_or(0)
        + destination_leg.as_ref().map(|leg| leg.minutes).unwrap_or(0);

    let fare = itinerary.total_price;
    let total = ((fare + transfer_cost) * 100.0).round() / 100.0;
    let flight_minutes = itinerary.total_duration_minutes;
    let total_minutes = flight_minutes + transfer_minutes;

    let mut provenance = base_provenance(source.data_card_id(), source.id(), 1);
    provenance.freshness = if source.is_live() {
        Freshness::Live
    } else {
        Freshness::Fixture
    };
    provenance.notes.push(
        "Door-to-door adds the cheapest curated transfer at each end to the party fare."
            .to_string(),
    );
    provenance.notes.push(
        "Transfer times and costs are curated estimates (airport-access card), not live routing."
            .to_string(),
    );
    provenance
        .notes
        .push("Reliability-adjusted pricing is not included yet.".to_string());
    provenance.notes.extend(notes);

    Ok(Json(DoorToDoorResponse {
        route,
        travel_date: date,
        currency,
        itinerary_id: itinerary.id,
        fare,
        transfer_cost: (transfer_cost * 100.0).round() / 100.0,
        total,
        flight_minutes,
        transfer_minutes,
        total_minutes,
        origin: origin_leg,
        destination: destination_leg,
        provenance,
    }))
}

/// The cheapest curated transfer to the city centre, converted to the
/// response currency. Missing data is a note, not an error.
async fn choose_transfer(
    state: &AppState,
    airport: &str,
    currency: &str,
    notes: &mut Vec<String>,
) -> Option<DoorToDoorLeg> {
    let access = access_for(airport)?;
    let transfer = access
        .transfers
        .iter()
        .min_by(|a, b| a.cost.total_cmp(&b.cost))?;
    let cost = if transfer.currency.eq_ignore_ascii_case(currency) {
        transfer.cost
    } else {
        match fx_rate_for(state, &transfer.currency, currency).await {
            Some((rate, _, degraded)) => {
                if degraded {
                    notes.push(format!(
                        "Bundled FX used for {} transfer in {}.",
                        airport, transfer.currency
                    ));
                }
                ((transfer.cost * rate) * 100.0).round() / 100.0
            }
            None => {
                notes.push(format!(
                    "No FX rate for {} transfer at {}; transfer cost omitted.",
                    transfer.currency, airport
                ));
                return None;
            }
        }
    };
    Some(DoorToDoorLeg {
        airport: airport.to_string(),
        mode: transfer.mode.clone(),
        minutes: transfer.minutes,
        cost,
        currency: currency.to_string(),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReliabilityParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub price: Option<f64>,
}

pub async fn reliability(
    State(state): State<AppState>,
    Query(params): Query<ReliabilityParams>,
) -> Result<Json<ReliabilityResponse>, ApiError> {
    let origin = resolve_place(&state, params.from.as_deref().unwrap_or_default())?;
    let destination = resolve_place(&state, params.to.as_deref().unwrap_or_default())?;
    if origin.code == destination.code {
        return Err(ApiError::bad_request(
            "same_place",
            "Origin and destination are the same airport.",
        ));
    }
    let route = format!("{}-{}", origin.code, destination.code);
    let aggregates = state
        .store()
        .db
        .reliability_for(&origin.code, &destination.code)
        .map_err(|error| ApiError::internal(format!("reliability store: {error}")))?;

    let mut carriers: Vec<CarrierReliability> = aggregates
        .into_iter()
        .filter_map(|aggregate| {
            let rate = on_time_rate(aggregate.on_time, aggregate.flights)?;
            Some(CarrierReliability {
                carrier: aggregate.carrier,
                flights: aggregate.flights,
                on_time: aggregate.on_time,
                on_time_rate: rate,
                adjusted_price: params.price.and_then(|price| adjusted_price(price, rate)),
                window: Some(ObservationWindow {
                    from: aggregate.first_date,
                    to: aggregate.last_date,
                    days: (aggregate.last_date - aggregate.first_date)
                        .num_days()
                        .max(0) as u32
                        + 1,
                }),
            })
        })
        .collect();
    carriers.sort_by(|a, b| match (a.adjusted_price, b.adjusted_price) {
        (Some(left), Some(right)) => left.total_cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => b.on_time_rate.total_cmp(&a.on_time_rate),
    });

    let flights: u32 = carriers.iter().map(|carrier| carrier.flights).sum();
    let mut provenance = base_provenance("reliability-bts", "BTS", flights as usize);
    provenance.notes.push(
        "US reporting carriers; cancellations and diversions are excluded. The share is flights arriving within 15 minutes of schedule."
            .to_string(),
    );
    if carriers.is_empty() {
        provenance.notes.push(format!(
            "No BTS reliability data for {route} yet. Ingest a month with faredesk-bts-ingest."
        ));
    }
    if let Some(price) = params.price {
        provenance.notes.push(format!(
            "Adjusted price = {price} divided by the carrier's on-time rate, in the fare's currency; rates below 50% are not priced."
        ));
    }

    Ok(Json(ReliabilityResponse {
        route,
        carriers,
        provenance,
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlertBody {
    pub email: String,
    pub origin: String,
    pub destination: String,
    pub travel_date: NaiveDate,
    #[serde(default)]
    pub threshold: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertWatch {
    pub id: i64,
    pub email: String,
    pub route: String,
    pub travel_date: NaiveDate,
    pub threshold: Option<f64>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertView {
    pub id: i64,
    pub route: String,
    pub travel_date: NaiveDate,
    pub price: f64,
    pub kind: String,
    pub created_at: String,
    pub delivered_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertsResponse {
    pub watches: Vec<AlertWatch>,
    pub alerts: Vec<AlertView>,
}

fn watch_view(watch: faredesk_store::Watch) -> AlertWatch {
    AlertWatch {
        id: watch.id,
        email: watch.email,
        route: watch.route,
        travel_date: watch.travel_date,
        threshold: watch.threshold,
        created_at: watch.created_at,
    }
}

fn alert_view(alert: faredesk_store::AlertRow) -> AlertView {
    AlertView {
        id: alert.id,
        route: alert.route,
        travel_date: alert.travel_date,
        price: alert.price,
        kind: alert.kind,
        created_at: alert.created_at,
        delivered_at: alert.delivered_at,
    }
}

pub async fn create_alert(
    State(state): State<AppState>,
    Json(body): Json<CreateAlertBody>,
) -> Result<Json<AlertWatch>, ApiError> {
    let email = body.email.trim().to_lowercase();
    if !email.contains('@') || email.len() > 254 {
        return Err(ApiError::bad_request(
            "invalid_email",
            "Give a valid email address.",
        ));
    }
    if body.travel_date < now().date_naive() {
        return Err(ApiError::bad_request(
            "date_in_past",
            "The travel date has already passed.",
        ));
    }
    if body.threshold.is_some_and(|threshold| threshold <= 0.0) {
        return Err(ApiError::bad_request(
            "invalid_threshold",
            "threshold must be greater than zero.",
        ));
    }
    let origin = resolve_place(&state, &body.origin)?;
    let destination = resolve_place(&state, &body.destination)?;
    if origin.code == destination.code {
        return Err(ApiError::bad_request(
            "same_place",
            "Origin and destination are the same airport.",
        ));
    }
    let route = format!("{}-{}", origin.code, destination.code);
    let id = state
        .store()
        .db
        .create_watch(
            &email,
            &route,
            body.travel_date,
            body.threshold,
            &now().to_rfc3339(),
        )
        .map_err(|error| ApiError::internal(format!("watch store: {error}")))?;

    Ok(Json(AlertWatch {
        id,
        email,
        route,
        travel_date: body.travel_date,
        threshold: body.threshold,
        created_at: now().to_rfc3339(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct AlertsParams {
    pub email: Option<String>,
}

pub async fn list_alerts(
    State(state): State<AppState>,
    Query(params): Query<AlertsParams>,
) -> Result<Json<AlertsResponse>, ApiError> {
    let email = params.email.unwrap_or_default().trim().to_lowercase();
    if email.is_empty() {
        return Err(ApiError::bad_request(
            "missing_email",
            "Give the email the watches belong to, e.g. /api/alerts?email=you@example.com.",
        ));
    }
    let watches = state
        .store()
        .db
        .list_watches(&email)
        .map_err(|error| ApiError::internal(format!("watch store: {error}")))?
        .into_iter()
        .map(watch_view)
        .collect();
    let alerts = state
        .store()
        .db
        .alerts_for_email(&email, 50)
        .map_err(|error| ApiError::internal(format!("alert store: {error}")))?
        .into_iter()
        .map(alert_view)
        .collect();

    Ok(Json(AlertsResponse { watches, alerts }))
}

#[derive(Debug, Deserialize)]
pub struct DeleteAlertParams {
    pub email: Option<String>,
}

pub async fn delete_alert(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<DeleteAlertParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let email = params.email.unwrap_or_default().trim().to_lowercase();
    if email.is_empty() {
        return Err(ApiError::bad_request(
            "missing_email",
            "Give the email the watch belongs to.",
        ));
    }
    let deleted = state
        .store()
        .db
        .delete_watch(id, &email)
        .map_err(|error| ApiError::internal(format!("watch store: {error}")))?;
    if !deleted {
        return Err(ApiError::not_found(
            "watch_not_found",
            "No active watch with that id for this email.",
        ));
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

fn parse_dataset(value: &str) -> Option<Dataset> {
    match value.trim().to_ascii_lowercase().as_str() {
        "fares" => Some(Dataset::Fares),
        "airports" => Some(Dataset::Airports),
        "routes" => Some(Dataset::Routes),
        "fx" => Some(Dataset::Fx),
        "holidays" => Some(Dataset::Holidays),
        "providers" => Some(Dataset::Providers),
        "history" => Some(Dataset::History),
        "climate" => Some(Dataset::Climate),
        "access" => Some(Dataset::Access),
        "reliability" => Some(Dataset::Reliability),
        "meta" => Some(Dataset::Meta),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Flex calendar
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct FlexCalendarParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<NaiveDate>,
}

/// The cheapest one-way fare per departure day around a date, from Skiplagged.
pub async fn flex_calendar(
    State(state): State<AppState>,
    Query(params): Query<FlexCalendarParams>,
) -> Result<Json<faredesk_core::FlexCalendarResponse>, ApiError> {
    let (Some(from), Some(to), Some(date)) = (params.from, params.to, params.date) else {
        return Err(ApiError::bad_request(
            "missing_query",
            "Give from, to and date, e.g. /api/flex-calendar?from=JFK&to=LAX&date=2027-01-15.",
        ));
    };
    let origin = resolve_place(&state, &from)?;
    let destination = resolve_place(&state, &to)?;
    let skiplagged = state.skiplagged().ok_or_else(|| {
        ApiError::bad_request(
            "source_not_configured",
            "The flex calendar needs Skiplagged, which is turned off (SKIPLAGGED=off).",
        )
    })?;

    let key = format!("flex:{}:{}:{date}", origin.code, destination.code);
    let mut provenance = base_provenance("fares-skiplagged", "skiplagged", 0);
    provenance.notes.push(
        "Indicative: Skiplagged's cheapest fare per day across all its fare types, which can include hidden-city tickets that search excludes. Search a day to see bookable fares."
            .to_string(),
    );

    let days = if let Some(cached) = state.store().flex_cache.get(&key) {
        provenance.freshness = Freshness::Cached;
        cached.value
    } else {
        match state.store().db.consume_quota(
            "skiplagged",
            now().date_naive(),
            state.daily_limit("skiplagged"),
        ) {
            Ok(QuotaOutcome::Granted { .. }) => {}
            Ok(QuotaOutcome::Exhausted { reset_on, .. }) => {
                return Err(ApiError::quota_exhausted(
                    "The daily Skiplagged cap is reached. Try again tomorrow.",
                    reset_on.format("%Y-%m-%d").to_string(),
                ));
            }
            Err(error) => return Err(ApiError::internal(format!("quota ledger: {error}"))),
        }
        let days = skiplagged
            .flex_calendar(&origin.code, &destination.code, date)
            .await
            .map_err(|error| match error {
                SourceError::InvalidRequest(message) => {
                    ApiError::bad_request("source_rejected", message)
                }
                other => ApiError::unavailable(other.to_string()),
            })?;
        // Six hours: the calendar is for picking a day, not for quoting.
        state.store().flex_cache.insert(
            key,
            days.clone(),
            std::time::Duration::from_secs(6 * 60 * 60),
        );
        provenance.freshness = Freshness::Live;
        days
    };
    provenance.sample_size = days.len();

    Ok(Json(faredesk_core::FlexCalendarResponse {
        origin,
        destination,
        currency: "USD".to_string(),
        days,
        provenance,
    }))
}
