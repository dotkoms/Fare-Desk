//! Reference data, the TTL cache, the quota ledger and fare history.

pub mod access;
pub mod cache;
pub mod db;
pub mod reference;
pub mod registry;

use std::path::Path;

use chrono::{DateTime, Utc};
use faredesk_core::{
    ClimateMonth, DataCard, ExploreCache, FxRate, GroundLeg, Holiday, Provider, SearchSnapshot,
};

pub use access::{access_for, access_note, AirportAccess};
pub use cache::{Cached, TimedCache};
pub use db::{Db, DbError, QuotaOutcome};
pub use reference::{Reference, ReferenceError};
pub use registry::{data_cards, load_providers, RegistryError};

pub const CACHE_CAPACITY: usize = 512;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Reference(#[from] ReferenceError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    Db(#[from] DbError),
}

/// Everything the API needs to answer a request without holding a second copy
/// of the reference data.
#[derive(Debug)]
pub struct Store {
    pub reference: Reference,
    pub providers: Vec<Provider>,
    pub db: Db,
    pub search_cache: TimedCache<SearchSnapshot>,
    pub fx_cache: TimedCache<FxRate>,
    pub holidays_cache: TimedCache<Vec<Holiday>>,
    pub explore_cache: TimedCache<ExploreCache>,
    pub climate_cache: TimedCache<Vec<ClimateMonth>>,
    pub ground_cache: TimedCache<GroundLeg>,
    cards: Vec<DataCard>,
}

impl Store {
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let reference = Reference::load()?;
        let providers = load_providers()?;
        let cards = data_cards(reference.generated_at(), reference.source_summary());
        Ok(Self {
            reference,
            providers,
            db: Db::open(db_path)?,
            search_cache: TimedCache::new(CACHE_CAPACITY),
            fx_cache: TimedCache::new(CACHE_CAPACITY),
            holidays_cache: TimedCache::new(CACHE_CAPACITY),
            explore_cache: TimedCache::new(CACHE_CAPACITY),
            climate_cache: TimedCache::new(CACHE_CAPACITY),
            ground_cache: TimedCache::new(CACHE_CAPACITY),
            cards,
        })
    }

    pub fn in_memory() -> Result<Self, StoreError> {
        let reference = Reference::load()?;
        let providers = load_providers()?;
        let cards = data_cards(reference.generated_at(), reference.source_summary());
        Ok(Self {
            reference,
            providers,
            db: Db::open_in_memory()?,
            search_cache: TimedCache::new(CACHE_CAPACITY),
            fx_cache: TimedCache::new(CACHE_CAPACITY),
            holidays_cache: TimedCache::new(CACHE_CAPACITY),
            explore_cache: TimedCache::new(CACHE_CAPACITY),
            climate_cache: TimedCache::new(CACHE_CAPACITY),
            ground_cache: TimedCache::new(CACHE_CAPACITY),
            cards,
        })
    }

    pub fn cards(&self) -> &[DataCard] {
        &self.cards
    }

    pub fn card_by_id(&self, id: &str) -> Option<&DataCard> {
        self.cards.iter().find(|card| card.id == id)
    }

    pub fn cache_len(&self) -> usize {
        self.search_cache.len()
            + self.fx_cache.len()
            + self.holidays_cache.len()
            + self.explore_cache.len()
            + self.climate_cache.len()
            + self.ground_cache.len()
    }

    pub fn sweep_cache(&self) {
        self.search_cache.sweep();
        self.fx_cache.sweep();
        self.holidays_cache.sweep();
        self.explore_cache.sweep();
        self.climate_cache.sweep();
        self.ground_cache.sweep();
    }

    pub fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
