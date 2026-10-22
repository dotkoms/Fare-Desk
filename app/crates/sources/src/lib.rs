//! Fare sources and reference-data clients.

pub mod fixtures;
pub mod reference_data;
pub mod serpapi;

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
    Fixtures(fixtures::FixtureSource),
    SerpApi(serpapi::SerpApiSource),
}

impl Source {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Fixtures(source) => source.id(),
            Self::SerpApi(source) => source.id(),
        }
    }

    pub fn data_card_id(&self) -> &'static str {
        match self {
            Self::Fixtures(source) => source.data_card_id(),
            Self::SerpApi(source) => source.data_card_id(),
        }
    }

    pub fn is_live(&self) -> bool {
        matches!(self, Self::SerpApi(_))
    }

    pub async fn search(&self, query: &ResolvedQuery<'_>) -> Result<Vec<Offer>, SourceError> {
        match self {
            Self::Fixtures(source) => source.search(query),
            Self::SerpApi(source) => source.search(query).await,
        }
    }
}
