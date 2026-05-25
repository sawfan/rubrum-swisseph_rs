use std::fmt;

/// Adapter errors produced when converting Swiss Ephemeris outputs into `rubrum` types.
#[derive(Debug)]
pub enum AdapterError {
    /// Swiss Ephemeris returned an error message.
    SwissEphemeris(String),

    /// Adapter was given a value outside the expected range.
    InvalidValue {
        field: &'static str,
        value: f64,
        message: &'static str,
    },

    /// Adapter was asked to convert a body/point/flag that isn't mapped yet.
    Unsupported(&'static str),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterError::SwissEphemeris(msg) => write!(f, "Swiss Ephemeris error: {msg}"),
            AdapterError::InvalidValue {
                field,
                value,
                message,
            } => write!(f, "Invalid {field} value ({value}): {message}"),
            AdapterError::Unsupported(what) => write!(f, "Unsupported conversion: {what}"),
        }
    }
}

impl std::error::Error for AdapterError {}

impl From<String> for AdapterError {
    fn from(value: String) -> Self {
        AdapterError::SwissEphemeris(value)
    }
}
