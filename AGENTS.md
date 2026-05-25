# Agent notes

## rubrum_swisseph adapter layer

This repository exposes a small adapter library that converts Swiss Ephemeris (`swisseph`) outputs
into `rubrum` domain types.

Key modules:
- `src/adapter/calc.rs`: convert `swisseph::EclipticPosition` into `rubrum::PlacementMotion` and
  `rubrum::PlacementEphemeris` via `EphemerisExtras` (longitude -> `SignDegree`, speed sign -> `Motion`, plus latitude/distance/speeds).
- `src/adapter/houses.rs`: convert `swisseph::Cusp` to `Vec<rubrum::HouseSignDegree>` and
  `swisseph::AscMc` into angle `rubrum::Placement`s.
- `src/adapter/body.rs`: map `swisseph::Body` to `rubrum::Occupant` (physical bodies and chart points).
- `src/adapter/util.rs`: longitude normalization helpers.

Upstream (rubrum) improvements made in this repo:
- `rubrum::EphemerisExtras::from_ecliptic(...)` constructor for building extras from the common Swiss Ephemeris ecliptic tuple.

Ops performed by the agent:
- Added a top-level `README.md` describing current adapter features and usage.
- Fixed `Cargo.toml` dependency syntax so `cargo fmt` / `clippy` / `test` run successfully.

If local path dependencies are availble at `lib/` (`lib/rubrum`, `lib/swisseph`), then assume that this is to allow avante access to the local dependency folders, but do not change the Cargo.toml to use those paths directly.

