#!/usr/bin/env bash
set -euo pipefail

# This script lives at rust/opendp_verified/repro/tools/. The self-contained
# reproduction crate (its own Cargo.toml + src/ + Generated/) is `repro/` = one up;
# the shared aeneas/charon checkouts live at the git repo root = three more up.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repro_root="$(cd "${script_dir}/.." && pwd)"
git_root="$(cd "${repro_root}/../../.." && pwd)"

# The reproduction crate is self-contained under repro/: Charon `cargo`-builds it
# in place, and all generated Lean lands beside it.
crate_root="${repro_root}"
llbc_file="${repro_root}/opendp_verified.llbc"
generated_root="${crate_root}/Generated/OpenDP"
patch_root="${crate_root}/aeneas_patches"
default_charon_bin="${git_root}/aeneas/charon/bin/charon"
default_aeneas_bin="${git_root}/aeneas/bin/aeneas"
CHARON_BIN="${default_charon_bin}"
AENEAS_BIN="${default_aeneas_bin}"

if [[ ! -x "${CHARON_BIN}" ]]; then
  echo "Could not find charon at ${CHARON_BIN}. Build Aeneas in ./aeneas and rerun the refresh script." >&2
  exit 1
fi

if [[ ! -x "${AENEAS_BIN}" ]]; then
  echo "Could not find aeneas at ${AENEAS_BIN}. Build Aeneas in ./aeneas and rerun the refresh script." >&2
  exit 1
fi

if ! command -v patch >/dev/null 2>&1; then
  echo "Could not find patch. Install it and rerun the refresh script." >&2
  exit 1
fi

update_patch() {
  local template_file="$1"
  local edited_file="$2"
  local patch_file="$3"
  local template_label="$4"
  local edited_label="$5"

  if [[ ! -f "${template_file}" || ! -f "${edited_file}" ]]; then
    return 0
  fi

  local tmp_patch
  tmp_patch="$(mktemp)"
  if diff -u --label "${template_label}" --label "${edited_label}" \
      "${template_file}" "${edited_file}" > "${tmp_patch}"; then
    rm -f "${patch_file}"
  else
    mv "${tmp_patch}" "${patch_file}"
    tmp_patch=""
  fi
  if [[ -n "${tmp_patch}" && -f "${tmp_patch}" ]]; then
    rm -f "${tmp_patch}"
  fi
}

apply_patch_if_present() {
  local patch_file="$1"
  if [[ -s "${patch_file}" ]]; then
    echo "Applying $(basename "${patch_file}")"
    patch -p0 -i "${patch_file}"
  fi
}

echo "[0/4] Updating tracked patch set from current external files"
update_patch \
  "${generated_root}/FunsExternal_Template.lean" \
  "${generated_root}/FunsExternal.lean" \
  "${patch_root}/FunsExternal.patch" \
  "FunsExternal_Template.lean" \
  "FunsExternal.lean"
update_patch \
  "${generated_root}/TypesExternal_Template.lean" \
  "${generated_root}/TypesExternal.lean" \
  "${patch_root}/TypesExternal.patch" \
  "TypesExternal_Template.lean" \
  "TypesExternal.lean"

echo "[1/4] Regenerating LLBC with Charon"
(
  cd "${crate_root}"
  "${CHARON_BIN}" cargo \
    --preset=aeneas \
    --dest-file "${llbc_file}"
)

echo "[2/4] Regenerating Lean with Aeneas"
"${AENEAS_BIN}" -backend lean "${llbc_file}" \
  -dest "${repro_root}" \
  -subdir /Generated/OpenDP \
  -namespace OpenDP \
  -split-files

# Aeneas with -split-files emits Generated/OpenDP/{Funs,Types,...}.lean but NOT a
# namespace-root aggregator, and `Generated/` is gitignored — so `import
# Generated.OpenDP` (from OpenDPVerified.lean) has nothing to resolve to on a
# fresh checkout. Write the aggregator here so generation produces a complete,
# importable tree. `Funs` transitively imports `Types` and `FunsExternal`.
echo "[2b/4] Writing Generated/OpenDP.lean namespace-root aggregator"
cat > "${crate_root}/Generated/OpenDP.lean" <<'EOF'
-- Namespace-root aggregator for the Aeneas-generated `OpenDP` modules.
-- `Funs` transitively imports `Types` and `FunsExternal`.
import Generated.OpenDP.Funs
EOF

echo "[3/4] Rebuilding external companion files from templates"
cp "${generated_root}/TypesExternal_Template.lean" "${generated_root}/TypesExternal.lean"
cp "${generated_root}/FunsExternal_Template.lean" "${generated_root}/FunsExternal.lean"
(
  cd "${generated_root}"
  apply_patch_if_present "${patch_root}/TypesExternal.patch"
  apply_patch_if_present "${patch_root}/FunsExternal.patch"
)

echo "[4/4] Refresh complete"
echo "Generated Lean is in ${generated_root}"
