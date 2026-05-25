# rubrum_swisseph unknown_web (wasm32-unknown-unknown)

Browser demo that ties together:

- `swisseph` (Swiss Ephemeris) for planetary positions
- `rubrum_swisseph` adapter to convert Swiss Ephemeris outputs into `rubrum` domain types
- `rubrum_svg` (optional feature) to render a full wheel chart to SVG

This demo targets **`wasm32-unknown-unknown`** (no WASI). Ephemeris `.se1` files are loaded by the JS host and provided to the wasm module via a Swiss Ephemeris VFS registered inside wasm.

## Prerequisites

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Run

From the repo root:

```sh
trunk serve --open --config rubrum_swisseph_rs/examples/unknown_web/web/Trunk.toml
```

## Notes

- The web assets live in `web/assets/`.
- The SVG glyph sprite is staged as `assets/glyphs_white.svg` and referenced by the theme embedded in `rubrum_render`.

