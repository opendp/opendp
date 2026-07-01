#!/usr/bin/env bash
# Build entry point for the `opendp_verified` Lean development.
#
# Step 1 GUARDS the build: it refuses to proceed unless the pinned toolchain is
# internally consistent (see tools/check_lean_pins.sh). This makes a version
# discrepancy — the failure mode that produces hours of inscrutable errors —
# impossible to build through silently.
set -euo pipefail

# This script lives at rust/opendp_verified/repro/tools/. The lake sub-package
# root (its own lakefile.lean / lean-toolchain / lake-manifest.json / .lake) is
# `repro/` = one directory up. All lake commands run from there.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
proj_dir="$(cd "$script_dir/.." && pwd)"
git_root="$(cd "$proj_dir/../../.." && pwd)"

# 0. Setup — patches for 4.30: the SampCert checkout is grabbed by hash (a clean
#    v4.29 fork commit, 9cb29f35; check_lean_pins prints the clone command if it is
#    missing). We vendor its 4.30-compatibility change as a tracked patch and clobber
#    the checkout with `git apply` here. Idempotent: skip if already applied.
sampcert="$git_root/SampCert"
sc_patch="$proj_dir/sampcert_patches/sampcert-4.30.patch"
if [[ -e "$sampcert/.git" && -f "$sc_patch" ]]; then
  if git -C "$sampcert" apply --reverse --check "$sc_patch" 2>/dev/null; then
    : # already applied
  elif git -C "$sampcert" apply --check "$sc_patch" 2>/dev/null; then
    git -C "$sampcert" apply "$sc_patch" && echo "==> applied SampCert 4.30 patch."
  else
    echo "WARN: SampCert 4.30 patch neither applies cleanly nor is already applied — inspect $sampcert" >&2
  fi
fi

# 1. Refuse to build on a mismatched stack.
"$script_dir/check_lean_pins.sh"

# 2. Fetch the pinned Mathlib oleans (no-op if already cached).
( cd "$proj_dir" && lake exe cache get )

# 3. Build the verified library.
( cd "$proj_dir" && lake build OpenDPVerified )
