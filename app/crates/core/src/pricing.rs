use crate::contract::{Offer, PriceBreakdown, SearchQuery};

/// Curated checked-bag fees, per bag, per traveller, in EUR.
///
/// These are first-bag airport/online rates for a typical economy fare and are
/// only used when a source reports that a bag costs extra without naming the
/// amount. The response marks the total as estimated so the UI can say so.
const BAG_FEE_EUR: &[(&str, f64)] = &[
    ("TAP", 40.0),
    ("TP", 40.0),
    ("Lufthansa", 45.0),
    ("LH", 45.0),
    ("Ryanair", 45.0),
    ("FR", 45.0),
    ("easyJet", 42.0),
    ("U2", 42.0),
    ("Iberia", 40.0),
    ("IB", 40.0),
    ("ITA", 45.0),
    ("AZ", 45.0),
    ("British Airways", 50.0),
    ("BA", 50.0),
    ("KLM", 45.0),
    ("KL", 45.0),
    ("Air France", 45.0),
    ("AF", 45.0),
    ("Vueling", 40.0),
    ("VY", 40.0),
];

const DEFAULT_BAG_FEE_EUR: f64 = 45.0;

pub fn estimate_bag_fee(carrier: &str) -> f64 {
    BAG_FEE_EUR
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(carrier))
        .map(|(_, fee)| *fee)
        .unwrap_or(DEFAULT_BAG_FEE_EUR)
}

/// Price one offer for a party, including the checked bags the traveller asked
/// for that the fare does not include.
pub fn price_offer(offer: &Offer, query: &SearchQuery) -> (f64, PriceBreakdown) {
    let travellers = query.travellers();
    let fare = round_money(offer.price_per_traveller * f64::from(travellers));

    let missing_per_traveller = query
        .checked_bags
        .saturating_sub(offer.bags_included.checked);
    let missing_bags = u16::from(missing_per_traveller) * u16::from(travellers);

    let (bag_fee, estimated) = if missing_bags == 0 {
        (0.0, false)
    } else {
        match offer.bag_fee_per_checked_bag {
            Some(fee) => (fee, false),
            None => (
                estimate_bag_fee(
                    offer
                        .segments
                        .first()
                        .map(|s| s.marketing_carrier.as_str())
                        .unwrap_or(""),
                ),
                true,
            ),
        }
    };
    let bags = round_money(bag_fee * f64::from(missing_bags));
    let total = round_money(fare + bags);

    let breakdown = PriceBreakdown {
        fare,
        taxes_and_fees: offer.taxes_and_fees,
        bags,
        bag_fee_estimated: estimated,
        per_traveller: round_money(total / f64::from(travellers)),
        travellers,
    };
    (total, breakdown)
}

pub fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{Bags, FareRules};

    fn offer(price: f64, included: u8, bag_fee: Option<f64>) -> Offer {
        Offer {
            segments: vec![],
            total_duration_minutes: None,
            price_per_traveller: price,
            currency: "EUR".into(),
            taxes_and_fees: None,
            bags_included: Bags {
                carry_on: 1,
                checked: included,
            },
            bag_fee_per_checked_bag: bag_fee,
            fare_family: None,
            fare_rules: FareRules::default(),
            provider_url: String::new(),
            provider_name: String::new(),
            source: "test".into(),
            verified_at: None,
            risk: vec![],
            warnings: vec![],
        }
    }

    fn query(people: u8, bags: u8) -> SearchQuery {
        SearchQuery {
            adults: people,
            checked_bags: bags,
            ..Default::default()
        }
    }

    #[test]
    fn party_total_multiplies_fare() {
        let (total, breakdown) = price_offer(&offer(119.0, 1, None), &query(2, 1));
        assert_eq!(total, 238.0);
        assert_eq!(breakdown.bags, 0.0);
        assert_eq!(breakdown.per_traveller, 119.0);
    }

    #[test]
    fn missing_bags_are_added_per_traveller() {
        let (total, breakdown) = price_offer(&offer(104.0, 0, Some(45.0)), &query(2, 1));
        assert_eq!(total, 298.0);
        assert_eq!(breakdown.bags, 90.0);
        assert!(!breakdown.bag_fee_estimated);
    }

    #[test]
    fn unknown_bag_fee_is_estimated_and_flagged() {
        let (total, breakdown) = price_offer(&offer(100.0, 0, None), &query(1, 1));
        assert_eq!(total, 145.0);
        assert!(breakdown.bag_fee_estimated);
    }

    #[test]
    fn included_bags_are_not_charged_twice() {
        let (total, breakdown) = price_offer(&offer(189.0, 1, Some(45.0)), &query(1, 1));
        assert_eq!(total, 189.0);
        assert_eq!(breakdown.bags, 0.0);
    }

    #[test]
    fn known_carriers_beat_the_default() {
        assert_eq!(estimate_bag_fee("Ryanair"), 45.0);
        assert_eq!(estimate_bag_fee("TP"), 40.0);
        assert_eq!(estimate_bag_fee("Unknown Air"), DEFAULT_BAG_FEE_EUR);
    }
}
