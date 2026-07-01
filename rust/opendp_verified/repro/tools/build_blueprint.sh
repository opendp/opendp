#!/usr/bin/env bash
# Build the web blueprint (dependency graph + prose) for opendp_verified.
#
# The blueprint is plasTeX-based and INDEPENDENT of the pinned Lean toolchain:
# `leanblueprint` is installed as an isolated `uv tool`, so building docs cannot
# perturb lean-toolchain / lakefile / manifests (see check_lean_pins.sh).
#
# The blueprint lives INSIDE this repro dir (repro/blueprint/), so we invoke
# `plastex` directly on it -- `leanblueprint web` would instead look for a
# `blueprint/` at the lake project root and not find it.
#
# Output: repro/blueprint/web/index.html (+ .../web/dep_graph_document.html).
#
# Prereqs (one-time):
#   uv tool install leanblueprint        # isolated; pulls plastex + deps
#   brew install graphviz                # `dot`, for the dependency graph
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bp_src="$(cd "$script_dir/../blueprint/src" && pwd)"

# plastex lives in leanblueprint's isolated uv-tool venv bin (uv exposes only the
# `leanblueprint` entry point on PATH, not `plastex`).
bp_bin="$HOME/.local/share/uv/tools/leanblueprint/bin"
if [[ ! -x "$bp_bin/plastex" ]]; then
  echo "plastex not found. Install with: uv tool install leanblueprint" >&2
  exit 1
fi

# plastex.cfg emits to ../web (i.e. repro/blueprint/web/).
( cd "$bp_src" && PATH="$bp_bin:$PATH" plastex -c plastex.cfg web.tex )
echo "Blueprint built: $(cd "$bp_src/../web" && pwd)/index.html"

# Gotcha (cost a long debug once): in a theorem-environment optional-argument
# TITLE, never use a literal '[' -- plasTeX counts raw '['/']' tokens and a bare
# '[' makes it run past the closing ']', swallowing the rest of the document into
# the title (cyclic DOM -> bare `RecursionError`, no line number). Use \lbrack.
