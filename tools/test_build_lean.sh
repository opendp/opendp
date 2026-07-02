#!/usr/bin/env bash
# CI compile test: the opendp_verified Lean library AND its blueprint both build.
#
# Self-contained: resolves everything from THIS script's own location, so it can
# be invoked from any working directory (CI runner, editor, etc.). It shells out
# to the repo's idiomatic build entry points (build_lean.sh -> pins guard + lake
# build; build_blueprint_lean.sh -> plasTeX), so there is a single source of truth.
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
echo "==> [1/2] Lean: check pins + generate (Charon->Aeneas) + lake build OpenDPVerified ..."
lean_log="$tmp/lean.log"
# Stream build_lean.sh live — lake's per-module `[k/n] Building …` progress, the
# pins guard, and the Charon->Aeneas generation — each line prefixed with elapsed
# seconds, so a slow module is visible on a runner whose logs carry no per-line
# clock. `tee` keeps an UNPREFIXED copy in "$lean_log" for the error/sorry greps
# below (which anchor on `^error:` etc.). pipefail (set above) makes the pipeline
# report build_lean.sh's exit, not tee's/the reader's.
SECONDS=0
if ! "$script_dir/build_lean.sh" 2>&1 | tee "$lean_log" \
     | while IFS= read -r line; do printf '[+%5ds] %s\n' "$SECONDS" "$line"; done; then
  echo "---- lean build output (tail) ----" >&2
  tail -n 60 "$lean_log" >&2
  # Disambiguate the most common failure modes so the one-line FAIL is actionable
  # rather than a catch-all. The Charon->Aeneas generation step is a frequent
  # culprit on fresh checkouts (its output is gitignored and must be regenerated).
  if grep -q "Generated/ is still incomplete\|Aeneas did not emit" "$lean_log"; then
    fail "Charon->Aeneas generation did not produce Generated/ — see the [1/4]..[4/4] refresh output above (charon/aeneas binary build or 'charon cargo' likely failed)."
  elif grep -qi "opam not found\|charon build finished but\|aeneas build finished but\|GNU make not found" "$lean_log"; then
    fail "Aeneas/Charon binary build failed — see build_aeneas_bin_lean.sh output above (OCaml/opam toolchain or rust nightly)."
  elif grep -q "no such file or directory" "$lean_log" && grep -q "opendp_verified/Generated" "$lean_log"; then
    fail "lake could not find Generated/ (Charon->Aeneas output missing) — generation must run before 'lake build'."
  else
    fail "Lean build did not complete (pins mismatch, generation failure, or compile error)."
  fi
fi
if grep -qE '^error:|: error:' "$lean_log"; then
  grep -nE '^error:|: error:' "$lean_log" | sed 's#.*/rust/opendp_verified/##' >&2
  fail "Lean build reported errors."
fi
# Aeneas/Std upstream `sorry`s are expected; ours (under rust/opendp_verified/)
# must not exist.
if grep -E "declaration uses 'sorry'" "$lean_log" | grep -q 'rust/opendp_verified'; then
  grep -E "declaration uses 'sorry'" "$lean_log" | grep 'rust/opendp_verified' >&2
  fail "Lean build has a 'sorry' in our sources."
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
"$script_dir/build_blueprint_lean.sh" >"$bp_log" 2>&1 || bp_status=$?

if grep -q 'RecursionError' "$bp_log"; then
  fail "Blueprint hit a plasTeX RecursionError -- this almost always means a MALFORMED blueprint .tex: a literal '[' in a theorem-environment optional-argument title makes plasTeX run past the closing ']' (use \\lbrack instead). Check rust/opendp_verified/blueprint/src/*.tex."
fi
# "unhashable type: '<thm-env>'" from plastexdepgraph's set(nodes) means a theorem
# environment (definition/lemma/...) resolved to plasTeX's unhashable
# UnrecognizedMacro -- i.e. \newtheorem never registered it. The usual cause is a
# MISSING TeX INSTALL: plasTeX needs `kpsewhich` (TeXLive) to locate the amsthm
# .sty and the \input'd macro files. Verify texlive-* is installed in CI.
if grep -qE "unhashable type: '(definition|theorem|lemma|proposition|corollary|remark)'" "$bp_log" \
   || { grep -q 'unhashable type' "$bp_log" && grep -q 'depgraph' "$bp_log"; }; then
  fail "Blueprint: a theorem environment was unrecognized (plastexdepgraph 'unhashable type' in set(nodes)). This almost always means TeXLive is MISSING — plasTeX needs kpsewhich to resolve amsthm/.sty and \\input files so \\newtheorem can register the environments. Install texlive-* (see .github/workflows/lean-repro.yml)."
fi
if [[ "$bp_status" -ne 0 ]]; then
  echo "---- blueprint build output (tail) ----" >&2
  tail -n 40 "$bp_log" >&2
  fail "Blueprint build failed (see output above)."
fi
web_dir="$script_dir/../rust/opendp_verified/blueprint/web"
[[ -f "$web_dir/index.html" && -f "$web_dir/dep_graph_document.html" ]] \
  || fail "Blueprint output incomplete (missing index.html or dep_graph_document.html)."
echo "    Blueprint OK."

echo "PASS: Lean library and blueprint both compile."
