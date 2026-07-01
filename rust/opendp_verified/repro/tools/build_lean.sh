#!/usr/bin/env bash
# Build entry point for the `opendp_verified` Lean development.
#
# Step 1 GUARDS the build: it refuses to proceed unless the pinned toolchain is
# internally consistent (see tools/check_lean_pins.sh). This makes a version
# discrepancy — the failure mode that produces hours of inscrutable errors —
# impossible to build through silently.
set -euo pipefail

# This script lives at rust/opendp_verified/repro/tools/; the repo root (lakefile) is 4 up.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../../../.." && pwd)"

# 1. Refuse to build on a mismatched stack.
"$script_dir/check_lean_pins.sh"

# 2. Fetch the pinned Mathlib oleans (no-op if already cached).
( cd "$repo_root" && lake exe cache get )

# 3. Build the verified library.
( cd "$repo_root" && lake build OpenDPVerified )
