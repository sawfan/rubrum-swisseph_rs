use rubrum::sign_degree::SignDegree;

use crate::adapter::error::AdapterError;

/// Normalizes an ecliptic longitude into the inclusive-exclusive range [0, 360).
///
/// Swiss Ephemeris generally returns values already in-range, but normalization makes the
/// adapter resilient to negative or >360° values when flags change.
pub fn normalize_longitude_deg(mut degrees: f64) -> Result<f64, AdapterError> {
    if !degrees.is_finite() {
        return Err(AdapterError::InvalidValue {
            field: "longitude",
            value: degrees,
            message: "must be finite",
        });
    }

    // Rust's `%` keeps the sign, so do a classic positive modulus.
    degrees %= 360.0;
    if degrees < 0.0 {
        degrees += 360.0;
    }

    // Guard against 360.0 due to floating point roundoff.
    if degrees >= 360.0 {
        degrees -= 360.0;
    }

    Ok(degrees)
}

/// Converts a Swiss Ephemeris ecliptic longitude (degrees) into a `rubrum::SignDegree`.
#[inline]
pub fn sign_degree_from_longitude_deg(degrees: f64) -> Result<SignDegree, AdapterError> {
    let normalized = normalize_longitude_deg(degrees)?;
    Ok(SignDegree::new(normalized))
}
