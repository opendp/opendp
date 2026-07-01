#!/usr/bin/env bash
# Verifies the pinned, internally-consistent toolchain for `opendp_verified`.
#
# The Lean verification stack spans four independently-versioned components that
# MUST agree on one Lean toolchain (a mismatch produces inscrutable build
# failures — Aeneas was authored against a different Lean than SampCert/Mathlib):
#
#   * root `lean-toolchain`            (the canonical Lean version)
#   * Aeneas backend  (commit + its own lean-toolchain)
#   * Charon          (commit, pinned by Aeneas's `charon-pin`)
#   * SampCert        (commit + its own lean-toolchain)
#   * Mathlib         (inputRev in the root lake-manifest + its lean-toolchain)
#
# Because `aeneas` and `SampCert` are local *path* dependencies (no commit pin in
# the lakefile), nothing else guards their versions — this script is that guard.
# It exits non-zero on ANY discrepancy so the build refuses to proceed on a
# mismatched stack. Run it before `lake build OpenDPVerified`.
set -uo pipefail

# This script lives at rust/opendp_verified/repro/tools/; the repo root is 4 up.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"

# ---------------------------------------------------------------------------
# Canonical pins — the single source of truth. Bump these together, never alone.
# ---------------------------------------------------------------------------
LEAN_TOOLCHAIN="leanprover/lean4:v4.30.0-rc2"
AENEAS_COMMIT="a14083a6c9b0658e79d7f80cf996ad95e0864ccd"   # native v4.30.0-rc2 backend
CHARON_COMMIT="ed22146b1cd4d0b578006a58b3299d41ecbe0fd4"   # == Aeneas's charon-pin
SAMPCERT_COMMIT="9cb29f35bf56d160c199a56438add7f89542b83d" # + tracked SampCert patch
MATHLIB_INPUTREV="v4.30.0-rc2"

fail=0
check() { # label expected actual
  if [[ "$2" == "$3" ]]; then
    printf "  \033[32m✓\033[0m %-32s %s\n" "$1" "$3"
  else
    printf "  \033[31m✗\033[0m %-32s expected '%s', got '%s'\n" "$1" "$2" "$3"
    fail=1
  fi
}

ttc() { cat "$1" 2>/dev/null | tr -d '[:space:]'; }          # trimmed toolchain file
rev() { git -C "$1" rev-parse HEAD 2>/dev/null; }            # commit of a checkout

echo "Checking opendp_verified Lean toolchain pins…"

# 1. root toolchain = the canonical version
check "lean-toolchain (root)"      "$LEAN_TOOLCHAIN" "$(ttc "$repo_root/lean-toolchain")"
# 2–4. every dependent component must use the SAME Lean toolchain
check "aeneas backend toolchain"   "$LEAN_TOOLCHAIN" "$(ttc "$repo_root/aeneas/backends/lean/lean-toolchain")"
check "SampCert toolchain"         "$LEAN_TOOLCHAIN" "$(ttc "$repo_root/SampCert/lean-toolchain")"
check "mathlib toolchain"          "$LEAN_TOOLCHAIN" "$(ttc "$repo_root/.lake/packages/mathlib/lean-toolchain")"
# 5. Aeneas commit (the un-pinned path dep)
check "aeneas HEAD"                "$AENEAS_COMMIT"  "$(rev "$repo_root/aeneas")"
# 6. Charon commit, and that Aeneas asks for exactly that commit
check "charon HEAD"                "$CHARON_COMMIT"  "$(rev "$repo_root/aeneas/charon")"
check "aeneas charon-pin"          "$CHARON_COMMIT"  "$(tail -1 "$repo_root/aeneas/charon-pin" 2>/dev/null | tr -d '[:space:]')"
# 7. SampCert commit (the local patch is applied on top of this commit)
check "SampCert HEAD"              "$SAMPCERT_COMMIT" "$(rev "$repo_root/SampCert")"
# 8. Mathlib pin recorded in the root manifest
check "mathlib inputRev (manifest)" "$MATHLIB_INPUTREV" \
  "$(grep -A2 '"name": "mathlib"' "$repo_root/lake-manifest.json" 2>/dev/null \
     | grep -oE '"inputRev": "[^"]*"' | head -1 | sed -E 's/.*: "(.*)"/\1/')"

if [[ $fail -ne 0 ]]; then
  echo "" >&2
  echo "✗ Lean pin check FAILED — refusing to build on a mismatched toolchain." >&2
  echo "  Fix the offending component(s) above, or update the canonical pins in" >&2
  echo "  tools/check_lean_pins.sh (all together) if you intend to bump versions." >&2
  exit 1
fi
echo "✓ All Lean pins consistent ($LEAN_TOOLCHAIN)."
