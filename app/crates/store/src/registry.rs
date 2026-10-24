use chrono::{DateTime, Utc};
use faredesk_core::{DataCard, Dataset, Provider};

const PROVIDERS_JSON: &str = include_str!("../../../data/providers.json");

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("provider registry is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn load_providers() -> Result<Vec<Provider>, RegistryError> {
    Ok(serde_json::from_str(PROVIDERS_JSON)?)
}

pub fn provider_for_name<'a>(providers: &'a [Provider], name: &str) -> Option<&'a Provider> {
    providers
        .iter()
        .find(|provider| provider.name.eq_ignore_ascii_case(name))
}

fn card(
    id: &str,
    dataset: Dataset,
    source: &str,
    licence: &str,
    method: &str,
    updated_at: DateTime<Utc>,
    notes: &[&str],
) -> DataCard {
    DataCard {
        id: id.to_string(),
        dataset,
        source: source.to_string(),
        licence: licence.to_string(),
        method: method.to_string(),
        updated_at,
        notes: notes.iter().map(|note| note.to_string()).collect(),
    }
}

/// Every dataset a client can see, with the method and licence behind it.
pub fn data_cards(airports_updated_at: DateTime<Utc>, airports_source: String) -> Vec<DataCard> {
    let release = DateTime::parse_from_rfc3339("2026-10-22T00:00:00Z")
        .expect("valid release date")
        .with_timezone(&Utc);
    vec![
        card(
            "airports-reference",
            Dataset::Airports,
            "OurAirports + mwgg/Airports",
            "Public domain + MIT",
            "Large and medium airports with scheduled service. IANA timezones joined by ICAO, then IATA. Airport coordinates come from OurAirports.",
            airports_updated_at,
            &[
                airports_source.as_str(),
                "Airports without a timezone are omitted rather than guessed.",
            ],
        ),
        card(
            "fares-kiwi",
            Dataset::Fares,
            "Kiwi.com flight search (official MCP server)",
            "Kiwi.com terms of use",
            "Live Kiwi.com results from the public MCP server Kiwi.com publishes for agents; no key. Prices are for the whole party with the requested checked and cabin bags included, and are fetched at search time.",
            release,
            &[
                "Protected connections only: Kiwi does not flag self-transfers per result, so they are not requested.",
                "Kiwi.com sells the ticket; fare rules are not provided and are confirmed on Kiwi.com.",
                "The app redirects to Kiwi.com; it does not book.",
            ],
        ),
        card(
            "fares-skiplagged",
            Dataset::Fares,
            "Skiplagged (official MCP server)",
            "Skiplagged terms of use",
            "Standard nonstop fares from Skiplagged's public MCP server, no key, merged with Kiwi.com results; and the flex calendar of the cheapest fare per day. Prices are for the whole party, fetched at search time.",
            release,
            &[
                "Hidden-city and self-transfer fares are excluded from search; connections are not shown because the source does not name the connection airport.",
                "Bag terms are not provided; requested checked bags use the estimate table and are marked estimated.",
                "The flex calendar is indicative and can include fare types that search excludes.",
            ],
        ),
        card(
            "fares-serpapi",
            Dataset::Fares,
            "SerpAPI Google Flights",
            "SerpAPI terms of service",
            "Live Google Flights results via SerpAPI, mapped to the itinerary contract. Prices are per traveller before bag add-ons and are confirmed at fetch time.",
            release,
            &[
                "Google is never scraped directly.",
                "The app redirects to the provider; it does not book.",
                "Bag fees that the source reports only as 'for a fee' use the curated estimate table and are marked estimated.",
            ],
        ),
        card(
            "route-coverage",
            Dataset::Routes,
            "First-party route graph",
            "GPL-3.0-or-later",
            "Origin-destination pairs the demo fixtures cover, plus whether a live fare source is configured. Coverage is a configuration fact, not a price observation.",
            release,
            &["Live coverage means the source is configured, not that a route was searched."],
        ),
        card(
            "fx-frankfurter",
            Dataset::Fx,
            "Frankfurter (ECB reference rates)",
            "ECB terms of use",
            "Daily ECB reference rates, cached for 24 hours. Falls back to a bundled snapshot when the service is unreachable.",
            release,
            &["Reference rates are for display and comparison, not settlement."],
        ),
        card(
            "holidays-nager",
            Dataset::Holidays,
            "Nager.Date",
            "MIT",
            "Public holidays by country and year, cached for 24 hours. No fallback dataset is bundled, so an outage returns a degraded empty list.",
            release,
            &[],
        ),
        card(
            "providers-registry",
            Dataset::Providers,
            "First-party registry",
            "GPL-3.0-or-later",
            "Curated airline and OTA deep-link templates. Templates are not partner-verified yet.",
            release,
            &["No affiliate relationships."],
        ),
        card(
            "pos-quotes",
            Dataset::Pos,
            "SerpAPI storefront quotes",
            "SerpAPI terms of service",
            "The same itinerary screened in the departure, destination, US and GB Google Flights country storefronts, converted to the home currency with the card FX fee. Fixtures return identical prices across markets.",
            release,
            &[
                "Only shortlisted itineraries are screened.",
                "Eligibility is a filter, not an identity: no card numbers, addresses or documents.",
                "A foreign storefront is only recommended after the gap persists.",
            ],
        ),
        card(
            "explore-feed",
            Dataset::Explore,
            "SerpAPI + demo route graph",
            "SerpAPI terms of service / GPL-3.0-or-later",
            "Cheapest destination per month from the home airport, one search per destination on the 15th of the month. Destinations come from the demo route graph until a schedule feed is added.",
            release,
            &[
                "One adult, carry-on only.",
                "Months with fewer than five observations are omitted, not guessed.",
            ],
        ),
        card(
            "climate-open-meteo",
            Dataset::Climate,
            "Open-Meteo",
            "CC BY 4.0",
            "Monthly normals from the last complete calendar year of daily temperatures and precipitation, aggregated per month. Comfort is a simple temperature-and-rain score, not a forecast.",
            release,
            &["Normals are historical; they do not predict a specific trip."],
        ),
        card(
            "airport-access",
            Dataset::Access,
            "Curated estimates",
            "GPL-3.0-or-later",
            "City-centre transfer estimates and minimum connection times for launch airports, plus live driving (OSRM over OpenStreetMap) and public-transit (Transitous) legs from the city centre. Curated rows are typical estimates; live legs are one representative departure.",
            release,
            &[
                "Times are typical, not live.",
                "Minimum connection times are guidance for a single ticket; verify with the operator.",
                "Driving legs use OSRM; transit legs use Transitous (GTFS feeds per operator).",
            ],
        ),
        card(
            "reliability-bts",
            Dataset::Reliability,
            "US Bureau of Transportation Statistics",
            "Public domain",
            "Monthly On-Time Performance zips, aggregated to the share of flights arriving within 15 minutes by route and reporting carrier. Cancellations and diversions are excluded.",
            release,
            &[
                "US reporting carriers only.",
                "Coverage depends on which monthly files have been ingested.",
            ],
        ),
        card(
            "data-cards-registry",
            Dataset::Meta,
            "First-party registry",
            "GPL-3.0-or-later",
            "Lists every dataset with its source, licence and method so a reader can audit what a response is built from.",
            release,
            &[],
        ),
        card(
            "fare-history",
            Dataset::History,
            "Fare Desk observations",
            "Fare Desk data licence",
            "Daily minimum price per route, travel date and physical service. Written from live searches until the scheduler lands.",
            release,
            &["Collection starts before launch so the buy/wait tab is not empty on day one."],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn providers_load() {
        let providers = load_providers().unwrap();
        assert!(providers.iter().any(|provider| provider.id == "tap"));
        assert!(providers
            .iter()
            .any(|provider| provider.id == "google-flights"));
        assert!(provider_for_name(&providers, "Ryanair").is_some());
    }

    #[test]
    fn cards_cover_every_dataset() {
        let cards = data_cards(Utc::now(), "test".into());
        for dataset in [
            Dataset::Airports,
            Dataset::Fares,
            Dataset::Routes,
            Dataset::Fx,
            Dataset::Holidays,
            Dataset::Providers,
            Dataset::History,
            Dataset::Pos,
            Dataset::Explore,
            Dataset::Climate,
            Dataset::Access,
            Dataset::Reliability,
            Dataset::Meta,
        ] {
            assert!(
                cards.iter().any(|card| card.dataset == dataset),
                "{dataset:?}"
            );
        }
    }
}
