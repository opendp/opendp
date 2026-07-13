#!/usr/bin/env bash
# Apply the local OpenDP bridge patch to the pinned alerus checkout.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
git_root="$(cd "$script_dir/.." && pwd)"
alerus_dir="$git_root/alerus"
patch_file="$script_dir/alerus_patches/opendp-local.patch"

"$script_dir/check_alerus_pins.sh"

if [[ ! -f "$patch_file" ]]; then
  echo "missing patch file: $patch_file" >&2
  exit 1
fi

if git -C "$alerus_dir" apply --reverse --check "$patch_file" 2>/dev/null; then
  echo "alerus OpenDP patch already applied."
elif git -C "$alerus_dir" apply --check "$patch_file" 2>/dev/null; then
  git -C "$alerus_dir" apply "$patch_file"
  echo "applied alerus OpenDP patch."
else
  echo "alerus patch does not apply cleanly; inspect $alerus_dir/random/Cargo.toml" >&2
  exit 1
fi

echo ""
echo "Next:"
echo "  1. Install Rust 1.96.0 if it is not already available."
echo "  2. Download Verus release 0.2026.06.14.4ea7d0f and set VERUS_BIN_DIR to its unpacked directory."
echo "  3. Run tools/verify_alerus.sh"
