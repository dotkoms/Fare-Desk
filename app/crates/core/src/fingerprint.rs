use crate::contract::Segment;

/// A stable id for the physical services in an itinerary. Two offers for the
/// same flights and date share a fingerprint even when an OTA sells them as
/// different products.
pub fn fingerprint(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(|segment| {
            let date = segment
                .depart_local
                .get(..10)
                .unwrap_or(&segment.depart_local);
            format!(
                "{}-{}-{}",
                segment.marketing_carrier.trim().replace(' ', ""),
                segment.service_number.trim().replace(' ', ""),
                date
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{Cabin, Place};

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

    fn segment(carrier: &str, number: &str, depart: &str) -> Segment {
        Segment {
            id: format!("{carrier}{number}"),
            marketing_carrier: carrier.into(),
            operating_carrier: carrier.into(),
            service_number: number.into(),
            from: place("BER"),
            to: place("LIS"),
            depart_terminal: None,
            arrive_terminal: None,
            depart_local: depart.into(),
            arrive_local: depart.into(),
            duration_minutes: 200,
            aircraft: None,
            cabin: Cabin::Economy,
            fare_class: None,
        }
    }

    #[test]
    fn same_services_share_a_fingerprint() {
        let a = fingerprint(&[segment("TP", "1041", "2027-03-01T07:10:00+01:00")]);
        let b = fingerprint(&[segment("TP", "1041", "2027-03-01T07:10:00+01:00")]);
        assert_eq!(a, b);
        assert_eq!(a, "TP-1041-2027-03-01");
    }

    #[test]
    fn different_dates_differ() {
        let a = fingerprint(&[segment("TP", "1041", "2027-03-01T07:10:00+01:00")]);
        let b = fingerprint(&[segment("TP", "1041", "2027-03-02T07:10:00+01:00")]);
        assert_ne!(a, b);
    }

    #[test]
    fn connection_changes_the_fingerprint() {
        let a = fingerprint(&[segment("FR", "1", "2027-03-01T07:00:00+01:00")]);
        let b = fingerprint(&[
            segment("FR", "1", "2027-03-01T07:00:00+01:00"),
            segment("FR", "2", "2027-03-01T12:00:00+00:00"),
        ]);
        assert_ne!(a, b);
        assert_eq!(b, "FR-1-2027-03-01|FR-2-2027-03-01");
    }
}
