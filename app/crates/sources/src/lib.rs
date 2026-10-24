//! Fare sources and reference-data clients.

pub mod climate;
/// Demo inventory for tests only; the server never serves it.
#[cfg(any(test, feature = "test-fixtures"))]
pub mod fixtures;
pub mod ground;
pub mod kiwi;
pub mod mcp;
pub mod reference_data;
pub mod serpapi;
pub mod skiplagged;
pub mod transit;

use chrono::NaiveDate;
use faredesk_core::{Offer, Place, SearchQuery};

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("source is not configured: {0}")]
    NotConfigured(String),
    #[error("source rejected the request: {0}")]
    InvalidRequest(String),
    #[error("source is unavailable: {0}")]
    Unavailable(String),
    #[error("could not parse the source response: {0}")]
    Parse(String),
}

impl SourceError {
    /// True when retrying later and using a cached answer makes sense.
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::Unavailable(_))
    }
}

/// A query after the origin and destination codes have been resolved.
#[derive(Debug, Clone)]
pub struct ResolvedQuery<'a> {
    pub request: &'a SearchQuery,
    pub origin: &'a Place,
    pub destination: &'a Place,
}

impl ResolvedQuery<'_> {
    pub fn route(&self) -> String {
        format!("{}-{}", self.origin.code, self.destination.code)
    }
}

/// The configured fare sources. An enum keeps the trait async without a boxed
/// future or an extra dependency.
pub enum Source {
    #[cfg(any(test, feature = "test-fixtures"))]
    Fixtures(fixtures::FixtureSource),
    Kiwi(kiwi::KiwiSource),
    SerpApi(serpapi::SerpApiSource),
}

impl Source {
    pub fn id(&self) -> &'static str {
        match self {
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::Fixtures(source) => source.id(),
            Self::Kiwi(source) => source.id(),
            Self::SerpApi(source) => source.id(),
        }
    }

    pub fn data_card_id(&self) -> &'static str {
        match self {
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::Fixtures(source) => source.data_card_id(),
            Self::Kiwi(source) => source.data_card_id(),
            Self::SerpApi(source) => source.data_card_id(),
        }
    }

    pub fn is_live(&self) -> bool {
        matches!(self, Self::Kiwi(_) | Self::SerpApi(_))
    }

    pub async fn search(&self, query: &ResolvedQuery<'_>) -> Result<Vec<Offer>, SourceError> {
        match self {
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::Fixtures(source) => source.search(query),
            Self::Kiwi(source) => source.search(query).await,
            Self::SerpApi(source) => source.search(query).await,
        }
    }

    /// The same search priced in a country storefront. Fixtures are
    /// market-agnostic and return their one price table.
    pub async fn search_in_market(
        &self,
        query: &ResolvedQuery<'_>,
        market: &str,
        currency: &str,
    ) -> Result<Vec<Offer>, SourceError> {
        match self {
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::Fixtures(source) => source.search(query),
            // Kiwi.com sells from one storefront; it cannot be priced per market.
            Self::Kiwi(source) => source.search(query).await,
            Self::SerpApi(source) => source.search_in_market(query, market, currency).await,
        }
    }
}

/// Fill a provider deep-link template (`{origin}`, `{destination}`, `{date}`, `{adults}`).
pub fn fill_template(
    template: &str,
    origin: &str,
    destination: &str,
    date: NaiveDate,
    adults: u8,
) -> String {
    template
        .replace("{origin}", origin)
        .replace("{destination}", destination)
        .replace("{date}", &date.format("%Y-%m-%d").to_string())
        .replace("{adults}", &adults.to_string())
}

/// The demo fare source, in test builds only. A server build returns `None`, so
/// it can only ever answer with live fares or say that none are configured.
#[cfg(any(test, feature = "test-fixtures"))]
pub fn fixture_source(
    places: std::collections::HashMap<String, Place>,
    providers: Vec<faredesk_core::Provider>,
) -> Option<Source> {
    Some(Source::Fixtures(fixtures::FixtureSource::new(
        places, providers,
    )))
}

#[cfg(not(any(test, feature = "test-fixtures")))]
pub fn fixture_source(
    _places: std::collections::HashMap<String, Place>,
    _providers: Vec<faredesk_core::Provider>,
) -> Option<Source> {
    None
}
