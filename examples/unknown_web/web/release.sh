#!/usr/bin/env bash
# Release build helper for the rubrum_swisseph unknown_web demo.
#
# What it does:
# - Builds the Trunk bundle in release mode.
# - Optionally runs wasm-opt (Binaryen) if installed.
# - Prints raw + gzip + brotli sizes so you can track size regressions.
#
# Usage:
#   ./release.sh
#   WASM_OPT=1 ./release.sh            # run wasm-opt -Oz if available
#   OUT_DIR=dist_release ./release.sh  # write output into a custom folder

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
WEB_DIR="$ROOT_DIR/rubrum_swisseph_rs/examples/unknown_web/web"

OUT_DIR="${OUT_DIR:-dist}"
WASM_OPT="${WASM_OPT:-0}"

say() {
  printf '%s\n' "$*"
}

die() {
  say "error: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

need_cmd trunk
need_cmd gzip

human_bytes() {
  local n="$1"

  if command -v numfmt >/dev/null 2>&1; then
    # IEC units (KiB, MiB, GiB) are usually the most useful for binary artifacts.
    numfmt --to=iec-i --suffix=B --format="%.2f" "$n"
  else
    printf '%s B' "$n"
  fi
}

# brotli is optional; if missing we still report raw+gzip sizes.
HAVE_BROTLI=0
if command -v brotli >/dev/null 2>&1; then
  HAVE_BROTLI=1
fi

say "==> Building Trunk bundle (release)"
trunk build --config "$WEB_DIR/Trunk.toml" --release --dist "$OUT_DIR"

# Trunk staging puts the wasm at the dist root, per Trunk.toml post_build hook.
WASM_PATH="$WEB_DIR/$OUT_DIR/rubrum_swisseph_unknown.wasm"
[ -f "$WASM_PATH" ] || die "wasm output not found at: $WASM_PATH"

maybe_wasm_opt() {
  local wasm_in="$1"
  local wasm_out="$2"

  if [ "$WASM_OPT" != "1" ]; then
    return 0
  fi

  if ! command -v wasm-opt >/dev/null 2>&1; then
    say "==> wasm-opt not installed; skipping (install: apt-get install binaryen)"
    return 0
  fi

  say "==> Running wasm-opt -Oz"
  wasm-opt -Oz -o "$wasm_out" "$wasm_in"
}

maybe_wasm_opt "$WASM_PATH" "$WEB_DIR/$OUT_DIR/rubrum_swisseph_unknown.opt.wasm"

size_report() {
  local p="$1"
  [ -f "$p" ] || return 0

  local raw
  raw="$(wc -c < "$p" | tr -d ' ')"

  local gz
  gz="$(gzip -c "$p" | wc -c | tr -d ' ')"

  local br="n/a"
  if [ "$HAVE_BROTLI" = "1" ]; then
    br="$(brotli -q 11 -c "$p" | wc -c | tr -d ' ')"
  fi

  say "==> Size: $(basename "$p")"
  say "    raw:   $(human_bytes "$raw") ($raw bytes)"
  say "    gzip:  $(human_bytes "$gz") ($gz bytes)"

  if [ "$br" = "n/a" ]; then
    say "    brotli:n/a"
  else
    say "    brotli:$(human_bytes "$br") ($br bytes)"
  fi
}

say "==> Size report"
size_report "$WASM_PATH"
size_report "$WEB_DIR/$OUT_DIR/rubrum_swisseph_unknown.opt.wasm"

say "==> Done"
say "Output: $WEB_DIR/$OUT_DIR"

