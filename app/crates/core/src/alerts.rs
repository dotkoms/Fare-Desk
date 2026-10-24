//! Alert rules: quiet hours, thresholds and new lows.

use chrono::{DateTime, Timelike, Utc};

/// Parse "23:00-07:00" into minutes from midnight. The window may wrap.
pub fn parse_quiet_hours(value: &str) -> Option<(u32, u32)> {
    let (start, end) = value.trim().split_once('-')?;
    let parse = |part: &str| {
        let (hour, minute) = part.trim().split_once(':')?;
        let hour: u32 = hour.parse().ok()?;
        let minute: u32 = minute.parse().ok()?;
        (hour < 24 && minute < 60).then_some(hour * 60 + minute)
    };
    Some((parse(start)?, parse(end)?))
}

/// True when `now` falls inside the quiet window. Evaluated in UTC.
pub fn is_quiet(now: DateTime<Utc>, start_minute: u32, end_minute: u32) -> bool {
    let minute = now.hour() * 60 + now.minute();
    if start_minute == end_minute {
        return false;
    }
    if start_minute < end_minute {
        minute >= start_minute && minute < end_minute
    } else {
        minute >= start_minute || minute < end_minute
    }
}

pub fn matches_threshold(price: f64, threshold: f64) -> bool {
    price <= threshold
}

/// A new low must beat every earlier daily minimum, not merely equal it.
pub fn is_new_low(price: f64, previous_min: Option<f64>) -> bool {
    previous_min.is_some_and(|previous| price < previous)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 23, hour, minute, 0).unwrap()
    }

    #[test]
    fn quiet_hours_parse_and_wrap() {
        assert_eq!(parse_quiet_hours("23:00-07:00"), Some((1380, 420)));
        assert_eq!(parse_quiet_hours("01:30-05:00"), Some((90, 300)));
        assert_eq!(parse_quiet_hours("25:00-07:00"), None);
        assert_eq!(parse_quiet_hours("nope"), None);
    }

    #[test]
    fn quiet_window_wraps_midnight() {
        assert!(is_quiet(at(23, 30), 1380, 420));
        assert!(is_quiet(at(2, 0), 1380, 420));
        assert!(!is_quiet(at(7, 0), 1380, 420));
        assert!(!is_quiet(at(12, 0), 1380, 420));
    }

    #[test]
    fn a_same_day_window_is_respected() {
        assert!(is_quiet(at(2, 0), 60, 300));
        assert!(!is_quiet(at(6, 0), 60, 300));
    }

    #[test]
    fn thresholds_and_new_lows() {
        assert!(matches_threshold(99.0, 100.0));
        assert!(!matches_threshold(101.0, 100.0));
        assert!(is_new_low(99.0, Some(100.0)));
        assert!(!is_new_low(100.0, Some(100.0)));
        assert!(!is_new_low(99.0, None));
    }
}
