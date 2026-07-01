#!/usr/bin/env bash
# CI compile test: the opendp_verified Lean library AND its blueprint both build.
#
# Self-contained: resolves everything from THIS script's own location, so it can
# be invoked from any working directory (CI runner, editor, etc.). It shells out
# to the repo's idiomatic build entry points (build_lean.sh -> pins guard + lake
# build; build_blueprint.sh -> plasTeX), so there is a single source of truth.
#
# Exit 0 iff both compile cleanly. Any failure exits non-zero and prints why.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

# --------------------------------------------------------------------------- #
# 1. Lean library compiles (pins guarded, then `lake build`).
# --------------------------------------------------------------------------- #
echo "==> [1/2] Lean: check pins + lake build OpenDPVerified ..."
lean_log="$tmp/lean.log"
if ! "$script_dir/build_lean.sh" >"$lean_log" 2>&1; then
  echo "---- lean build output (tail) ----" >&2
  tail -n 40 "$lean_log" >&2
  fail "Lean build did not complete (pins mismatch or compile error)."
fi
if grep -qE '^error:|: error:' "$lean_log"; then
  grep -nE '^error:|: error:' "$lean_log" | sed 's#.*/repro/##' >&2
  fail "Lean build reported errors."
fi
# Aeneas/Std upstream `sorry`s are expected; ours (under repro/) must not exist.
if grep -E "declaration uses 'sorry'" "$lean_log" | grep -q 'opendp_verified/repro'; then
  grep -E "declaration uses 'sorry'" "$lean_log" | grep 'opendp_verified/repro' >&2
  fail "Lean build has a 'sorry' in repro sources."
fi
echo "    Lean OK."

# --------------------------------------------------------------------------- #
# 2. Blueprint compiles (plasTeX web build + dependency graph).
#    plasTeX fails fast with a RecursionError when a theorem-environment
#    optional-argument TITLE contains a literal '[' (its bracket counter runs
#    past the closing ']' and swallows the document). Treat that as a MALFORMED
#    blueprint .tex, not a transient failure, and say so.
# --------------------------------------------------------------------------- #
echo "==> [2/2] Blueprint: plasTeX web build ..."
bp_log="$tmp/blueprint.log"
bp_status=0
"$script_dir/build_blueprint.sh" >"$bp_log" 2>&1 || bp_status=$?

if grep -q 'RecursionError' "$bp_log"; then
  fail "Blueprint hit a plasTeX RecursionError -- this almost always means a MALFORMED blueprint .tex: a literal '[' in a theorem-environment optional-argument title makes plasTeX run past the closing ']' (use \\lbrack instead). Check rust/opendp_verified/repro/blueprint/src/*.tex."
fi
if [[ "$bp_status" -ne 0 ]]; then
  echo "---- blueprint build output (tail) ----" >&2
  tail -n 40 "$bp_log" >&2
  fail "Blueprint build failed (see output above)."
fi
web_dir="$script_dir/../blueprint/web"
[[ -f "$web_dir/index.html" && -f "$web_dir/dep_graph_document.html" ]] \
  || fail "Blueprint output incomplete (missing index.html or dep_graph_document.html)."
echo "    Blueprint OK."

echo "PASS: Lean library and blueprint both compile."
