use rubrum::{
    angle::Angle, coordinate::Coordinate, house::House, house_sign_degree::HouseSignDegree,
    occupant::Occupant, placement::Placement,
};

use crate::adapter::{error::AdapterError, util::sign_degree_from_longitude_deg};

/// Convert a Swiss Ephemeris `Cusp` (12 cusp longitudes) into `rubrum` cusp coordinates.
///
/// Output is always 12 items in 1..=12 order.
pub fn house_cusps_from_swisseph(
    cusp: &swisseph::Cusp,
) -> Result<Vec<HouseSignDegree>, AdapterError> {
    let entries = [
        (House::First, cusp.first),
        (House::Second, cusp.second),
        (House::Third, cusp.third),
        (House::Fourth, cusp.fourth),
        (House::Fifth, cusp.fifth),
        (House::Sixth, cusp.sixth),
        (House::Seventh, cusp.seventh),
        (House::Eighth, cusp.eighth),
        (House::Ninth, cusp.ninth),
        (House::Tenth, cusp.tenth),
        (House::Eleventh, cusp.eleventh),
        (House::Twelfth, cusp.twelfth),
    ];

    let mut out = Vec::with_capacity(12);
    for (house, lon) in entries {
        let sign_degree = sign_degree_from_longitude_deg(lon)?;
        out.push(HouseSignDegree::new(house, sign_degree));
    }

    Ok(out)
}

/// Convert Swiss Ephemeris ascendant/MC/etc output into `rubrum` angle placements.
///
/// Swiss Ephemeris provides additional points (ARMC, equatorial ascendant, co-ascendants,
/// polar ascendant). This adapter currently maps only:
/// - Ascendant
/// - Midheaven (MC)
/// - Vertex
pub fn angle_placements_from_swisseph(
    asc_mc: &swisseph::AscMc,
) -> Result<Vec<Placement>, AdapterError> {
    let mut placements = Vec::with_capacity(3);

    let asc = sign_degree_from_longitude_deg(asc_mc.ascendant)?;
    placements.push(Placement::new(
        Coordinate::SignDegree(asc),
        Occupant::Angle(Angle::Ascendant),
    ));

    let mc = sign_degree_from_longitude_deg(asc_mc.mc)?;
    placements.push(Placement::new(
        Coordinate::SignDegree(mc),
        Occupant::Angle(Angle::Midheaven),
    ));

    let vx = sign_degree_from_longitude_deg(asc_mc.vertex)?;
    placements.push(Placement::new(
        Coordinate::SignDegree(vx),
        Occupant::Angle(Angle::Vertex),
    ));

    Ok(placements)
}

/// Convert the Swiss Ephemeris `house_pos` numeric return into a `rubrum::House`.
///
/// `house_pos` returns a floating point house position, e.g. 1.0..12.999.
/// This adapter floors it into a 1-based house index.
pub fn house_from_house_pos(house_pos: f64) -> Result<House, AdapterError> {
    if !house_pos.is_finite() {
        return Err(AdapterError::InvalidValue {
            field: "house_pos",
            value: house_pos,
            message: "must be finite",
        });
    }

    if !(1.0..13.0).contains(&house_pos) {
        return Err(AdapterError::InvalidValue {
            field: "house_pos",
            value: house_pos,
            message: "must be within [1, 13)",
        });
    }

    let idx = house_pos.floor() as i32;
    House::from_1_based_i32(idx).ok_or(AdapterError::InvalidValue {
        field: "house_pos",
        value: house_pos,
        message: "did not map to a 1..=12 house",
    })
}
