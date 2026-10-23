/// Inclusive percentile (linear interpolation, the Excel PERCENTILE.INC
/// definition) over a sorted slice. Returns None for an empty slice.
pub fn percentile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let p = p.clamp(0.0, 100.0);
    let rank = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        return Some(sorted[lower]);
    }
    let weight = rank - lower as f64;
    Some(sorted[lower] * (1.0 - weight) + sorted[upper] * weight)
}

/// Share of observations at or below `value`, 0–100. Used for "this fare is
/// in the 22nd percentile of what we have seen".
pub fn percentile_rank(sorted: &[f64], value: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let at_or_below = sorted.iter().filter(|price| **price <= value).count();
    Some(at_or_below as f64 / sorted.len() as f64 * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_interpolate() {
        let prices = [100.0, 110.0, 120.0, 130.0, 140.0];
        assert_eq!(percentile(&prices, 0.0), Some(100.0));
        assert_eq!(percentile(&prices, 50.0), Some(120.0));
        assert_eq!(percentile(&prices, 100.0), Some(140.0));
        assert_eq!(percentile(&prices, 25.0), Some(110.0));
        assert_eq!(percentile(&prices, 10.0), Some(104.0));
    }

    #[test]
    fn percentile_handles_single_and_empty() {
        assert_eq!(percentile(&[119.0], 90.0), Some(119.0));
        assert_eq!(percentile(&[], 50.0), None);
    }

    #[test]
    fn rank_counts_values_at_or_below() {
        let prices = [100.0, 110.0, 120.0, 130.0];
        assert_eq!(percentile_rank(&prices, 120.0), Some(75.0));
        assert_eq!(percentile_rank(&prices, 99.0), Some(0.0));
        assert_eq!(percentile_rank(&prices, 200.0), Some(100.0));
        assert_eq!(percentile_rank(&[], 120.0), None);
    }
}
