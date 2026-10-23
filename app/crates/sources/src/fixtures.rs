//! Deterministic demo inventory. Same route and date, same offers; not real
//! inventory and never presented as live. Fixture prices do not move with the
//! calendar, so the UI must label them as fixtures.

use std::collections::HashMap;

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use faredesk_core::{Bags, Cabin, FareRules, Offer, Place, Provider, Risk, Segment};

use crate::{ResolvedQuery, SourceError};

pub struct FixtureSource {
    places: HashMap<String, Place>,
    providers: Vec<Provider>,
}

impl FixtureSource {
    pub fn id(&self) -> &'static str {
        "fixtures"
    }

    pub fn data_card_id(&self) -> &'static str {
        "fares-fixture"
    }

    pub fn new(places: HashMap<String, Place>, providers: Vec<Provider>) -> Self {
        Self { places, providers }
    }

    pub fn search(&self, query: &ResolvedQuery<'_>) -> Result<Vec<Offer>, SourceError> {
        let route = (query.origin.code.as_str(), query.destination.code.as_str());
        let mut offers = Vec::new();
        for fixture in all_fixtures() {
            if fixture.origin != route.0 || fixture.destination != route.1 {
                continue;
            }
            if let Some(max) = query.request.max_stops {
                if fixture.legs.len().saturating_sub(1) as u8 > max {
                    continue;
                }
            }
            offers.push(self.build(fixture, fixture.legs, query, false)?);
        }
        // The demo network is drawn one way. A return search asks for the reverse
        // route, so fly the same services back on a mirrored schedule.
        if offers.is_empty() {
            for fixture in all_fixtures() {
                if fixture.origin != route.1 || fixture.destination != route.0 {
                    continue;
                }
                if let Some(max) = query.request.max_stops {
                    if fixture.legs.len().saturating_sub(1) as u8 > max {
                        continue;
                    }
                }
                let legs = self.mirror(fixture, query.request.depart_date)?;
                offers.push(self.build(fixture, &legs, query, true)?);
            }
        }
        Ok(offers)
    }

    /// The fixture's legs reversed: same carriers and aircraft, flight numbers one
    /// higher, departing mid-day with 75-minute connections.
    fn mirror(&self, fixture: &FixtureOffer, date: NaiveDate) -> Result<Vec<Leg>, SourceError> {
        const CONNECTION_MINUTES: i64 = 75;
        let mut legs = Vec::with_capacity(fixture.legs.len());
        let mut next_departure: Option<DateTime<Utc>> = None;
        for leg in fixture.legs.iter().rev() {
            let from = self.place(leg.to)?;
            let (hour, minute) = match next_departure {
                None => (6 + leg.hour % 12, leg.minute),
                Some(instant) => {
                    let tz: Tz = from.timezone.parse().map_err(|_| {
                        SourceError::Unavailable(format!("unknown timezone {}", from.timezone))
                    })?;
                    let local = instant.with_timezone(&tz);
                    (local.hour(), local.minute())
                }
            };
            let depart = depart_at(from, date, hour, minute)?;
            next_departure = Some(
                depart
                    + Duration::minutes(i64::from(leg.minutes))
                    + Duration::minutes(CONNECTION_MINUTES),
            );
            legs.push(Leg {
                from: leg.to,
                to: leg.from,
                hour,
                minute,
                ..*leg
            });
        }
        Ok(legs)
    }

    fn build(
        &self,
        fixture: &FixtureOffer,
        fixture_legs: &[Leg],
        query: &ResolvedQuery<'_>,
        mirrored: bool,
    ) -> Result<Offer, SourceError> {
        let date = query.request.depart_date;
        let mut segments = Vec::with_capacity(fixture.legs.len());
        let mut first_departure: Option<DateTime<Utc>> = None;
        let mut last_arrival: Option<DateTime<Utc>> = None;

        for leg in fixture_legs {
            let number = if mirrored {
                next_flight_number(leg.number)
            } else {
                leg.number.to_string()
            };
            let from = self.place(leg.from)?;
            let to = self.place(leg.to)?;
            let depart = depart_at(from, date, leg.hour, leg.minute)?;
            let arrive = depart + Duration::minutes(i64::from(leg.minutes));
            first_departure.get_or_insert(depart);
            last_arrival = Some(arrive);
            segments.push(Segment {
                id: format!(
                    "{}-{}-{}",
                    leg.carrier.replace(' ', ""),
                    number.replace(' ', ""),
                    date
                ),
                marketing_carrier: leg.carrier.to_string(),
                operating_carrier: leg.carrier.to_string(),
                service_number: number,
                from: from.clone(),
                to: to.clone(),
                depart_terminal: None,
                arrive_terminal: None,
                depart_local: local_rfc3339(depart, &from.timezone)?,
                arrive_local: local_rfc3339(arrive, &to.timezone)?,
                duration_minutes: leg.minutes,
                aircraft: Some(leg.aircraft.to_string()),
                cabin: Cabin::Economy,
                fare_class: None,
            });
        }

        let total_duration = match (first_departure, last_arrival) {
            (Some(start), Some(end)) => (end - start).num_minutes().max(0) as u32,
            _ => 0,
        };

        let provider = self
            .providers
            .iter()
            .find(|provider| provider.name.eq_ignore_ascii_case(fixture.carrier_name()));

        let provider_name = provider
            .map(|p| p.name.clone())
            .unwrap_or_else(|| fixture.carrier_name().to_string());
        let provider_url = provider
            .map(|p| {
                fill_template(
                    &p.deep_link_template,
                    query.origin.code.as_str(),
                    query.destination.code.as_str(),
                    date,
                    query.request.adults,
                )
            })
            .unwrap_or_default();

        Ok(Offer {
            segments,
            total_duration_minutes: Some(total_duration),
            price_per_traveller: fixture.price,
            currency: "EUR".to_string(),
            taxes_and_fees: None,
            bags_included: Bags {
                carry_on: fixture.carry_on,
                checked: fixture.checked,
            },
            bag_fee_per_checked_bag: fixture.bag_fee,
            fare_family: Some(fixture.fare_family.to_string()),
            fare_rules: FareRules {
                refundable: fixture.refundable,
                changeable: fixture.change_fee.is_some(),
                change_fee: fixture.change_fee,
                cancel_fee: if fixture.refundable { Some(0.0) } else { None },
            },
            provider_url,
            provider_name,
            source: self.id().to_string(),
            // Fixtures are not real prices; never claim they were verified.
            verified_at: None,
            risk: fixture.risk.to_vec(),
            warnings: vec!["Demo fixture, not live inventory.".to_string()],
        })
    }

    fn place(&self, code: &str) -> Result<&Place, SourceError> {
        self.places.get(code).ok_or_else(|| {
            SourceError::Unavailable(format!(
                "fixture airport {code} is missing from the reference set"
            ))
        })
    }
}

/// Every airport named by the fixture table. The API resolves these once.
pub const PLACE_CODES: &[&str] = &[
    "BER", "LIS", "LHR", "BCN", "JFK", "STN", "MAD", "FRA", "AMS",
];

#[derive(Clone, Copy)]
struct Leg {
    carrier: &'static str,
    number: &'static str,
    from: &'static str,
    to: &'static str,
    hour: u32,
    minute: u32,
    minutes: u32,
    aircraft: &'static str,
}

struct FixtureOffer {
    origin: &'static str,
    destination: &'static str,
    legs: &'static [Leg],
    price: f64,
    fare_family: &'static str,
    carry_on: u8,
    checked: u8,
    bag_fee: Option<f64>,
    refundable: bool,
    change_fee: Option<f64>,
    risk: &'static [Risk],
}

impl FixtureOffer {
    fn carrier_name(&self) -> &'static str {
        self.legs
            .first()
            .map(|leg| leg.carrier)
            .unwrap_or("Fare Desk")
    }
}

const NONE: &[Risk] = &[];
const SELF_TRANSFER: &[Risk] = &[Risk::SelfTransfer, Risk::OtaOnly];

const BER_LIS: &[FixtureOffer] = &[
    FixtureOffer {
        origin: "BER",
        destination: "LIS",
        legs: &[Leg {
            carrier: "TAP Air Portugal",
            number: "TP 1041",
            from: "BER",
            to: "LIS",
            hour: 7,
            minute: 10,
            minutes: 200,
            aircraft: "Airbus A320",
        }],
        price: 119.0,
        fare_family: "Economy Light",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: false,
        change_fee: Some(75.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "LIS",
        legs: &[Leg {
            carrier: "easyJet",
            number: "U2 4501",
            from: "BER",
            to: "LIS",
            hour: 12,
            minute: 20,
            minutes: 195,
            aircraft: "Airbus A320neo",
        }],
        price: 149.0,
        fare_family: "Standard",
        carry_on: 1,
        checked: 0,
        bag_fee: Some(42.0),
        refundable: false,
        change_fee: Some(60.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "LIS",
        legs: &[
            Leg {
                carrier: "Ryanair",
                number: "FR 146",
                from: "BER",
                to: "STN",
                hour: 7,
                minute: 0,
                minutes: 105,
                aircraft: "Boeing 737-800",
            },
            Leg {
                carrier: "Ryanair",
                number: "FR 1082",
                from: "STN",
                to: "LIS",
                hour: 9,
                minute: 15,
                minutes: 165,
                aircraft: "Boeing 737-800",
            },
        ],
        price: 104.0,
        fare_family: "Basic",
        carry_on: 1,
        checked: 0,
        bag_fee: Some(45.0),
        refundable: false,
        change_fee: Some(55.0),
        risk: SELF_TRANSFER,
    },
    FixtureOffer {
        origin: "BER",
        destination: "LIS",
        legs: &[
            Leg {
                carrier: "Iberia",
                number: "IB 3671",
                from: "BER",
                to: "MAD",
                hour: 6,
                minute: 40,
                minutes: 170,
                aircraft: "Airbus A321",
            },
            Leg {
                carrier: "Iberia",
                number: "IB 3114",
                from: "MAD",
                to: "LIS",
                hour: 10,
                minute: 5,
                minutes: 80,
                aircraft: "Airbus A320",
            },
        ],
        price: 129.0,
        fare_family: "Economy",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: false,
        change_fee: Some(70.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "LIS",
        legs: &[Leg {
            carrier: "TAP Air Portugal",
            number: "TP 1043",
            from: "BER",
            to: "LIS",
            hour: 8,
            minute: 0,
            minutes: 200,
            aircraft: "Airbus A321neo",
        }],
        price: 189.0,
        fare_family: "Economy Flex",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: true,
        change_fee: Some(0.0),
        risk: NONE,
    },
];

const BER_LHR: &[FixtureOffer] = &[
    FixtureOffer {
        origin: "BER",
        destination: "LHR",
        legs: &[Leg {
            carrier: "British Airways",
            number: "BA 993",
            from: "BER",
            to: "LHR",
            hour: 7,
            minute: 30,
            minutes: 115,
            aircraft: "Airbus A320",
        }],
        price: 139.0,
        fare_family: "Euro Traveller",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: false,
        change_fee: Some(80.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "LHR",
        legs: &[Leg {
            carrier: "easyJet",
            number: "U2 8601",
            from: "BER",
            to: "LHR",
            hour: 6,
            minute: 20,
            minutes: 110,
            aircraft: "Airbus A319",
        }],
        price: 89.0,
        fare_family: "Standard",
        carry_on: 1,
        checked: 0,
        bag_fee: Some(42.0),
        refundable: false,
        change_fee: Some(60.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "LHR",
        legs: &[
            Leg {
                carrier: "Lufthansa",
                number: "LH 181",
                from: "BER",
                to: "FRA",
                hour: 8,
                minute: 10,
                minutes: 75,
                aircraft: "Airbus A320",
            },
            Leg {
                carrier: "Lufthansa",
                number: "LH 910",
                from: "FRA",
                to: "LHR",
                hour: 10,
                minute: 40,
                minutes: 105,
                aircraft: "Airbus A320neo",
            },
        ],
        price: 129.0,
        fare_family: "Light",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: false,
        change_fee: Some(70.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "LHR",
        legs: &[
            Leg {
                carrier: "KLM",
                number: "KL 1820",
                from: "BER",
                to: "AMS",
                hour: 9,
                minute: 15,
                minutes: 80,
                aircraft: "Embraer 190",
            },
            Leg {
                carrier: "KLM",
                number: "KL 1011",
                from: "AMS",
                to: "LHR",
                hour: 12,
                minute: 5,
                minutes: 80,
                aircraft: "Boeing 737-800",
            },
        ],
        price: 119.0,
        fare_family: "Light",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: false,
        change_fee: Some(70.0),
        risk: NONE,
    },
];

const BER_BCN: &[FixtureOffer] = &[
    FixtureOffer {
        origin: "BER",
        destination: "BCN",
        legs: &[Leg {
            carrier: "easyJet",
            number: "U2 2201",
            from: "BER",
            to: "BCN",
            hour: 10,
            minute: 5,
            minutes: 155,
            aircraft: "Airbus A320",
        }],
        price: 79.0,
        fare_family: "Standard",
        carry_on: 1,
        checked: 0,
        bag_fee: Some(42.0),
        refundable: false,
        change_fee: Some(60.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "BCN",
        legs: &[Leg {
            carrier: "Vueling",
            number: "VY 1885",
            from: "BER",
            to: "BCN",
            hour: 18,
            minute: 30,
            minutes: 150,
            aircraft: "Airbus A320neo",
        }],
        price: 69.0,
        fare_family: "Basic",
        carry_on: 1,
        checked: 0,
        bag_fee: Some(40.0),
        refundable: false,
        change_fee: Some(55.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "BCN",
        legs: &[
            Leg {
                carrier: "Iberia",
                number: "IB 3671",
                from: "BER",
                to: "MAD",
                hour: 7,
                minute: 5,
                minutes: 170,
                aircraft: "Airbus A321",
            },
            Leg {
                carrier: "Iberia",
                number: "IB 3001",
                from: "MAD",
                to: "BCN",
                hour: 12,
                minute: 0,
                minutes: 75,
                aircraft: "Airbus A320",
            },
        ],
        price: 119.0,
        fare_family: "Economy",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: false,
        change_fee: Some(70.0),
        risk: NONE,
    },
];

const BER_JFK: &[FixtureOffer] = &[
    FixtureOffer {
        origin: "BER",
        destination: "JFK",
        legs: &[
            Leg {
                carrier: "Lufthansa",
                number: "LH 181",
                from: "BER",
                to: "FRA",
                hour: 8,
                minute: 30,
                minutes: 75,
                aircraft: "Airbus A320",
            },
            Leg {
                carrier: "Lufthansa",
                number: "LH 400",
                from: "FRA",
                to: "JFK",
                hour: 11,
                minute: 20,
                minutes: 500,
                aircraft: "Boeing 747-8",
            },
        ],
        price: 449.0,
        fare_family: "Light",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: false,
        change_fee: Some(150.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "JFK",
        legs: &[
            Leg {
                carrier: "KLM",
                number: "KL 1820",
                from: "BER",
                to: "AMS",
                hour: 9,
                minute: 40,
                minutes: 80,
                aircraft: "Embraer 190",
            },
            Leg {
                carrier: "KLM",
                number: "KL 641",
                from: "AMS",
                to: "JFK",
                hour: 12,
                minute: 30,
                minutes: 480,
                aircraft: "Boeing 787-9",
            },
        ],
        price: 489.0,
        fare_family: "Light",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: false,
        change_fee: Some(150.0),
        risk: NONE,
    },
    FixtureOffer {
        origin: "BER",
        destination: "JFK",
        legs: &[
            Leg {
                carrier: "British Airways",
                number: "BA 993",
                from: "BER",
                to: "LHR",
                hour: 7,
                minute: 55,
                minutes: 115,
                aircraft: "Airbus A320",
            },
            Leg {
                carrier: "British Airways",
                number: "BA 117",
                from: "LHR",
                to: "JFK",
                hour: 12,
                minute: 5,
                minutes: 465,
                aircraft: "Boeing 777-300ER",
            },
        ],
        price: 419.0,
        fare_family: "World Traveller",
        carry_on: 1,
        checked: 1,
        bag_fee: None,
        refundable: false,
        change_fee: Some(150.0),
        risk: NONE,
    },
];

const FIXTURES: &[&[FixtureOffer]] = &[BER_LIS, BER_LHR, BER_BCN, BER_JFK];

/// "TP 1041" → "TP 1042": the return service of a demo flight.
fn next_flight_number(number: &str) -> String {
    let split = number.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    let (prefix, digits) = number.split_at(split);
    match digits.parse::<u32>() {
        Ok(value) => format!("{prefix}{}", value + 1),
        Err(_) => number.to_string(),
    }
}

fn all_fixtures() -> impl Iterator<Item = &'static FixtureOffer> {
    FIXTURES.iter().flat_map(|route| route.iter())
}

fn depart_at(
    place: &Place,
    date: NaiveDate,
    hour: u32,
    minute: u32,
) -> Result<DateTime<Utc>, SourceError> {
    let tz: Tz = place.timezone.parse().map_err(|_| {
        SourceError::Unavailable(format!(
            "unknown timezone {} for {}",
            place.timezone, place.code
        ))
    })?;
    let naive = date
        .and_hms_opt(hour, minute, 0)
        .ok_or_else(|| SourceError::InvalidRequest("invalid departure time".into()))?;
    let local = tz.from_local_datetime(&naive).earliest().ok_or_else(|| {
        SourceError::InvalidRequest(format!("{}:{} does not exist on {date}", hour, minute))
    })?;
    Ok(local.with_timezone(&Utc))
}

fn local_rfc3339(instant: DateTime<Utc>, timezone: &str) -> Result<String, SourceError> {
    let tz: Tz = timezone
        .parse()
        .map_err(|_| SourceError::Unavailable(format!("unknown timezone {timezone}")))?;
    Ok(instant.with_timezone(&tz).to_rfc3339())
}

pub fn fill_template(
    template: &str,
    origin: &str,
    destination: &str,
    date: NaiveDate,
    adults: u8,
) -> String {
    template
        .replace("{origin}", origin)
        .replace("{destination}", destination)
        .replace("{date}", &date.format("%Y-%m-%d").to_string())
        .replace("{adults}", &adults.to_string())
}

/// The API uses this to refuse to build a fixture source without the airports
/// every fixture references.
pub fn fixture_place_codes() -> &'static [&'static str] {
    PLACE_CODES
}

pub fn known_routes() -> Vec<(&'static str, &'static str)> {
    let mut routes: Vec<(&'static str, &'static str)> = all_fixtures()
        .map(|fixture| (fixture.origin, fixture.destination))
        .collect();
    routes.sort_unstable();
    routes.dedup();
    routes
}

#[cfg(test)]
mod tests {
    use super::*;
    use faredesk_core::{SearchQuery, Sort};

    fn places() -> HashMap<String, Place> {
        let reference = faredesk_store::Reference::load().unwrap();
        PLACE_CODES
            .iter()
            .map(|code| (code.to_string(), reference.by_code(code).unwrap().clone()))
            .collect()
    }

    fn providers() -> Vec<Provider> {
        faredesk_store::load_providers().unwrap()
    }

    fn query<'a>(
        request: &'a SearchQuery,
        origin: &'a Place,
        destination: &'a Place,
    ) -> ResolvedQuery<'a> {
        ResolvedQuery {
            request,
            origin,
            destination,
        }
    }

    #[test]
    fn berlin_lisbon_has_the_demo_offers() {
        let places = places();
        let source = FixtureSource::new(places.clone(), providers());
        let request = SearchQuery {
            origin: "BER".into(),
            destination: "LIS".into(),
            depart_date: NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
            checked_bags: 1,
            sort: Sort::Best,
            ..Default::default()
        };
        let offers = source
            .search(&query(&request, &places["BER"], &places["LIS"]))
            .unwrap();
        assert_eq!(offers.len(), 5);
        assert!(offers
            .iter()
            .any(|offer| offer.price_per_traveller == 119.0));
        assert!(offers.iter().any(|offer| offer.stops() == 1));
        let ryanair = offers
            .iter()
            .find(|offer| offer.segments[0].marketing_carrier == "Ryanair")
            .unwrap();
        assert!(ryanair.risk.contains(&Risk::SelfTransfer));
        assert_eq!(ryanair.segments.len(), 2);
    }

    #[test]
    fn departure_times_carry_the_local_offset() {
        let places = places();
        let source = FixtureSource::new(places.clone(), providers());
        let request = SearchQuery {
            origin: "BER".into(),
            destination: "LIS".into(),
            depart_date: NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
            ..Default::default()
        };
        let offers = source
            .search(&query(&request, &places["BER"], &places["LIS"]))
            .unwrap();
        let tap = offers
            .iter()
            .find(|offer| offer.segments[0].marketing_carrier == "TAP Air Portugal")
            .unwrap();
        assert_eq!(tap.segments[0].depart_local, "2027-03-01T07:10:00+01:00");
        assert_eq!(tap.segments[0].arrive_local, "2027-03-01T09:30:00+00:00");
        assert_eq!(tap.duration_minutes(), 200);
    }

    #[test]
    fn summer_dates_use_summer_time() {
        let places = places();
        let source = FixtureSource::new(places.clone(), providers());
        let request = SearchQuery {
            origin: "BER".into(),
            destination: "LIS".into(),
            depart_date: NaiveDate::from_ymd_opt(2027, 7, 1).unwrap(),
            ..Default::default()
        };
        let offers = source
            .search(&query(&request, &places["BER"], &places["LIS"]))
            .unwrap();
        let tap = &offers[0];
        assert_eq!(tap.segments[0].depart_local, "2027-07-01T07:10:00+02:00");
        assert_eq!(tap.segments[0].arrive_local, "2027-07-01T09:30:00+01:00");
    }

    #[test]
    fn stops_filter_works() {
        let places = places();
        let source = FixtureSource::new(places.clone(), providers());
        let request = SearchQuery {
            origin: "BER".into(),
            destination: "LIS".into(),
            depart_date: NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
            max_stops: Some(0),
            ..Default::default()
        };
        let offers = source
            .search(&query(&request, &places["BER"], &places["LIS"]))
            .unwrap();
        assert_eq!(offers.len(), 3);
        assert!(offers.iter().all(|offer| offer.stops() == 0));
    }

    #[test]
    fn connection_duration_includes_the_layover() {
        let places = places();
        let source = FixtureSource::new(places.clone(), providers());
        let request = SearchQuery {
            origin: "BER".into(),
            destination: "LIS".into(),
            depart_date: NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
            ..Default::default()
        };
        let offers = source
            .search(&query(&request, &places["BER"], &places["LIS"]))
            .unwrap();
        let ryanair = offers
            .iter()
            .find(|offer| offer.segments.len() == 2)
            .unwrap();
        // 07:00 CET departs 06:00Z, the 09:15 connection lands 12:00Z: 360 minutes.
        assert_eq!(ryanair.duration_minutes(), 360);
        assert!(
            ryanair.duration_minutes()
                > ryanair
                    .segments
                    .iter()
                    .map(|s| s.duration_minutes)
                    .sum::<u32>()
        );
    }

    #[test]
    fn unknown_route_is_empty_not_an_error() {
        let places = places();
        let source = FixtureSource::new(places.clone(), providers());
        let reference = faredesk_store::Reference::load().unwrap();
        let request = SearchQuery::default();
        let origin = reference.by_code("BER").unwrap();
        let destination = reference.by_code("SYD").unwrap();
        assert!(source
            .search(&query(&request, origin, destination))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn templates_fill() {
        let date = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
        assert_eq!(
            fill_template(
                "https://x/{origin}/{destination}/{date}/{adults}",
                "BER",
                "LIS",
                date,
                2
            ),
            "https://x/BER/LIS/2027-03-01/2"
        );
    }
}
