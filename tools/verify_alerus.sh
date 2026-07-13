#!/usr/bin/env bash
# Verify the pinned alerus checkout against the local OpenDP crate.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
git_root="$(cd "$script_dir/.." && pwd)"
alerus_manifest="$git_root/alerus/Cargo.toml"
verus_release="0.2026.06.14.4ea7d0f"

"$script_dir/setup_alerus.sh"

if ! cargo +1.96.0 --version >/dev/null 2>&1; then
  echo "Rust 1.96.0 is required. Install it with: rustup toolchain install 1.96.0" >&2
  exit 1
fi

if [[ -n "${VERUS_BIN_DIR:-}" ]]; then
  verus_bin_dir="$VERUS_BIN_DIR"
else
  verus_bin_dir="$(find "$git_root" -maxdepth 2 -type f -name cargo-verus -path "$git_root/verus*" | head -1)"
  verus_bin_dir="${verus_bin_dir%/cargo-verus}"
fi

if [[ -z "${verus_bin_dir:-}" || ! -x "$verus_bin_dir/cargo-verus" ]]; then
  echo "Set VERUS_BIN_DIR to an unpacked Verus ${verus_release} release directory containing cargo-verus." >&2
  exit 1
fi

echo "Building the alerus random bridge crate..."
RUSTUP_TOOLCHAIN=1.96.0 cargo build -p random --manifest-path "$alerus_manifest"

echo "Running cargo-verus on alerus..."
RUSTUP_TOOLCHAIN=1.96.0 "$verus_bin_dir/cargo-verus" verify -p alerus --manifest-path "$alerus_manifest"
