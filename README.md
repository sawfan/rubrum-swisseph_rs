# rubrum_swisseph

Thin adapter layer between the [`swisseph`](https://crates.io/crates/swisseph) crate (Swiss Ephemeris bindings)
and [`rubrum`](https://crates.io/crates/rubrum) domain types.

This crate is intentionally small and explicit: keep Swiss Ephemeris types at the boundary,
convert into `rubrum` types for the rest of your application.

## Features

### Ecliptic position conversions

From `swisseph::EclipticPosition`:

- `adapter::calc::placement_motion_from_ecliptic_position`
  - Longitude → `rubrum::Coordinate::SignDegree`
  - Longitude speed sign → `rubrum::Motion` (`Direct` / `Retrograde`)
  - Produces `rubrum::PlacementMotion`

- `adapter::calc::placement_ephemeris_from_ecliptic_position`
  - All of the above, plus
  - Preserves latitude / distance / speeds via `rubrum::ephemeris::EphemerisExtras`
  - Produces `rubrum::PlacementEphemeris`

### House and angle conversions

- `adapter::houses::house_cusps_from_swisseph`
  - Converts a `swisseph::Cusp` (12 cusp longitudes) into 12 `rubrum::HouseSignDegree` entries
  - Output ordering is always 1..=12

- `adapter::houses::angle_placements_from_swisseph`
  - Converts `swisseph::AscMc` into `rubrum::Placement` angles
  - Currently maps: Ascendant, Midheaven (MC), Vertex

- `adapter::houses::house_from_house_pos`
  - Converts Swiss Ephemeris `house_pos` (e.g. 1.0..12.999) into a `rubrum::House` by flooring

### Body/point mapping

- `adapter::body::occupant_from_swisseph_body`
  - Maps `swisseph::Body` → `rubrum::Occupant`
  - Supports physical bodies and common chart points (nodes/apogees, nutation)
  - Returns an error for unsupported Swiss Ephemeris body variants

### Utilities and errors

- `adapter::util::normalize_longitude_deg`
  - Normalizes a longitude to the inclusive-exclusive range `[0, 360)`

- `adapter::util::sign_degree_from_longitude_deg`
  - Convenience conversion into `rubrum::SignDegree`

- `adapter::error::AdapterError`
  - A small error type for conversion failures (`InvalidValue`, `Unsupported`, and Swiss Ephemeris errors)

## Usage

Add the dependency:

```toml
[dependencies]
rubrum_swisseph = "*"
```

Convert an ecliptic position:

```rust
use rubrum_swisseph::adapter;

let pos = swisseph::EclipticPosition {
    longitude: 123.0,
    latitude: 1.25,
    distance_in_au: 0.75,
    speed_in_longitude: 0.02,
    speed_in_latitude: -0.001,
    speed_in_distance: 0.0001,
};

let ephem = adapter::calc::placement_ephemeris_from_ecliptic_position(swisseph::Body::Mars, &pos)?;
assert!(ephem.motion.is_direct());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Convert cusps and angles:

```rust
use rubrum_swisseph::adapter;

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

let cusps = adapter::houses::house_cusps_from_swisseph(&cusp)?;
assert_eq!(cusps.len(), 12);

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

let angles = adapter::houses::angle_placements_from_swisseph(&asc_mc)?;
assert_eq!(angles.len(), 3);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Development

Typical commands:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

Notes:

- This repo may contain local symlinks under `./lib/` for development; the crate should continue to
  depend on published `rubrum`/`swisseph` crates (do not switch `Cargo.toml` to path deps).

