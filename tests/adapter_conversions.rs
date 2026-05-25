use rubrum::{Angle, House, Occupant, ephemeris::EphemerisExtras};
use rubrum_swisseph::adapter;

#[test]
fn normalize_longitude_wraps() {
    assert!((adapter::util::normalize_longitude_deg(370.0).unwrap() - 10.0).abs() < 1e-9);
    assert!((adapter::util::normalize_longitude_deg(-10.0).unwrap() - 350.0).abs() < 1e-9);
}

#[test]
fn house_pos_flooring() {
    assert_eq!(
        adapter::houses::house_from_house_pos(1.0).unwrap(),
        House::First
    );
    assert_eq!(
        adapter::houses::house_from_house_pos(1.999).unwrap(),
        House::First
    );
    assert_eq!(
        adapter::houses::house_from_house_pos(12.5).unwrap(),
        House::Twelfth
    );
}

#[test]
fn cusps_convert_to_house_sign_degree() {
    let cusp = swisseph::Cusp {
        first: 10.0,
        second: 40.0,
        third: 70.0,
        fourth: 100.0,
        fifth: 130.0,
        sixth: 160.0,
        seventh: 190.0,
        eighth: 220.0,
        ninth: 250.0,
        tenth: 280.0,
        eleventh: 310.0,
        twelfth: 340.0,
    };

    let cusps = adapter::houses::house_cusps_from_swisseph(&cusp).unwrap();
    assert_eq!(cusps.len(), 12);
    assert_eq!(cusps[0].house, House::First);
    assert!((cusps[0].sign_degree.degrees - 10.0).abs() < 1e-9);
}

#[test]
fn angles_convert_to_placements() {
    let asc_mc = swisseph::AscMc {
        ascendant: 15.0,
        mc: 105.0,
        armc: 0.0,
        vertex: 200.0,
        equatorial_ascendant: 0.0,
        co_ascendant_wk: 0.0,
        co_ascendant_mm: 0.0,
        polar_ascendant: 0.0,
    };

    let placements = adapter::houses::angle_placements_from_swisseph(&asc_mc).unwrap();
    assert_eq!(placements.len(), 3);
    assert_eq!(placements[0].occupant, Occupant::Angle(Angle::Ascendant));
}

#[test]
fn ecliptic_position_to_placement_motion_sets_retrograde() {
    let pos = swisseph::EclipticPosition {
        longitude: 123.0,
        latitude: 0.0,
        distance_in_au: 1.0,
        speed_in_longitude: -0.01,
        speed_in_latitude: 0.0,
        speed_in_distance: 0.0,
    };

    let pm =
        adapter::calc::placement_motion_from_ecliptic_position(swisseph::Body::Mars, &pos).unwrap();
    assert!(pm.is_retrograde());
}

#[test]
fn ecliptic_position_to_placement_ephemeris_includes_extras() {
    let pos = swisseph::EclipticPosition {
        longitude: 123.0,
        latitude: 1.25,
        distance_in_au: 0.75,
        speed_in_longitude: 0.02,
        speed_in_latitude: -0.001,
        speed_in_distance: 0.0001,
    };

    let pe = adapter::calc::placement_ephemeris_from_ecliptic_position(swisseph::Body::Mars, &pos)
        .unwrap();

    assert_eq!(pe.extras.latitude_deg, Some(1.25));
    assert_eq!(pe.extras.distance_au, Some(0.75));
    assert_eq!(pe.extras.speed_lon_deg_per_day, Some(0.02));

    // Basic sanity: converting to a PlacementEphemeris doesn't break the existing motion rule.
    assert_eq!(
        pe.extras.motion_from_lon_speed(),
        Some(rubrum::Motion::Direct)
    );

    // And `EphemerisExtras` default is still usable.
    assert_eq!(EphemerisExtras::default().speed_lon_deg_per_day, None);
}
