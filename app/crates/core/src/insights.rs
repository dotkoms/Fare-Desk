//! Seasonal medians and fare weather, derived only from collected history.

use std::collections::BTreeMap;

use crate::advice::Observation;
use crate::contract::{FareWeather, ObservationWindow, Range, SeasonalityPoint, WeatherClass};
use crate::stats::percentile;

/// A month is shown only when it has this many observations. A median from
/// three points is not a seasonal fact.
pub const MIN_MONTH_SAMPLE: usize = 5;
/// Below this many observations for a travel date there is no weather call.
pub const MIN_WEATHER_SAMPLE: usize = 5;

/// Median, p10 and p90 per travel month, oldest month first.
pub fn seasonality(observations: &[Observation]) -> Vec<SeasonalityPoint> {
    let mut months: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for observation in observations {
        months
            .entry(observation.travel_date.format("%Y-%m").to_string())
            .or_default()
            .push(observation.price);
    }

    months
        .into_iter()
        .filter_map(|(month, mut prices)| {
            if prices.len() < MIN_MONTH_SAMPLE {
                return None;
            }
            prices.sort_by(f64::total_cmp);
            Some(SeasonalityPoint {
                month,
                median: percentile(&prices, 50.0)?.round(),
                p10: percentile(&prices, 10.0)?.round(),
                p90: percentile(&prices, 90.0)?.round(),
                sample: prices.len(),
            })
        })
        .collect()
}

/// Volatility for one travel date: coefficient of variation, the last-seven
/// observation range and how often the fare fell by 5% or more.
pub fn fare_weather(
    observations: &[Observation],
    travel_date: chrono::NaiveDate,
    today: chrono::NaiveDate,
) -> FareWeather {
    let mut series: Vec<(chrono::NaiveDate, f64)> = observations
        .iter()
        .filter(|observation| observation.travel_date == travel_date)
        .map(|observation| (observation.observed_on, observation.price))
        .collect();
    series.sort_by_key(|(date, _)| *date);

    let sample_size = series.len();
    let window = series
        .first()
        .zip(series.last())
        .map(|((from, _), (to, _))| ObservationWindow {
            from: *from,
            to: *to,
            days: (*to - *from).num_days().max(0) as u32 + 1,
        });

    if sample_size < MIN_WEATHER_SAMPLE {
        return FareWeather {
            route: String::new(),
            travel_date,
            classification: WeatherClass::Insufficient,
            cv: 0.0,
            range7: None,
            drop_frequency: 0.0,
            sample_size,
            window,
        };
    }

    let prices: Vec<f64> = series.iter().map(|(_, price)| *price).collect();
    let mean = prices.iter().sum::<f64>() / prices.len() as f64;
    let variance = prices
        .iter()
        .map(|price| (price - mean).powi(2))
        .sum::<f64>()
        / prices.len() as f64;
    let cv = if mean > 0.0 {
        variance.sqrt() / mean
    } else {
        0.0
    };

    let recent = &prices[prices.len().saturating_sub(7)..];
    let range7 = Range {
        low: recent.iter().copied().fold(f64::INFINITY, f64::min),
        high: recent.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    };

    let drops = prices
        .windows(2)
        .filter(|pair| pair[1] <= pair[0] * 0.95)
        .count();
    let drop_frequency = drops as f64 / (prices.len() - 1) as f64;

    let first = prices.first().copied().unwrap_or(0.0);
    let last = *prices.last().unwrap_or(&0.0);
    let days_out = (travel_date - today).num_days().max(0);
    let classification = if first > 0.0 && last >= first * 1.02 && days_out <= 45 {
        WeatherClass::Rising
    } else if cv >= 0.12 || drop_frequency >= 0.30 {
        WeatherClass::Erratic
    } else {
        WeatherClass::Stable
    };

    FareWeather {
        route: String::new(),
        travel_date,
        classification,
        cv: (cv * 100.0).round() / 100.0,
        range7: Some(Range {
            low: range7.low.round(),
            high: range7.high.round(),
        }),
        drop_frequency: (drop_frequency * 100.0).round() / 100.0,
        sample_size,
        window,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate};

    fn observations(travel: NaiveDate, prices: &[f64]) -> Vec<Observation> {
        let start = travel - Duration::days(prices.len() as i64 + 20);
        prices
            .iter()
            .enumerate()
            .map(|(index, price)| Observation {
                travel_date: travel,
                observed_on: start + Duration::days(index as i64),
                price: *price,
            })
            .collect()
    }

    #[test]
    fn seasonality_needs_five_observations_per_month() {
        let march = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        let april = NaiveDate::from_ymd_opt(2027, 4, 1).unwrap();
        let mut all = observations(march, &[100.0, 110.0, 120.0, 130.0, 140.0]);
        // April has only four points and must be omitted.
        all.extend(observations(april, &[90.0, 95.0, 100.0, 105.0]));

        let points = seasonality(&all);
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].month, "2027-03");
        assert_eq!(points[0].median, 120.0);
        assert_eq!(points[0].sample, 5);
    }

    #[test]
    fn weather_is_insufficient_without_enough_history() {
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        let weather = fare_weather(
            &observations(travel, &[100.0, 101.0]),
            travel,
            travel - Duration::days(30),
        );
        assert_eq!(weather.classification, WeatherClass::Insufficient);
        assert_eq!(weather.sample_size, 2);
        assert!(weather.range7.is_none());
    }

    #[test]
    fn stable_prices_are_stable() {
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        let weather = fare_weather(
            &observations(travel, &[100.0, 100.0, 101.0, 100.0, 101.0, 100.0]),
            travel,
            travel - Duration::days(60),
        );
        assert_eq!(weather.classification, WeatherClass::Stable);
        assert!(weather.cv < 0.12);
        assert_eq!(weather.range7.unwrap().low, 100.0);
    }

    #[test]
    fn erratic_prices_are_erratic() {
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        let weather = fare_weather(
            &observations(travel, &[100.0, 160.0, 90.0, 150.0, 85.0, 140.0]),
            travel,
            travel - Duration::days(60),
        );
        assert_eq!(weather.classification, WeatherClass::Erratic);
        assert!(weather.cv >= 0.12);
        assert!(weather.drop_frequency > 0.0);
    }

    #[test]
    fn rising_inside_the_window_is_rising() {
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        let weather = fare_weather(
            &observations(travel, &[100.0, 101.0, 102.0, 103.0, 104.0, 110.0]),
            travel,
            travel - Duration::days(10),
        );
        assert_eq!(weather.classification, WeatherClass::Rising);
        assert!(weather.range7.unwrap().high >= 110.0);
    }
}
