#!/usr/bin/env bash
# Build entry point for the `opendp_verified` Lean development.
#
# Step 1 GUARDS the build: it refuses to proceed unless the pinned toolchain is
# internally consistent (see tools/check_lean_pins.sh). This makes a version
# discrepancy — the failure mode that produces hours of inscrutable errors —
# impossible to build through silently.
set -euo pipefail

# This script lives at tools/. The lake package is rooted at the verified
# crate (rust/opendp_verified/ — lakefile.lean / lean-toolchain /
# lake-manifest.json / .lake), so all lake commands run from there. The
# gitignored aeneas/SampCert checkouts live at the git repo root.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
git_root="$(cd "$script_dir/.." && pwd)"
crate_root="$git_root/rust/opendp_verified"
proj_dir="$crate_root"

# 0. Setup — patches for 4.30: the SampCert checkout is grabbed by hash (a clean
#    v4.29 fork commit, 9cb29f35; check_lean_pins prints the clone command if it is
#    missing). We vendor its 4.30-compatibility change as a tracked patch and clobber
#    the checkout with `git apply` here. Idempotent: skip if already applied.
sampcert="$git_root/SampCert"
sc_patch="$crate_root/sampcert_patches/sampcert-4.30.patch"
if [[ -e "$sampcert/.git" && -f "$sc_patch" ]]; then
  if git -C "$sampcert" apply --reverse --check "$sc_patch" 2>/dev/null; then
    : # already applied
  elif git -C "$sampcert" apply --check "$sc_patch" 2>/dev/null; then
    git -C "$sampcert" apply "$sc_patch" && echo "==> applied SampCert 4.30 patch."
  else
    echo "WARN: SampCert 4.30 patch neither applies cleanly nor is already applied — inspect $sampcert" >&2
  fi
fi

# 1. Refuse to build if any dependency checkout was edited (only the SampCert
#    patch above is allowed). Runs while SampCert is in its patched state.
"$script_dir/check_deps_pristine_lean.sh"

# 2. Fetch the pinned Mathlib oleans (no-op if already cached). This also RESOLVES
#    the dependency graph, materialising `.lake/packages/mathlib/` — which the pin
#    guard inspects. Running it before the guard means a fresh checkout (CI or
#    local) has every dependency present, so the guard sees the real fetched
#    toolchains instead of empty strings.
( cd "$proj_dir" && lake exe cache get )

# 3. Refuse to build on a mismatched stack (now that every dep checkout exists).
"$script_dir/check_lean_pins.sh"

# 4. Ensure the Aeneas-generated Lean sources exist. `Generated/` is a build
#    artifact of the Charon->Aeneas pipeline and is gitignored, so a fresh
#    checkout — CI or a new clone — has none, and `lake build` then fails with a
#    cryptic "no such file or directory: .../opendp_verified/Generated" from the
#    `.submodules `Generated`` glob. Regenerate when absent (or when REFRESH=1),
#    which first builds the charon/aeneas binaries if they are missing. A local
#    proof-dev whose Generated/ already exists skips this — regeneration rebuilds
#    the Rust crate (minutes) — unless REFRESH=1 forces a refresh.
gen_funs="$crate_root/Generated/OpenDP/Funs.lean"
gen_root="$crate_root/Generated/OpenDP.lean"
if [[ "${REFRESH:-0}" == "1" || ! -f "$gen_funs" || ! -f "$gen_root" ]]; then
  if [[ "${REFRESH:-0}" == "1" ]]; then
    echo "==> REFRESH=1 — (re)generating Aeneas Lean sources under Generated/…"
  else
    echo "==> Generated/ is absent or incomplete — generating Aeneas Lean sources…"
    echo "    (missing: $( [[ -f "$gen_funs" ]] || echo -n "Generated/OpenDP/Funs.lean " )$( [[ -f "$gen_root" ]] || echo -n "Generated/OpenDP.lean" ))"
  fi
  "$script_dir/build_aeneas_bin_lean.sh"
  "$script_dir/refresh_opendp_verified_aeneas.sh"
  if [[ ! -f "$gen_funs" || ! -f "$gen_root" ]]; then
    echo "" >&2
    echo "ERROR: Charon->Aeneas refresh ran but Generated/ is still incomplete." >&2
    echo "       Expected both:" >&2
    echo "         $gen_funs        $( [[ -f "$gen_funs" ]] && echo '(ok)' || echo '(MISSING)' )" >&2
    echo "         $gen_root  $( [[ -f "$gen_root" ]] && echo '(ok)' || echo '(MISSING)' )" >&2
    echo "       Inspect the [1/4]..[4/4] refresh output above (charon cargo / aeneas)." >&2
    exit 1
  fi
  echo "==> Aeneas Lean sources generated."
else
  echo "==> Generated/ present — skipping Charon->Aeneas refresh (set REFRESH=1 to force)."
fi

# 5. Build the verified library.
( cd "$proj_dir" && lake build OpenDPVerified )

# 6. Assert the verified chain is COMPLETE: every end-to-end sampler theorem
#    (stages 2-9, uniform through discrete Gaussian) exists and depends only on
#    the sanctioned axiom set, and no handwritten source contains a `sorry`.
#    `lake build` alone enforces neither (a sorry is only a warning; a deleted
#    theorem doesn't break compilation).
"$script_dir/check_verified_chain_lean.sh"
