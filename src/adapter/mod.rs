//! Conversions between `swisseph` outputs and `rubrum` types.
//!
//! This crate is intended to be a thin boundary adapter:
//! - Keep Swiss Ephemeris-specific types (`swisseph::*`) at the edge.
//! - Convert into `rubrum` domain types for the rest of the application.

pub mod error;

pub mod util;

pub mod body;
pub mod calc;
pub mod houses;
