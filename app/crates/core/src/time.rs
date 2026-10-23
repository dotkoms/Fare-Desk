use chrono::DateTime;

/// Minutes between an arrival and the next departure, from local timestamps
/// with offsets. Negative values mean the departure is before the arrival.
pub fn layover_minutes(arrive_local: &str, depart_local: &str) -> Option<i64> {
    let arrive = DateTime::parse_from_rfc3339(arrive_local).ok()?;
    let depart = DateTime::parse_from_rfc3339(depart_local).ok()?;
    Some((depart - arrive).num_minutes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layover_respects_offsets() {
        // 07:45 at Stansted (UTC) to 09:15 at Stansted: 90 minutes.
        assert_eq!(
            layover_minutes("2027-03-01T07:45:00+00:00", "2027-03-01T09:15:00+00:00"),
            Some(90)
        );
        // 09:30+01:00 is 08:30 UTC; 12:10+00:00 is 12:10 UTC: 220 minutes.
        assert_eq!(
            layover_minutes("2027-03-01T09:30:00+01:00", "2027-03-01T12:10:00+00:00"),
            Some(220)
        );
    }

    #[test]
    fn bad_timestamps_return_none() {
        assert!(layover_minutes("07:45", "09:15").is_none());
    }
}
