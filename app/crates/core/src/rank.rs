use crate::contract::{Itinerary, Score, Sort};
use crate::pricing::round_money;

/// €/hour used to turn duration into money. Adjustable later behind "advanced".
pub const VALUE_OF_TIME_EUR_PER_HOUR: f64 = 15.0;
/// €/stop. A connection costs more than the time it takes.
pub const STOP_PENALTY_EUR: f64 = 25.0;

pub fn score_for(itinerary: &Itinerary) -> Score {
    let price = round_money(itinerary.total_price);
    let time_cost = round_money(
        f64::from(itinerary.total_duration_minutes) / 60.0 * VALUE_OF_TIME_EUR_PER_HOUR,
    );
    let stop_penalty = round_money(f64::from(itinerary.stops) * STOP_PENALTY_EUR);
    let risk_premium = round_money(
        itinerary
            .risk
            .iter()
            .map(|risk| risk.premium())
            .sum::<f64>(),
    );
    Score {
        total: round_money(price + time_cost + stop_penalty + risk_premium),
        price,
        time_cost,
        stop_penalty,
        risk_premium,
    }
}

/// Attach a score to every itinerary. Idempotent.
pub fn apply_scores(itineraries: &mut [Itinerary]) {
    for itinerary in itineraries {
        itinerary.score = Some(score_for(itinerary));
    }
}

pub fn sort_itineraries(itineraries: &mut [Itinerary], sort: Sort) {
    itineraries.sort_by(|a, b| {
        let key = |itinerary: &Itinerary| {
            itinerary
                .score
                .map(|s| s.total)
                .unwrap_or(itinerary.total_price)
        };
        match sort {
            Sort::Best => key(a).total_cmp(&key(b)),
            Sort::Cheapest => a
                .total_price
                .total_cmp(&b.total_price)
                .then(a.total_duration_minutes.cmp(&b.total_duration_minutes)),
            Sort::Fastest => a
                .total_duration_minutes
                .cmp(&b.total_duration_minutes)
                .then(a.total_price.total_cmp(&b.total_price)),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{Bags, Cabin, FareRules, Place, PriceBreakdown, Risk, Segment};

    fn place(code: &str) -> Place {
        Place {
            id: format!("airport:{code}"),
            code: code.into(),
            name: code.into(),
            city: code.into(),
            country: "DE".into(),
            country_name: "Germany".into(),
            lat: 0.0,
            lng: 0.0,
            timezone: "Europe/Berlin".into(),
            metro: None,
            scheduled_service: true,
        }
    }

    fn itinerary(id: &str, price: f64, minutes: u32, stops: u8, risk: Vec<Risk>) -> Itinerary {
        let segment = Segment {
            id: id.into(),
            marketing_carrier: "TP".into(),
            operating_carrier: "TP".into(),
            service_number: "1".into(),
            from: place("BER"),
            to: place("LIS"),
            depart_terminal: None,
            arrive_terminal: None,
            depart_local: "2027-03-01T07:10:00+01:00".into(),
            arrive_local: "2027-03-01T09:30:00+00:00".into(),
            duration_minutes: minutes,
            aircraft: None,
            cabin: Cabin::Economy,
            fare_class: None,
        };
        Itinerary {
            id: id.into(),
            segments: vec![segment],
            total_price: price,
            currency: "EUR".into(),
            total_duration_minutes: minutes,
            stops,
            fare_family: None,
            bags_included: Bags::default(),
            fare_rules: FareRules::default(),
            provider_url: String::new(),
            provider_name: String::new(),
            source: "test".into(),
            verified_at: None,
            risk,
            connection_warnings: Vec::new(),
            price_breakdown: PriceBreakdown {
                fare: price,
                taxes_and_fees: None,
                bags: 0.0,
                bag_fee_estimated: false,
                per_traveller: price,
                travellers: 1,
            },
            score: None,
        }
    }

    #[test]
    fn score_breaks_down_as_specified() {
        let score = score_for(&itinerary("a", 100.0, 120, 1, vec![]));
        assert_eq!(score.price, 100.0);
        assert_eq!(score.time_cost, 30.0);
        assert_eq!(score.stop_penalty, 25.0);
        assert_eq!(score.total, 155.0);
    }

    #[test]
    fn risk_is_priced_in() {
        let score = score_for(&itinerary(
            "a",
            100.0,
            120,
            0,
            vec![Risk::SelfTransfer, Risk::OtaOnly],
        ));
        assert_eq!(score.risk_premium, 65.0);
    }

    #[test]
    fn best_prefers_the_better_total_not_the_lowest_fare() {
        let mut list = vec![
            itinerary("cheap-slow", 80.0, 600, 0, vec![]),
            itinerary("dear-fast", 110.0, 120, 0, vec![]),
        ];
        apply_scores(&mut list);
        sort_itineraries(&mut list, Sort::Best);
        assert_eq!(list[0].id, "dear-fast");
        sort_itineraries(&mut list, Sort::Cheapest);
        assert_eq!(list[0].id, "cheap-slow");
        sort_itineraries(&mut list, Sort::Fastest);
        assert_eq!(list[0].id, "dear-fast");
    }
}
