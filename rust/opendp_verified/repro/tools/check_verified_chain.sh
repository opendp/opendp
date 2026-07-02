#!/usr/bin/env bash
# Completion guard for the verified sampler chain (see check_chain.lean for the
# Lean-side checks). Fails the build if:
#   1. any handwritten proof source contains a `sorry` token (the Lean-side
#      axiom check cannot catch these: the vendored Aeneas stdlib already
#      contributes `sorryAx` to every theorem, masking new ones), or
#   2. any end-to-end theorem is missing or depends on an axiom outside the
#      sanctioned trust surface.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
proj_dir="$(cd "$script_dir/.." && pwd)"

# 1. No `sorry` tokens in handwritten sources. Backtick-quoted doc mentions
#    (`sorry`) are excluded by the character classes.
if grep -rn --include='*.lean' -E '(^|[^`A-Za-z_])sorry([^`A-Za-z_]|$)' "$proj_dir/src"; then
  echo "ERROR: 'sorry' found in handwritten proof sources (above)." >&2
  exit 1
fi

# 2. `axiom` declarations may only live in the designated trust files (the
#    probabilistic core and the external specs). An axiom anywhere else is an
#    unproven claim hiding outside the audited trust surface.
allowed_axiom_files='core/primitives/semantics.lean|core/externals/dashu.lean|core/externals/openssl_rand.lean|core/externals/core_num_usize.lean|samplers/bernoulli/exp1.lean|samplers/uniform/semantics.lean'
if grep -rn --include='*.lean' -E '^[[:space:]]*axiom[[:space:]]' "$proj_dir/src"     | grep -vE "src/($allowed_axiom_files):"; then
  echo "ERROR: 'axiom' declared outside the designated trust files (above)." >&2
  exit 1
fi

# 3. End-to-end theorem existence, pinned statements, + axiom allowlist.
( cd "$proj_dir" && lake env lean "$script_dir/check_chain.lean" )

echo "==> verified-chain guard passed: chain complete, trust surface sanctioned."
