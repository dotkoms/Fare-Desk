//! Point-of-sale screening rules (docs/13).
//!
//! A storefront is a market (ISO country). The screen compares the same
//! physical itinerary priced in a few markets, converts each total to the
//! traveller's home currency including the card's FX fee, and labels each row
//! eligible, listed or ineligible. A foreign storefront is only recommended
//! when the gap clears the threshold, the row is eligible, and the gap has
//! persisted.

use std::collections::HashSet;

use crate::contract::{EligibilityProfile, PosChoice, PosQuote, PosRecommendation, PosStatus};

/// ISO country → local currency for the markets we screen. Fallback EUR.
const MARKET_CURRENCY: &[(&str, &str)] = &[
    ("AT", "EUR"),
    ("BE", "EUR"),
    ("BR", "BRL"),
    ("CA", "CAD"),
    ("CH", "CHF"),
    ("CZ", "CZK"),
    ("DE", "EUR"),
    ("DK", "DKK"),
    ("ES", "EUR"),
    ("FI", "EUR"),
    ("FR", "EUR"),
    ("GB", "GBP"),
    ("GR", "EUR"),
    ("HK", "HKD"),
    ("HU", "HUF"),
    ("IE", "EUR"),
    ("IT", "EUR"),
    ("JP", "JPY"),
    ("NL", "EUR"),
    ("NO", "NOK"),
    ("PL", "PLN"),
    ("PT", "EUR"),
    ("SE", "SEK"),
    ("SG", "SGD"),
    ("TR", "TRY"),
    ("US", "USD"),
    ("ZA", "ZAR"),
];

pub fn currency_for_country(country: &str) -> &'static str {
    MARKET_CURRENCY
        .iter()
        .find(|(code, _)| code.eq_ignore_ascii_case(country))
        .map(|(_, currency)| *currency)
        .unwrap_or("EUR")
}

/// Departure market, destination market, US and GB, deduplicated, at most five.
pub fn screening_markets(departure_country: &str, destination_country: &str) -> Vec<String> {
    let mut markets = Vec::new();
    for country in [departure_country, destination_country, "US", "GB", "IE"] {
        let market = country.trim().to_uppercase();
        if market.len() == 2 && !markets.contains(&market) {
            markets.push(market);
        }
        if markets.len() == 5 {
            break;
        }
    }
    markets
}

/// Net total in the home currency: fare and bags converted at the card rate
/// plus its FX fee. Same-currency totals carry no FX fee.
pub fn net_total(
    fare: f64,
    bags: f64,
    fx_rate: f64,
    fx_fee_percent: f64,
    same_currency: bool,
) -> f64 {
    let converted = (fare + bags) * fx_rate;
    let fee = if same_currency {
        0.0
    } else {
        converted * fx_fee_percent.clamp(0.0, 20.0) / 100.0
    };
    ((converted + fee) * 100.0).round() / 100.0
}

/// Eligible when the provider accepts a payment method the traveller has and
/// the market is one of their card countries. Listed when the price is visible
/// but a local card may be required. Ineligible when payment cannot be
/// completed at all.
pub fn status_for(
    profile: &EligibilityProfile,
    market: &str,
    provider_payment_methods: &[String],
) -> PosStatus {
    let can_pay = provider_payment_methods.is_empty()
        || provider_payment_methods.iter().any(|method| {
            profile
                .payment_methods
                .iter()
                .any(|owned| owned.eq_ignore_ascii_case(method))
        });
    if !can_pay {
        return PosStatus::Ineligible;
    }
    let market = market.to_uppercase();
    if profile
        .card_countries
        .iter()
        .any(|country| country.eq_ignore_ascii_case(&market))
    {
        PosStatus::Eligible
    } else {
        PosStatus::Listed
    }
}

/// A foreign row is recommended only above `max(20, 5%)` of the home total and
/// only when the gap has persisted. Otherwise the home storefront is the answer.
pub fn recommend(
    quotes: &[PosQuote],
    home_market: &str,
    home_net: f64,
    persisted_markets: &HashSet<String>,
    threshold_absolute: f64,
) -> PosRecommendation {
    let threshold = threshold_absolute.max(home_net * 0.05);
    let home = home_market.to_uppercase();

    let mut foreign: Vec<&PosQuote> = quotes
        .iter()
        .filter(|quote| quote.market != home && quote.status == PosStatus::Eligible)
        .collect();
    foreign.sort_by(|a, b| a.net_total.total_cmp(&b.net_total));
    let best = foreign.first().copied();

    match best {
        Some(candidate) if candidate.net_total <= home_net - threshold => {
            let saving = (home_net - candidate.net_total).round();
            if persisted_markets.contains(&candidate.market) {
                PosRecommendation {
                    choice: PosChoice::Foreign,
                    market: Some(candidate.market.clone()),
                    net_total: candidate.net_total,
                    saving,
                    persisted: true,
                    reason: format!(
                        "{} saves €{} and the gap has persisted; the fare rules are equivalent.",
                        candidate.market, saving
                    ),
                }
            } else {
                PosRecommendation {
                    choice: PosChoice::Home,
                    market: None,
                    net_total: home_net,
                    saving: 0.0,
                    persisted: false,
                    reason: format!(
                        "{} is €{} cheaper today, but the gap is new; check again in a few days. Your home storefront is the safe choice.",
                        candidate.market, saving
                    ),
                }
            }
        }
        Some(candidate) => PosRecommendation {
            choice: PosChoice::Home,
            market: None,
            net_total: home_net,
            saving: 0.0,
            persisted: persisted_markets.contains(&candidate.market),
            reason: format!(
                "{} is only €{} cheaper after bags and FX; below the €20 or 5% threshold, so your home storefront is the best total today.",
                candidate.market,
                (home_net - candidate.net_total).max(0.0).round()
            ),
        },
        None => PosRecommendation {
            choice: PosChoice::Home,
            market: None,
            net_total: home_net,
            saving: 0.0,
            persisted: false,
            reason: "No eligible foreign storefront beats your home total after bags and FX.".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(card_countries: &[&str]) -> EligibilityProfile {
        EligibilityProfile {
            residency: "DE".into(),
            nationality: "DE".into(),
            card_countries: card_countries.iter().map(|c| c.to_string()).collect(),
            payment_methods: vec!["card".into()],
        }
    }

    fn quote(market: &str, net: f64, status: PosStatus) -> PosQuote {
        PosQuote {
            market: market.into(),
            currency: "EUR".into(),
            fare: net,
            bags: 0.0,
            fx_fee: 0.0,
            net_total: net,
            status,
            provider_name: "Test".into(),
            provider_url: "https://example.test".into(),
            note: None,
        }
    }

    #[test]
    fn net_total_applies_the_card_fee_only_across_currencies() {
        assert_eq!(net_total(100.0, 20.0, 1.0, 3.0, true), 120.0);
        assert_eq!(net_total(100.0, 20.0, 1.1, 3.0, false), 135.96);
        assert_eq!(net_total(100.0, 0.0, 0.9, 0.0, false), 90.0);
    }

    #[test]
    fn status_uses_card_countries_and_payment_methods() {
        let card = vec!["card".to_string()];
        assert_eq!(
            status_for(&profile(&["DE"]), "DE", &card),
            PosStatus::Eligible
        );
        assert_eq!(
            status_for(&profile(&["DE"]), "PT", &card),
            PosStatus::Listed
        );
        assert_eq!(
            status_for(&profile(&["DE"]), "DE", &["ideal".to_string()]),
            PosStatus::Ineligible
        );
    }

    #[test]
    fn markets_are_deduplicated_and_capped() {
        assert_eq!(
            screening_markets("DE", "PT"),
            vec!["DE", "PT", "US", "GB", "IE"]
        );
        assert_eq!(screening_markets("US", "US"), vec!["US", "GB", "IE"]);
    }

    #[test]
    fn currency_map_falls_back_to_euro() {
        assert_eq!(currency_for_country("GB"), "GBP");
        assert_eq!(currency_for_country("US"), "USD");
        assert_eq!(currency_for_country("XX"), "EUR");
    }

    #[test]
    fn recommends_home_below_the_threshold() {
        let quotes = vec![
            quote("DE", 120.0, PosStatus::Eligible),
            quote("PT", 112.0, PosStatus::Eligible),
        ];
        let result = recommend(&quotes, "DE", 120.0, &HashSet::new(), 20.0);
        assert_eq!(result.choice, PosChoice::Home);
        assert_eq!(result.saving, 0.0);
    }

    #[test]
    fn recommends_a_persisted_foreign_gap() {
        let quotes = vec![
            quote("DE", 140.0, PosStatus::Eligible),
            quote("PT", 110.0, PosStatus::Eligible),
        ];
        let persisted: HashSet<String> = ["PT".to_string()].into_iter().collect();
        let result = recommend(&quotes, "DE", 140.0, &persisted, 20.0);
        assert_eq!(result.choice, PosChoice::Foreign);
        assert_eq!(result.market.as_deref(), Some("PT"));
        assert_eq!(result.saving, 30.0);
        assert!(result.persisted);
    }

    #[test]
    fn a_new_gap_keeps_the_home_storefront() {
        let quotes = vec![
            quote("DE", 140.0, PosStatus::Eligible),
            quote("PT", 110.0, PosStatus::Eligible),
        ];
        let result = recommend(&quotes, "DE", 140.0, &HashSet::new(), 20.0);
        assert_eq!(result.choice, PosChoice::Home);
        assert!(!result.persisted);
        assert!(result.reason.contains("gap is new"));
    }

    #[test]
    fn listed_rows_are_never_recommended() {
        let quotes = vec![
            quote("DE", 140.0, PosStatus::Eligible),
            quote("PT", 100.0, PosStatus::Listed),
        ];
        let result = recommend(&quotes, "DE", 140.0, &HashSet::new(), 20.0);
        assert_eq!(result.choice, PosChoice::Home);
    }
}
