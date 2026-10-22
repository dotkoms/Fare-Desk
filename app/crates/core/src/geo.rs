/// Great-circle distance in kilometres.
pub fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    let (lat1, lat2) = (lat1.to_radians(), lat2.to_radians());
    let d_lat = lat2 - lat1;
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lng / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_KM * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn berlin_to_lisbon_is_about_2300_km() {
        let km = haversine_km(52.3667, 13.5033, 38.7742, -9.1342);
        assert!((km - 2310.0).abs() < 40.0, "{km}");
    }

    #[test]
    fn same_point_is_zero() {
        assert!(haversine_km(51.47, -0.4543, 51.47, -0.4543) < 1e-9);
    }
}
