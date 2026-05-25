use rubrum::{
    coordinate::Coordinate,
    ephemeris::{EphemerisExtras, PlacementEphemeris},
    motion::Motion,
    occupant::Occupant,
    placement::{Placement, PlacementMotion},
};

use crate::adapter::{
    body::occupant_from_swisseph_body, error::AdapterError, util::sign_degree_from_longitude_deg,
};

/// Convert a Swiss Ephemeris ecliptic position into a `rubrum::PlacementMotion`.
///
/// This uses:
/// - longitude -> `Coordinate::SignDegree`
/// - speed_in_longitude sign -> `Motion`
///
/// Notes:
/// - This does not assign a house. Use `houses` + `house_pos` conversions when you want
///   `Coordinate::HouseSignDegree` placements.
pub fn placement_motion_from_ecliptic_position(
    body: swisseph::Body,
    pos: &swisseph::EclipticPosition,
) -> Result<PlacementMotion, AdapterError> {
    let occupant: Occupant = occupant_from_swisseph_body(body)?;
    let sign_degree = sign_degree_from_longitude_deg(pos.longitude)?;

    let coordinate = Coordinate::SignDegree(sign_degree);
    let placement = Placement::new(coordinate, occupant);

    let motion = if pos.speed_in_longitude < 0.0 {
        Motion::Retrograde
    } else {
        Motion::Direct
    };

    Ok(PlacementMotion::new(placement, motion))
}

/// Convert a Swiss Ephemeris ecliptic position into a `rubrum::PlacementEphemeris`.
///
/// This preserves additional ephemeris fields via `EphemerisExtras` (latitude, distance,
/// and speeds).
pub fn placement_ephemeris_from_ecliptic_position(
    body: swisseph::Body,
    pos: &swisseph::EclipticPosition,
) -> Result<PlacementEphemeris, AdapterError> {
    let placement_motion = placement_motion_from_ecliptic_position(body, pos)?;

    let extras = EphemerisExtras::from_ecliptic(
        pos.latitude,
        pos.distance_in_au,
        pos.speed_in_longitude,
        pos.speed_in_latitude,
        pos.speed_in_distance,
    );

    Ok(PlacementEphemeris::new(placement_motion, extras))
}
