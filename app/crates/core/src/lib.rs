//! Contract types shared by every Fare Desk source, the store and the API.
//!
//! The shapes follow `PLAN.md` §12. Anything that reaches a client carries
//! provenance: a data-card id, a source, an observation time and a freshness
//! state.

pub mod advice;
pub mod contract;
pub mod fingerprint;
pub mod geo;
pub mod pos;
pub mod pricing;
pub mod rank;
pub mod stats;

pub use advice::{advise, Advice, AdviceInput, Observation, MIN_OBSERVATIONS};
pub use contract::*;
pub use fingerprint::fingerprint;
pub use pos::{currency_for_country, net_total, recommend, screening_markets, status_for};
pub use pricing::price_offer;
pub use rank::{apply_scores, sort_itineraries};
pub use stats::{percentile, percentile_rank};
