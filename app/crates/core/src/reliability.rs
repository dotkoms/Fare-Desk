//! Reliability: on-time rate and reliability-adjusted price from BTS counts.

/// On-time share, 0–1. None when there are no flights.
pub fn on_time_rate(on_time: u32, flights: u32) -> Option<f64> {
    if flights == 0 {
        return None;
    }
    Some((f64::from(on_time) / f64::from(flights) * 1000.0).round() / 1000.0)
}

/// A low on-time rate would inflate the adjusted price absurdly, so rates
/// below 50% are reported as unknown rather than priced.
pub const MIN_RATE_FOR_ADJUSTMENT: f64 = 0.5;

/// `price / on-time probability`, the expected cost of a delay-prone fare.
pub fn adjusted_price(price: f64, on_time_rate: f64) -> Option<f64> {
    if !(0.0..=1.0).contains(&on_time_rate) || on_time_rate < MIN_RATE_FOR_ADJUSTMENT {
        return None;
    }
    Some((price / on_time_rate * 100.0).round() / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_is_a_share() {
        assert_eq!(on_time_rate(75, 100), Some(0.75));
        assert_eq!(on_time_rate(2, 3), Some(0.667));
        assert_eq!(on_time_rate(0, 0), None);
    }

    #[test]
    fn adjusted_price_divides_by_the_rate() {
        assert_eq!(adjusted_price(100.0, 0.8), Some(125.0));
        assert_eq!(adjusted_price(100.0, 1.0), Some(100.0));
    }

    #[test]
    fn very_poor_reliability_is_not_priced() {
        assert_eq!(adjusted_price(100.0, 0.4), None);
    }
}
