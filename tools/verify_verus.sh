#!/usr/bin/env bash
# Verify OpenDP's in-tree Verus proof companions.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
git_root="$(cd "$script_dir/.." && pwd)"
manifest="$git_root/rust/opendp_verus/Cargo.toml"
verus_release="0.2026.06.14.4ea7d0f"

if ! cargo +1.96.0 --version >/dev/null 2>&1; then
  echo "Rust 1.96.0 is required. Install it with: rustup toolchain install 1.96.0" >&2
  exit 1
fi

if [[ -n "${VERUS_BIN_DIR:-}" ]]; then
  verus_bin_dir="$VERUS_BIN_DIR"
else
  verus_bin="$(find "$git_root" "$git_root/out" -maxdepth 3 -type f -name cargo-verus 2>/dev/null | head -1)"
  verus_bin_dir="${verus_bin%/cargo-verus}"
fi

if [[ -z "${verus_bin_dir:-}" || ! -x "$verus_bin_dir/cargo-verus" ]]; then
  echo "Set VERUS_BIN_DIR to an unpacked Verus ${verus_release} release containing cargo-verus." >&2
  exit 1
fi

actual_verus_release=""
if [[ -f "$verus_bin_dir/version.txt" ]]; then
  actual_verus_release="$(tr -d '\r\n' < "$verus_bin_dir/version.txt")"
fi
if [[ "$actual_verus_release" != "$verus_release" ]]; then
  echo "Verus release mismatch: found ${actual_verus_release:-unknown}, expected ${verus_release}." >&2
  exit 1
fi

echo "Verifying OpenDP Verus proofs..."
RUSTUP_TOOLCHAIN=1.96.0 "$verus_bin_dir/cargo-verus" verify --manifest-path "$manifest"
