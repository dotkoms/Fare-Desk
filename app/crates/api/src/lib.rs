//! Fare Desk HTTP API.

pub mod config;
pub mod error;
mod handlers;
mod ratelimit;

use std::sync::Arc;

use axum::http::{HeaderValue, Method};
use axum::routing::{delete, get, post};
use axum::Router;
use chrono::{DateTime, Utc};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use faredesk_core::SearchQuery;
use faredesk_sources::climate::ClimateClient;
use faredesk_sources::ground::GroundClient;
use faredesk_sources::kiwi::KiwiSource;
use faredesk_sources::reference_data::ReferenceClient;
use faredesk_sources::serpapi::SerpApiSource;
use faredesk_sources::transit::TransitClient;
use faredesk_sources::Source;
use faredesk_store::{Store, StoreError};

pub use config::Config;
pub use error::ApiError;
pub use ratelimit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    store: Store,
    config: Config,
    fixtures: Option<Arc<Source>>,
    kiwi: Option<Arc<Source>>,
    serpapi: Option<Arc<Source>>,
    reference_client: ReferenceClient,
    climate_client: ClimateClient,
    ground_client: GroundClient,
    transit_client: TransitClient,
    limiter: RateLimiter,
    started_at: DateTime<Utc>,
}

impl AppState {
    pub fn new(config: Config) -> Result<Self, StoreError> {
        let store = Store::open(&config.database_path)?;
        Self::from_store(config, store)
    }

    /// In-memory state for tests.
    pub fn in_memory(config: Config) -> Result<Self, StoreError> {
        let store = Store::in_memory()?;
        Self::from_store(config, store)
    }

    /// Build a state with caller-supplied sources. Used by the API tests to
    /// simulate an unavailable source without reaching the network.
    pub fn in_memory_with_sources(
        config: Config,
        fixtures: Option<Source>,
        serpapi: Option<Source>,
    ) -> Result<Self, StoreError> {
        let store = Store::in_memory()?;
        let limiter = RateLimiter::new(config.rate_limit_per_minute);
        Ok(Self {
            inner: Arc::new(Inner {
                store,
                config,
                fixtures: fixtures.map(Arc::new),
                kiwi: None,
                serpapi: serpapi.map(Arc::new),
                reference_client: ReferenceClient::new(),
                climate_client: ClimateClient::new(),
                ground_client: GroundClient::new(),
                transit_client: TransitClient::new(),
                limiter,
                started_at: Utc::now(),
            }),
        })
    }

    fn from_store(config: Config, store: Store) -> Result<Self, StoreError> {
        let places = store.reference.place_map();
        let providers = store.providers.clone();
        let client = reqwest_client();

        // Present only in test builds: the server binary has no demo fares.
        let fixtures =
            faredesk_sources::fixture_source(places.clone(), providers.clone()).map(Arc::new);
        let kiwi = config.kiwi_enabled.then(|| {
            let mut source = KiwiSource::new(client.clone(), places.clone());
            if let Some(url) = &config.kiwi_url {
                source = source.with_url(url);
            }
            Arc::new(Source::Kiwi(source))
        });
        let serpapi = config.serpapi_key.as_ref().map(|key| {
            let mut source = SerpApiSource::new(client.clone(), key, providers, places);
            if let Some(base_url) = &config.serpapi_base_url {
                source = source.with_base_url(base_url);
            }
            Arc::new(Source::SerpApi(source))
        });

        let limiter = RateLimiter::new(config.rate_limit_per_minute);
        Ok(Self {
            inner: Arc::new(Inner {
                store,
                config,
                fixtures,
                kiwi,
                serpapi,
                reference_client: ReferenceClient::new(),
                climate_client: ClimateClient::new(),
                ground_client: GroundClient::new(),
                transit_client: TransitClient::new(),
                limiter,
                started_at: Utc::now(),
            }),
        })
    }

    pub fn store(&self) -> &Store {
        &self.inner.store
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    /// The test-only demo source; always `None` in a server build.
    pub fn fixtures(&self) -> Option<&Source> {
        self.inner.fixtures.as_deref()
    }

    pub fn serpapi(&self) -> Option<&Source> {
        self.inner.serpapi.as_deref()
    }

    pub fn kiwi(&self) -> Option<&Source> {
        self.inner.kiwi.as_deref()
    }

    /// True when some live fare source can answer now.
    pub fn live_available(&self) -> bool {
        self.inner.kiwi.is_some() || self.inner.serpapi.is_some()
    }

    /// The daily search cap for a source: Kiwi's politeness cap or the SerpAPI quota.
    pub fn daily_limit(&self, source_id: &str) -> u32 {
        match source_id {
            "kiwi" => self.inner.config.kiwi_daily_limit,
            _ => self.inner.config.serpapi_daily_limit,
        }
    }

    pub fn reference_client(&self) -> &ReferenceClient {
        &self.inner.reference_client
    }

    pub fn climate_client(&self) -> &ClimateClient {
        &self.inner.climate_client
    }

    pub fn ground_client(&self) -> &GroundClient {
        &self.inner.ground_client
    }

    pub fn transit_client(&self) -> &TransitClient {
        &self.inner.transit_client
    }

    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.inner.limiter
    }

    pub fn started_at(&self) -> DateTime<Utc> {
        self.inner.started_at
    }

    /// Resolve the requested source. `None` or `auto` means Kiwi.com, which needs no
    /// key; SerpAPI only when asked for by name and configured.
    pub fn source(&self, requested: Option<&str>) -> Result<Arc<Source>, ApiError> {
        let not_configured =
            |message: &str| ApiError::bad_request("source_not_configured", message.to_string());
        match requested.map(str::trim) {
            None | Some("") | Some("auto") => self
                .inner
                .kiwi
                .clone()
                .or_else(|| self.inner.serpapi.clone())
                .or_else(|| self.inner.fixtures.clone())
                .ok_or_else(|| {
                    not_configured(
                        "No live fare source is enabled. Remove KIWI=off to search Kiwi.com.",
                    )
                }),
            Some("kiwi") => self
                .inner
                .kiwi
                .clone()
                .ok_or_else(|| not_configured("Kiwi.com search is turned off (KIWI=off).")),
            Some("serpapi") => self
                .inner
                .serpapi
                .clone()
                .ok_or_else(|| not_configured("SerpAPI is not configured (SERPAPI_KEY).")),
            Some("fixtures") => self
                .inner
                .fixtures
                .clone()
                .ok_or_else(|| not_configured("Demo fares are not part of this build.")),
            Some(other) => Err(ApiError::bad_request(
                "unknown_source",
                format!("Unknown source {other:?}. Use auto, kiwi or serpapi."),
            )),
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AppState")
            .field("kiwi_enabled", &self.inner.kiwi.is_some())
            .field("serpapi_configured", &self.inner.serpapi.is_some())
            .field("started_at", &self.inner.started_at)
            .finish()
    }
}

fn reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("FareDesk/0.1 (+https://github.com/dotkoms/Fare-Desk)")
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .unwrap_or_default()
}

/// The cache key for one search. Every input that can change the answer is in
/// it, so a cached party of two never answers a party of one.
pub fn search_cache_key(
    source: &str,
    query: &SearchQuery,
    origin: &str,
    destination: &str,
) -> String {
    format!(
        "search:{source}:{origin}-{destination}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        query.depart_date,
        query
            .return_date
            .map(|date| date.to_string())
            .unwrap_or_else(|| "-".into()),
        query.adults,
        query.children,
        query.checked_bags,
        query.carry_on_bags,
        query.currency.to_uppercase(),
        cabin_key(query),
        query
            .max_stops
            .map(|stops| stops.to_string())
            .unwrap_or_else(|| "-".into()),
    )
}

fn cabin_key(query: &SearchQuery) -> &'static str {
    use faredesk_core::Cabin;
    match query.cabin {
        Cabin::Economy => "economy",
        Cabin::Premium => "premium",
        Cabin::Business => "business",
        Cabin::First => "first",
    }
}

pub fn app(state: AppState) -> Router {
    let cors = cors_layer(&state.inner.config.cors_origins);

    let api = Router::new()
        .route("/places", get(handlers::places))
        .route("/routes", get(handlers::routes))
        .route("/route-intel", get(handlers::route_intel))
        .route("/seasonality", get(handlers::seasonality_feed))
        .route("/fare-weather", get(handlers::fare_weather_feed))
        .route("/explore", get(handlers::explore))
        .route("/search", post(handlers::search))
        .route("/price-check", post(handlers::price_check))
        .route("/reference/fx", get(handlers::fx))
        .route("/reference/holidays", get(handlers::holidays))
        .route("/reference/providers", get(handlers::providers))
        .route("/reference/climate", get(handlers::climate))
        .route("/airport-access", get(handlers::airport_access))
        .route("/door-to-door", get(handlers::door_to_door))
        .route("/reliability", get(handlers::reliability))
        .route(
            "/alerts",
            get(handlers::list_alerts).post(handlers::create_alert),
        )
        .route("/alerts/{id}", delete(handlers::delete_alert))
        .route("/data-card", get(handlers::data_cards))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            handlers::rate_limit,
        ));

    Router::new()
        .route("/healthz", get(handlers::health))
        .nest("/api", api)
        .fallback(handlers::not_found)
        .layer(CatchPanicLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn cors_layer(origins: &[String]) -> CorsLayer {
    if origins.iter().any(|origin| origin == "*") {
        return CorsLayer::permissive();
    }
    let values: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();
    CorsLayer::new()
        .allow_origin(values)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
}

/// Run the HTTP server until shutdown.
pub async fn run(state: AppState) -> std::io::Result<()> {
    let port = state.inner.config.port;
    let sweeper = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5 * 60));
        loop {
            interval.tick().await;
            sweeper.store().sweep_cache();
        }
    });

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(
        port,
        serpapi = state.serpapi().is_some(),
        "faredesk api listening"
    );
    axum::serve(listener, app(state)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_changes_with_party_and_bags() {
        let mut query = SearchQuery::default();
        let a = search_cache_key("fixtures", &query, "BER", "LIS");
        query.adults = 2;
        let b = search_cache_key("fixtures", &query, "BER", "LIS");
        assert_ne!(a, b);
        query.adults = 1;
        query.checked_bags = 1;
        let c = search_cache_key("fixtures", &query, "BER", "LIS");
        assert_ne!(a, c);
    }
}
