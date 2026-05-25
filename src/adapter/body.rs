use crate::adapter::error::AdapterError;

/// Convert `swisseph::Body` into `rubrum::Occupant` when possible.
///
/// Note: Swiss Ephemeris conflates some non-physical points (nodes/apogees) into its `Body`
/// enum. In `rubrum`, those are represented via `ChartPoint`.
pub fn occupant_from_swisseph_body(body: swisseph::Body) -> Result<rubrum::Occupant, AdapterError> {
    use rubrum::{Angle, Body as RBody, ChartPoint, Occupant};
    use swisseph::Body as SBody;

    let occupant = match body {
        // Physical bodies.
        SBody::Sun => Occupant::Body(RBody::Sun),
        SBody::Moon => Occupant::Body(RBody::Moon),
        SBody::Mercury => Occupant::Body(RBody::Mercury),
        SBody::Venus => Occupant::Body(RBody::Venus),
        SBody::Mars => Occupant::Body(RBody::Mars),
        SBody::Jupiter => Occupant::Body(RBody::Jupiter),
        SBody::Saturn => Occupant::Body(RBody::Saturn),
        SBody::Uranus => Occupant::Body(RBody::Uranus),
        SBody::Neptune => Occupant::Body(RBody::Neptune),
        SBody::Pluto => Occupant::Body(RBody::Pluto),
        SBody::Earth => Occupant::Body(RBody::Earth),
        SBody::Chiron => Occupant::Body(RBody::Chiron),
        SBody::Pholus => Occupant::Body(RBody::Pholus),
        SBody::Ceres => Occupant::Body(RBody::Ceres),
        SBody::Pallas => Occupant::Body(RBody::Pallas),
        SBody::Juno => Occupant::Body(RBody::Juno),
        SBody::Vesta => Occupant::Body(RBody::Vesta),

        // Nodes/apogees are chart points in rubrum.
        SBody::MeanNode => Occupant::ChartPoint(ChartPoint::MeanNode),
        SBody::TrueNode => Occupant::ChartPoint(ChartPoint::TrueNode),
        SBody::MeanApog => Occupant::ChartPoint(ChartPoint::MeanApog),
        SBody::OscuApog => Occupant::ChartPoint(ChartPoint::OscuApog),
        SBody::IntpApog => Occupant::ChartPoint(ChartPoint::IntpApog),
        SBody::IntpPerg => Occupant::ChartPoint(ChartPoint::IntpPerg),

        // Swiss Ephemeris uses this "body" id for nutation; rubrum treats it as a chart point.
        SBody::EclNut => Occupant::ChartPoint(ChartPoint::EclNut),

        // These are not currently represented in rubrum.
        SBody::Cupido
        | SBody::Hades
        | SBody::Zeus
        | SBody::Kronos
        | SBody::Apollon
        | SBody::Admetos
        | SBody::Vulkanus
        | SBody::Poseidon
        | SBody::Isis
        | SBody::Nibiru
        | SBody::Harrington
        | SBody::NeptuneLeverrier
        | SBody::NeptuneAdams
        | SBody::PlutoLowell
        | SBody::PlutoPickering
        | SBody::Astraea
        | SBody::Hebe
        | SBody::Iris
        | SBody::Flora
        | SBody::Metis
        | SBody::Hygiea
        | SBody::Urania
        | SBody::IsisAstroid
        | SBody::Hilda
        | SBody::Philosophia
        | SBody::Sophia
        | SBody::Aletheia
        | SBody::Sapientia
        | SBody::Thule
        | SBody::Ursula
        | SBody::Eros
        | SBody::CupidoAstroid
        | SBody::Hidalgo
        | SBody::Lilith
        | SBody::Amor
        | SBody::Kama
        | SBody::Aphrodite
        | SBody::Apollo
        | SBody::Damocles
        | SBody::Cruithne
        | SBody::PoseidonAstroid
        | SBody::Vulcano
        | SBody::ZeusAstroid
        | SBody::Nessus => {
            return Err(AdapterError::Unsupported(
                "swisseph::Body variant not mapped to rubrum",
            ));
        }
    };

    // Keep the import used (Angle is referenced in docs sometimes) to avoid warnings.
    let _ = Angle::Ascendant;

    Ok(occupant)
}
