use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// An airport, city or metro area. `id` is stable (`airport:BER`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub id: String,
    pub code: String,
    pub name: String,
    pub city: String,
    /// ISO 3166-1 alpha-2.
    pub country: String,
    pub country_name: String,
    pub lat: f64,
    pub lng: f64,
    /// IANA zone, e.g. `Europe/Berlin`.
    pub timezone: String,
    /// Groups multi-airport cities, e.g. `LON`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metro: Option<String>,
    pub scheduled_service: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Cabin {
    #[default]
    Economy,
    Premium,
    Business,
    First,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Risk {
    /// Two separate tickets; a missed connection is the traveller's problem.
    SelfTransfer,
    /// Only sold through a third-party OTA.
    OtaOnly,
    /// Hidden-city ticketing.
    HiddenCity,
}

impl Risk {
    pub fn premium(self) -> f64 {
        match self {
            Self::SelfTransfer => 40.0,
            Self::OtaOnly => 25.0,
            Self::HiddenCity => 80.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bags {
    pub carry_on: u8,
    pub checked: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub id: String,
    pub marketing_carrier: String,
    pub operating_carrier: String,
    pub service_number: String,
    pub from: Place,
    pub to: Place,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depart_terminal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrive_terminal: Option<String>,
    /// Local time with offset, e.g. `2027-03-01T07:10:00+01:00`.
    pub depart_local: String,
    pub arrive_local: String,
    pub duration_minutes: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aircraft: Option<String>,
    pub cabin: Cabin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fare_class: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FareRules {
    pub refundable: bool,
    pub changeable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_fee: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_fee: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceBreakdown {
    /// Sum of per-traveller fares before bags.
    pub fare: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxes_and_fees: Option<f64>,
    /// Money added for checked bags the fare does not include.
    pub bags: f64,
    /// True when the bag fee came from a curated estimate rather than the source.
    pub bag_fee_estimated: bool,
    pub per_traveller: f64,
    pub travellers: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub total: f64,
    pub price: f64,
    pub time_cost: f64,
    pub stop_penalty: f64,
    pub risk_premium: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Itinerary {
    pub id: String,
    pub segments: Vec<Segment>,
    /// Party total including requested bags.
    pub total_price: f64,
    pub currency: String,
    pub total_duration_minutes: u32,
    pub stops: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fare_family: Option<String>,
    pub bags_included: Bags,
    pub fare_rules: FareRules,
    pub provider_url: String,
    pub provider_name: String,
    pub source: String,
    /// Null when the source cannot confirm the price at read time.
    pub verified_at: Option<DateTime<Utc>>,
    pub risk: Vec<Risk>,
    pub price_breakdown: PriceBreakdown,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<Score>,
}

/// What a source returns before pricing and ranking. One offer is one
/// purchasable itinerary for one traveller, before bag add-ons.
#[derive(Debug, Clone, PartialEq)]
pub struct Offer {
    pub segments: Vec<Segment>,
    /// Door-to-door duration including layovers, when the source reports it.
    pub total_duration_minutes: Option<u32>,
    pub price_per_traveller: f64,
    pub currency: String,
    pub taxes_and_fees: Option<f64>,
    pub bags_included: Bags,
    /// Per checked bag, per traveller, when the source knows the fee.
    pub bag_fee_per_checked_bag: Option<f64>,
    pub fare_family: Option<String>,
    pub fare_rules: FareRules,
    pub provider_url: String,
    pub provider_name: String,
    pub source: String,
    pub verified_at: Option<DateTime<Utc>>,
    pub risk: Vec<Risk>,
    pub warnings: Vec<String>,
}

impl Offer {
    pub fn duration_minutes(&self) -> u32 {
        self.total_duration_minutes
            .unwrap_or_else(|| self.segments.iter().map(|s| s.duration_minutes).sum())
    }

    pub fn stops(&self) -> u8 {
        self.segments.len().saturating_sub(1) as u8
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    #[default]
    Best,
    Cheapest,
    Fastest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchQuery {
    /// IATA code or place id (`airport:BER`).
    pub origin: String,
    pub destination: String,
    pub depart_date: NaiveDate,
    pub return_date: Option<NaiveDate>,
    pub adults: u8,
    pub children: u8,
    /// Checked bags per traveller.
    pub checked_bags: u8,
    pub carry_on_bags: u8,
    pub currency: String,
    pub cabin: Cabin,
    pub max_stops: Option<u8>,
    pub sort: Sort,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            origin: String::new(),
            destination: String::new(),
            depart_date: NaiveDate::from_ymd_opt(2027, 3, 1).expect("valid default date"),
            return_date: None,
            adults: 1,
            children: 0,
            checked_bags: 0,
            carry_on_bags: 1,
            currency: "EUR".to_string(),
            cabin: Cabin::Economy,
            max_stops: None,
            sort: Sort::Best,
        }
    }
}

impl SearchQuery {
    pub fn travellers(&self) -> u8 {
        self.adults.saturating_add(self.children).max(1)
    }

    pub fn route_key(&self, origin: &str, destination: &str) -> String {
        format!("{origin}-{destination}")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EligibilityProfile {
    pub residency: String,
    pub nationality: String,
    #[serde(default)]
    pub card_countries: Vec<String>,
    #[serde(default)]
    pub payment_methods: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PosStatus {
    Eligible,
    Listed,
    Ineligible,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PosQuote {
    pub market: String,
    pub currency: String,
    pub fare: f64,
    pub bags: f64,
    pub fx_fee: f64,
    /// In the home currency, after conversion and the card FX fee.
    pub net_total: f64,
    pub status: PosStatus,
    pub provider_name: String,
    pub provider_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PosChoice {
    Home,
    Foreign,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PosRecommendation {
    pub choice: PosChoice,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    pub net_total: f64,
    pub saving: f64,
    pub persisted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceCheckResponse {
    pub route: String,
    pub itinerary_id: String,
    pub home_market: String,
    pub home_currency: String,
    pub quotes: Vec<PosQuote>,
    pub recommendation: PosRecommendation,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Recommendation {
    Buy,
    Wait,
    Watch,
    Insufficient,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPoint {
    pub date: NaiveDate,
    pub price: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationWindow {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub days: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub low: f64,
    pub high: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvePoint {
    pub days_out: u32,
    pub median: f64,
    pub p10: f64,
    pub p90: f64,
    pub sample: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backtest {
    pub cases: u32,
    pub wait_wins: u32,
    pub avg_saving: f64,
    pub worst_loss: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackRecord {
    pub correct: u32,
    pub cases: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteIntel {
    pub route: String,
    pub travel_date: NaiveDate,
    pub currency: String,
    pub current_price: Option<f64>,
    pub percentile: Option<f64>,
    pub median: Option<f64>,
    pub p10: Option<f64>,
    pub p90: Option<f64>,
    pub history: Vec<HistoryPoint>,
    pub sample_size: usize,
    pub window: Option<ObservationWindow>,
    pub recommendation: Option<Recommendation>,
    pub min_observations: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected7: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected21: Option<Range>,
    /// Percent chance the fare is below p10 in seven days if the traveller waits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downside: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub curve: Vec<CurvePoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtest: Option<Backtest>,
    /// Null until the engine has made calls it can score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_record: Option<TrackRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub reasons: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub change_my_mind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteIntelResponse {
    pub intel: RouteIntel,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Freshness {
    /// Bundled reference data (airports, providers, data cards).
    Bundled,
    /// Generated from the bundled fixture set.
    Fixture,
    /// Fetched from the source during this request.
    Live,
    /// Served from cache inside its TTL.
    Cached,
    /// Cache entry served because the source failed.
    Stale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    pub data_card_id: String,
    pub source: String,
    pub observed_at: DateTime<Utc>,
    pub freshness: Freshness,
    pub age_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    pub degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
    pub sample_size: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub query: SearchQuery,
    pub route: String,
    pub itineraries: Vec<Itinerary>,
    pub provenance: Provenance,
}

/// The part of a search response worth caching and replaying when a source
/// fails. Provenance is rebuilt by the API around it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSnapshot {
    pub itineraries: Vec<Itinerary>,
    pub source: String,
    pub observed_at: DateTime<Utc>,
    pub sample_size: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacesResponse {
    pub places: Vec<Place>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Coverage {
    /// A live source can answer this route now.
    Live,
    /// Only the bundled fixture set covers this route.
    Fixture,
    /// Known airport, no route data.
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSummary {
    pub origin: Place,
    pub destination: Place,
    pub coverage: Coverage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_observed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutesResponse {
    pub routes: Vec<RouteSummary>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FxRate {
    pub date: String,
    pub base: String,
    pub quote: String,
    pub rate: f64,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FxResponse {
    pub rate: FxRate,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Holiday {
    pub date: NaiveDate,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name: Option<String>,
    pub country_code: String,
    pub global: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolidaysResponse {
    pub holidays: Vec<Holiday>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Airline,
    Ota,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub kind: ProviderKind,
    pub country: String,
    pub currencies: Vec<String>,
    pub payment_methods: Vec<String>,
    /// Deep link template. `{origin}`, `{destination}`, `{date}` are substituted.
    pub deep_link_template: String,
    pub affiliate: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersResponse {
    pub providers: Vec<Provider>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dataset {
    Fares,
    Airports,
    Routes,
    Fx,
    Holidays,
    Providers,
    History,
    Pos,
    Meta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataCard {
    pub id: String,
    pub dataset: Dataset,
    pub source: String,
    pub licence: String,
    /// How the data is produced, in one sentence a reader can audit.
    pub method: String,
    pub updated_at: DateTime<Utc>,
    pub notes: Vec<String>,
}
