//! The buy/wait engine.
//!
//! Rules follow PLAN.md §6.2. Everything is empirical: the recommendation
//! compares today's fare with the route's own observed distribution, the
//! ranges come from observed 7- and 21-day price moves, and the backtest
//! replays "wait or buy" at 60/30/14 days out. No confidence scores.

use chrono::NaiveDate;

use crate::contract::{Backtest, CurvePoint, Range, Recommendation};
use crate::stats::{percentile, percentile_rank};

pub const MIN_OBSERVATIONS: u32 = 14;
pub const WAIT_HORIZON_DAYS: i64 = 45;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observation {
    pub travel_date: NaiveDate,
    pub observed_on: NaiveDate,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct AdviceInput<'a> {
    /// All observations for the route, across travel dates.
    pub observations: &'a [Observation],
    pub travel_date: NaiveDate,
    pub today: NaiveDate,
    pub current_price: f64,
    pub p10: f64,
    pub p50: f64,
    /// Observations for this travel date.
    pub sample_size: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Advice {
    pub recommendation: Recommendation,
    pub expected7: Option<Range>,
    pub expected21: Option<Range>,
    /// Percent chance the fare is below p10 in seven days if the traveller waits.
    pub downside: Option<f64>,
    pub curve: Vec<CurvePoint>,
    pub backtest: Option<Backtest>,
    pub reasons: Vec<String>,
    pub change_my_mind: String,
}

pub fn advise(input: &AdviceInput<'_>) -> Advice {
    let days_out = (input.travel_date - input.today).num_days().max(0);
    let mut reasons = Vec::new();

    if (input.sample_size as u32) < MIN_OBSERVATIONS {
        reasons.push(format!(
            "Collecting baseline — {} of {} daily observations.",
            input.sample_size, MIN_OBSERVATIONS
        ));
        return Advice {
            recommendation: Recommendation::Insufficient,
            expected7: None,
            expected21: None,
            downside: None,
            curve: curve(input.observations),
            backtest: backtest(input.observations),
            reasons,
            change_my_mind: String::new(),
        };
    }

    let recommendation = if input.current_price <= input.p10 {
        reasons.push("Below the 10th percentile of observed fares for these dates.".to_string());
        Recommendation::Buy
    } else if input.current_price <= input.p50 {
        reasons.push(
            "Between the 10th and 50th percentile; there is no clear edge either way.".to_string(),
        );
        Recommendation::Watch
    } else if days_out <= 14 {
        reasons.push(
            "Above the median and inside 14 days; late drops are not something we can promise."
                .to_string(),
        );
        Recommendation::Watch
    } else if days_out > WAIT_HORIZON_DAYS {
        reasons.push(format!(
            "Above the median with {days_out} days to go; fares on this route usually improve before they rise."
        ));
        Recommendation::Wait
    } else {
        reasons.push(
            "Above the median but inside the 45-day window; watch rather than wait.".to_string(),
        );
        Recommendation::Watch
    };

    if let Some(rank) = percentile_rank(
        &sorted_prices_for_date(input.observations, input.travel_date),
        input.current_price,
    ) {
        reasons.push(format!(
            "Today's fare sits at the {:.0}th percentile.",
            rank
        ));
    }

    let expected7 = expected_range(input, 7);
    let expected21 = expected_range(input, 21);
    let downside = downside(input);

    if recommendation == Recommendation::Watch {
        reasons.push(
            "Do not manufacture a verdict: waiting and booking are both defensible.".to_string(),
        );
    }

    Advice {
        recommendation,
        expected7,
        expected21,
        downside,
        curve: curve(input.observations),
        backtest: backtest(input.observations),
        reasons,
        change_my_mind: format!(
            "A new sale bucket, a schedule change, or a fare below €{:.0}.",
            input.p10
        ),
    }
}

fn sorted_prices_for_date(observations: &[Observation], travel_date: NaiveDate) -> Vec<f64> {
    let mut prices: Vec<f64> = observations
        .iter()
        .filter(|observation| observation.travel_date == travel_date)
        .map(|observation| observation.price)
        .collect();
    prices.sort_by(f64::total_cmp);
    prices
}

/// Price ratios observed over a horizon (7 or 21 days) for the same travel
/// date. Needs at least three moves; otherwise no range is shown.
fn move_ratios(observations: &[Observation], travel_date: NaiveDate, horizon: i64) -> Vec<f64> {
    let mut series: Vec<(NaiveDate, f64)> = observations
        .iter()
        .filter(|observation| observation.travel_date == travel_date)
        .map(|observation| (observation.observed_on, observation.price))
        .collect();
    series.sort_by_key(|(date, _)| *date);

    let mut ratios = Vec::new();
    for (index, (start_date, start_price)) in series.iter().enumerate() {
        if *start_price <= 0.0 {
            continue;
        }
        // Allow a day either side so a missed collection does not erase the sample.
        for (end_date, end_price) in series.iter().skip(index + 1) {
            let gap = (*end_date - *start_date).num_days();
            if gap > horizon + 1 {
                break;
            }
            if gap >= horizon - 1 {
                ratios.push(end_price / start_price);
                break;
            }
        }
    }
    ratios
}

fn expected_range(input: &AdviceInput<'_>, horizon: i64) -> Option<Range> {
    let ratios = move_ratios(input.observations, input.travel_date, horizon);
    if ratios.len() < 3 {
        return None;
    }
    let mut sorted = ratios;
    sorted.sort_by(f64::total_cmp);
    let low = percentile(&sorted, 10.0)? * input.current_price;
    let high = percentile(&sorted, 90.0)? * input.current_price;
    Some(Range {
        low: low.max(0.0).round(),
        high: high.max(0.0).round(),
    })
}

fn downside(input: &AdviceInput<'_>) -> Option<f64> {
    let ratios = move_ratios(input.observations, input.travel_date, 7);
    if ratios.len() < 3 {
        return None;
    }
    let below = ratios
        .iter()
        .filter(|ratio| input.current_price * **ratio < input.p10)
        .count();
    Some((below as f64 / ratios.len() as f64 * 100.0).round())
}

/// Observed prices by lead-time bucket for the route. Buckets need at least
/// three observations to be shown.
fn curve(observations: &[Observation]) -> Vec<CurvePoint> {
    const BUCKETS: &[(i64, i64, u32)] = &[
        (0, 7, 4),
        (8, 14, 11),
        (15, 30, 22),
        (31, 60, 45),
        (61, 90, 75),
        (91, 180, 135),
        (181, 365, 270),
    ];

    BUCKETS
        .iter()
        .filter_map(|(from, to, midpoint)| {
            let mut prices: Vec<f64> = observations
                .iter()
                .filter(|observation| {
                    let days = (observation.travel_date - observation.observed_on).num_days();
                    days >= *from && days <= *to
                })
                .map(|observation| observation.price)
                .collect();
            if prices.len() < 3 {
                return None;
            }
            prices.sort_by(f64::total_cmp);
            Some(CurvePoint {
                days_out: *midpoint,
                median: percentile(&prices, 50.0)?.round(),
                p10: percentile(&prices, 10.0)?.round(),
                p90: percentile(&prices, 90.0)?.round(),
                sample: prices.len(),
            })
        })
        .collect()
}

/// Replay "wait or buy" at 60, 30 and 14 days out on completed travel dates.
/// Waiting wins when the fare later fell below the price at the checkpoint.
fn backtest(observations: &[Observation]) -> Option<Backtest> {
    let mut by_travel_date: std::collections::BTreeMap<NaiveDate, Vec<(NaiveDate, f64)>> =
        std::collections::BTreeMap::new();
    for observation in observations {
        by_travel_date
            .entry(observation.travel_date)
            .or_default()
            .push((observation.observed_on, observation.price));
    }

    let mut cases = 0u32;
    let mut wins = 0u32;
    let mut saved: f64 = 0.0;
    let mut worst: f64 = 0.0;

    for (travel_date, mut series) in by_travel_date {
        series.sort_by_key(|(date, _)| *date);
        for checkpoint in [60, 30, 14] {
            // The observation closest to the checkpoint, not merely any
            // observation at least that far out.
            let Some((checkpoint_date, checkpoint_price)) = series
                .iter()
                .filter(|(observed_on, _)| (*observed_on - travel_date).num_days() <= -checkpoint)
                .max_by_key(|(observed_on, _)| *observed_on)
            else {
                continue;
            };
            // Only judge a checkpoint we could have acted on, and only when the
            // travel date has been observed again after it.
            if *checkpoint_date == travel_date {
                continue;
            }
            let later: Vec<f64> = series
                .iter()
                .filter(|(observed_on, _)| observed_on > checkpoint_date)
                .map(|(_, price)| *price)
                .collect();
            if later.is_empty() {
                continue;
            }
            let later_min = later.iter().copied().fold(f64::INFINITY, f64::min);
            cases += 1;
            if later_min < *checkpoint_price {
                wins += 1;
                saved += *checkpoint_price - later_min;
            } else {
                worst = worst.max(later_min - *checkpoint_price);
            }
        }
    }

    if cases < 3 {
        return None;
    }
    Some(Backtest {
        cases,
        wait_wins: wins,
        avg_saving: (saved / f64::from(cases)).round(),
        worst_loss: worst.round(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn series(travel: NaiveDate, prices: &[f64]) -> Vec<Observation> {
        let start = travel - Duration::days(prices.len() as i64 + 30);
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

    fn input(
        observations: &[Observation],
        travel_date: NaiveDate,
        today: NaiveDate,
        current: f64,
        p10: f64,
        p50: f64,
        sample: usize,
    ) -> AdviceInput<'_> {
        AdviceInput {
            observations,
            travel_date,
            today,
            current_price: current,
            p10,
            p50,
            sample_size: sample,
        }
    }

    fn travel_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2027, 3, 1).unwrap()
    }

    #[test]
    fn refuses_to_advise_under_fourteen_observations() {
        let observations = series(travel_date(), &[100.0, 101.0]);
        let advice = advise(&input(
            &observations,
            travel_date(),
            travel_date() - Duration::days(60),
            100.0,
            100.0,
            101.0,
            2,
        ));
        assert_eq!(advice.recommendation, Recommendation::Insufficient);
        assert!(advice.reasons[0].contains("Collecting baseline"));
        assert!(advice.expected7.is_none());
    }

    #[test]
    fn below_p10_books_now() {
        let prices: Vec<f64> = (100..114).map(|price| price as f64).collect();
        let observations = series(travel_date(), &prices);
        let advice = advise(&input(
            &observations,
            travel_date(),
            travel_date() - Duration::days(60),
            99.0,
            100.3,
            106.5,
            14,
        ));
        assert_eq!(advice.recommendation, Recommendation::Buy);
        assert!(advice
            .reasons
            .iter()
            .any(|reason| reason.contains("10th percentile")));
    }

    #[test]
    fn above_median_far_out_waits_and_inside_fourteen_days_watches() {
        let prices: Vec<f64> = (100..114).map(|price| price as f64).collect();
        let observations = series(travel_date(), &prices);
        let far = advise(&input(
            &observations,
            travel_date(),
            travel_date() - Duration::days(60),
            140.0,
            100.3,
            106.5,
            14,
        ));
        assert_eq!(far.recommendation, Recommendation::Wait);
        assert!(far
            .reasons
            .iter()
            .any(|reason| reason.contains("days to go")));

        let near = advise(&input(
            &observations,
            travel_date(),
            travel_date() - Duration::days(7),
            140.0,
            100.3,
            106.5,
            14,
        ));
        assert_eq!(near.recommendation, Recommendation::Watch);
        assert!(near
            .reasons
            .iter()
            .any(|reason| reason.contains("inside 14 days")));
    }

    #[test]
    fn expected_ranges_and_downside_come_from_observed_moves() {
        // 25 daily points that rise by €2 a day: every 7-day move is ~+10%.
        let prices: Vec<f64> = (0..25).map(|index| 100.0 + index as f64 * 2.0).collect();
        let observations = series(travel_date(), &prices);
        let advice = advise(&input(
            &observations,
            travel_date(),
            travel_date() - Duration::days(60),
            142.0,
            100.0,
            121.0,
            25,
        ));
        let seven = advice.expected7.expect("7-day range");
        assert!(seven.low > 142.0, "{seven:?}");
        assert!(seven.high > seven.low);
        let twenty_one = advice.expected21.expect("21-day range");
        assert!(twenty_one.high >= twenty_one.low);
        let downside = advice.downside.expect("downside");
        assert!((0.0..=100.0).contains(&downside));
    }

    #[test]
    fn curve_buckets_need_three_observations() {
        let observations = series(travel_date(), &[100.0, 101.0, 102.0, 103.0]);
        let points = curve(&observations);
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].sample, 4);
    }

    #[test]
    fn backtest_counts_wait_wins_and_losses() {
        // One travel date, a 60-day checkpoint at €120, then prices fall to €110
        // (wait wins), and a 30-day checkpoint at €110 with no later drop.
        let travel = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        let observations = vec![
            Observation {
                travel_date: travel,
                observed_on: travel - Duration::days(60),
                price: 120.0,
            },
            Observation {
                travel_date: travel,
                observed_on: travel - Duration::days(45),
                price: 115.0,
            },
            Observation {
                travel_date: travel,
                observed_on: travel - Duration::days(30),
                price: 110.0,
            },
            Observation {
                travel_date: travel,
                observed_on: travel - Duration::days(14),
                price: 112.0,
            },
            Observation {
                travel_date: travel,
                observed_on: travel - Duration::days(7),
                price: 118.0,
            },
        ];
        let result = backtest(&observations).expect("backtest");
        assert_eq!(result.cases, 3);
        assert_eq!(result.wait_wins, 1);
        assert_eq!(result.avg_saving, 3.0);
        assert_eq!(result.worst_loss, 6.0);
    }
}
