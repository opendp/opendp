#!/usr/bin/env bash
# Verifies the pinned, internally-consistent toolchain for `opendp_verified`.
#
# The Lean verification stack spans several independently-versioned components
# that MUST agree on one Lean toolchain (a mismatch produces inscrutable build
# failures — Aeneas was authored against a different Lean than SampCert/Mathlib):
#
#   * crate `lean-toolchain`           (the canonical Lean version)
#   * Aeneas backend  (pinned by build-TAG; + its own lean-toolchain)
#   * Charon          (commit, pinned by Aeneas's `charon-pin`)
#   * SampCert        (team-fork commit HASH; + its own lean-toolchain + patch)
#   * Mathlib         (inputRev in the crate lake-manifest + its lean-toolchain)
#
# `aeneas` and `SampCert` are gitignored local checkouts (the lakefile requires
# them as path deps, so nothing else guards their versions). This script is that
# guard: it PREFLIGHTS their presence at the right ref and — if missing or wrong —
# fails fast with the exact `git clone`/`checkout` command to obtain them. Then it
# checks toolchain/charon/mathlib consistency. Run it before `lake build`.
set -uo pipefail

# This script lives at tools/. The lake package root (lean-toolchain +
# lake-manifest + .lake) is the verified crate, rust/opendp_verified/; the
# gitignored aeneas/SampCert checkouts live at the git repo root = one up.
git_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
proj_dir="$git_root/rust/opendp_verified"

# ---------------------------------------------------------------------------
# Canonical pins — the single source of truth. Bump these together, never alone.
# ---------------------------------------------------------------------------
LEAN_TOOLCHAIN="leanprover/lean4:v4.30.0-rc2"
# Aeneas: pin the durable build-TAG (Aeneas's own release channel), not a bare
# commit. The tag resolves to AENEAS_COMMIT, which the toolchain checks reuse.
AENEAS_URL="https://github.com/AeneasVerif/aeneas.git"
AENEAS_TAG="build-2026.05.07.071200-a14083a6c9b0658e79d7f80cf996ad95e0864ccd"
AENEAS_COMMIT="a14083a6c9b0658e79d7f80cf996ad95e0864ccd"   # == the tag's commit
CHARON_URL="https://github.com/AeneasVerif/charon.git"     # charon is NOT an aeneas submodule; cloned separately
CHARON_COMMIT="ed22146b1cd4d0b578006a58b3299d41ecbe0fd4"   # == Aeneas's charon-pin
# SampCert: our team's fork commit (has no tags). The build also needs the vendored
# 4.30 patch (rust/opendp_verified/sampcert_patches/sampcert-4.30.patch) applied on top — build_lean.sh
# `git apply`s it after the clone. The hash alone is not enough.
SAMPCERT_URL="https://github.com/Shoeboxam/SampCert.git"
SAMPCERT_COMMIT="9cb29f35bf56d160c199a56438add7f89542b83d"
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

# Preflight a gitignored local checkout: present and at the expected commit, else
# fail FAST with the exact command to obtain it. `ref` is what to check out (a tag
# or a hash); `want` is the commit it must resolve to.
require_checkout() { # name dir url ref want submodules(yes|no)
  local name="$1" dir="$2" url="$3" ref="$4" want="$5" subs="$6"
  if [[ ! -e "$dir/.git" ]]; then
    echo "" >&2
    echo "✗ missing dependency: $name (expected at $dir)" >&2
    echo "  It is gitignored — clone it at the pinned ref:" >&2
    if [[ "$subs" == yes ]]; then
      echo "    git clone --recurse-submodules $url \"$dir\" && git -C \"$dir\" checkout $ref && git -C \"$dir\" submodule update --init --recursive" >&2
    else
      echo "    git clone $url \"$dir\" && git -C \"$dir\" checkout $ref" >&2
    fi
    exit 1
  fi
  local head; head="$(rev "$dir")"
  if [[ "$head" != "$want" ]]; then
    echo "" >&2
    echo "✗ $name is at the wrong commit: have ${head:0:12}, need ${want:0:12} ($ref)" >&2
    echo "    git -C \"$dir\" fetch --tags && git -C \"$dir\" checkout $ref" >&2
    exit 1
  fi
  printf "  \033[32m✓\033[0m %-32s %s\n" "$name checkout" "$ref"
}

echo "Checking opendp_verified Lean toolchain pins…"

# 0. Preflight the gitignored deps (fail fast with a clone command if absent).
require_checkout "aeneas"   "$git_root/aeneas"   "$AENEAS_URL"   "$AENEAS_TAG"    "$AENEAS_COMMIT"   yes
require_checkout "SampCert" "$git_root/SampCert" "$SAMPCERT_URL" "$SAMPCERT_COMMIT" "$SAMPCERT_COMMIT" no

# 1. crate toolchain = the canonical version
check "lean-toolchain (crate)"     "$LEAN_TOOLCHAIN" "$(ttc "$proj_dir/lean-toolchain")"
# 2–4. every dependent component must use the SAME Lean toolchain
check "aeneas backend toolchain"   "$LEAN_TOOLCHAIN" "$(ttc "$git_root/aeneas/backends/lean/lean-toolchain")"
check "SampCert toolchain"         "$LEAN_TOOLCHAIN" "$(ttc "$git_root/SampCert/lean-toolchain")"
check "mathlib toolchain"          "$LEAN_TOOLCHAIN" "$(ttc "$proj_dir/.lake/packages/mathlib/lean-toolchain")"
# 5. Charon commit, and that Aeneas asks for exactly that commit
check "charon HEAD"                "$CHARON_COMMIT"  "$(rev "$git_root/aeneas/charon")"
check "aeneas charon-pin"          "$CHARON_COMMIT"  "$(tail -1 "$git_root/aeneas/charon-pin" 2>/dev/null | tr -d '[:space:]')"
# 6. Mathlib pin recorded in the crate manifest
check "mathlib inputRev (manifest)" "$MATHLIB_INPUTREV" \
  "$(grep -A2 '"name": "mathlib"' "$proj_dir/lake-manifest.json" 2>/dev/null \
     | grep -oE '"inputRev": "[^"]*"' | head -1 | sed -E 's/.*: "(.*)"/\1/')"

if [[ $fail -ne 0 ]]; then
  echo "" >&2
  echo "✗ Lean pin check FAILED — refusing to build on a mismatched toolchain." >&2
  echo "  Fix the offending component(s) above, or update the canonical pins in" >&2
  echo "  this script (all together) if you intend to bump versions." >&2
  exit 1
fi
echo "✓ All Lean pins consistent ($LEAN_TOOLCHAIN)."
