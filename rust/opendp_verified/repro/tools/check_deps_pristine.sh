#!/usr/bin/env bash
# Guard: the pinned dependency checkouts must be UNMODIFIED, with the single
# exception of the vendored SampCert 4.30 patch.
#
# Why: the whole point of pinning aeneas / charon / SampCert / mathlib by
# commit is that the verification is reproducible against upstream. A stray
# local edit to a dependency would silently change what we are verifying
# against. This check fails loudly if any dependency has been touched.
#
# The ONLY allowed edit is `sampcert_patches/sampcert-4.30.patch` applied to the
# SampCert checkout (a 4.30 compatibility shim we vendor; see build_lean.sh).
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
proj_dir="$(cd "$script_dir/.." && pwd)"
git_root="$(cd "$proj_dir/../../.." && pwd)"
sc_patch="$proj_dir/sampcert_patches/sampcert-4.30.patch"

fail() { echo "❌ dependency edited: $*" >&2; exit 1; }

echo "Checking dependency checkouts are pristine (only the SampCert patch is allowed)…"

# aeneas / charon / mathlib: must have NO local edits at all. We check tracked
# changes (staged + unstaged); untracked build artifacts are ignored so a local
# `.lake`/target dir does not trip the guard.
check_pristine() {  # display-name  dir
  local name="$1" dir="$2"
  [ -e "$dir/.git" ] || { echo "  – $name: absent (skipped)"; return 0; }
  if ! git -C "$dir" diff --quiet || ! git -C "$dir" diff --cached --quiet; then
    fail "$name has local modifications (no dependency may be edited — the only allowed edit is the vendored SampCert patch)"
  fi
  echo "  ✓ $name pristine"
}
check_pristine "aeneas"  "$git_root/aeneas"
check_pristine "charon"  "$git_root/aeneas/charon"
check_pristine "mathlib" "$proj_dir/.lake/packages/mathlib"

# SampCert may differ from its pinned HEAD ONLY by the vendored patch. Reversing
# the patch must therefore leave a byte-for-byte pristine tree. We reverse it,
# assert cleanliness, and re-apply (via a trap so the checkout is restored on any
# exit path, including the failure below).
sc="$git_root/SampCert"
if [ -e "$sc/.git" ]; then
  [ -f "$sc_patch" ] || fail "SampCert present but the vendored patch $sc_patch is missing"
  git -C "$sc" apply --reverse --check "$sc_patch" 2>/dev/null \
    || fail "SampCert is not exactly HEAD + the vendored patch (patch does not reverse cleanly)"
  git -C "$sc" apply --reverse "$sc_patch"
  trap 'git -C "'"$sc"'" apply "'"$sc_patch"'" 2>/dev/null || true' EXIT
  if ! git -C "$sc" diff --quiet || ! git -C "$sc" diff --cached --quiet; then
    fail "SampCert has edits BEYOND the vendored patch"
  fi
  echo "  ✓ SampCert = pinned HEAD + vendored patch only"
else
  echo "  – SampCert: absent (skipped)"
fi

echo "✓ all dependency checkouts pristine."
