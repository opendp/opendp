#!/usr/bin/env bash
# Builds the Charon + Aeneas binaries that the Charon->Aeneas Lean generation
# consumes (tools/refresh_opendp_verified_aeneas.sh):
#   * <git_root>/aeneas/charon/bin/charon   (Rust; the `charon cargo` frontend)
#   * <git_root>/aeneas/bin/aeneas          (OCaml; the LLBC -> Lean backend)
#
# Single source of truth for the OPAM-based toolchain build, used by BOTH CI and
# local setup (we deliberately do NOT use aeneas's nix flake here). Idempotent:
# if both binaries already exist it prints and returns, so a warm CI cache — or a
# local dev who already built them — is a no-op.
#
# Requires an active opam switch on OCaml 5.2 and a rustup able to fetch the
# nightly pinned by aeneas/charon/charon/rust-toolchain. If the OCaml toolchain
# is absent it fails fast with the exact commands to obtain it, rather than
# letting `make` emit an inscrutable dune error.
set -euo pipefail

# This script lives at rust/opendp_verified/repro/tools/. The shared aeneas
# checkout lives at the git repo root = four directories up.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
git_root="$(cd "$script_dir/../../../.." && pwd)"
aeneas_dir="$git_root/aeneas"
charon_bin="$aeneas_dir/charon/bin/charon"
aeneas_bin="$aeneas_dir/bin/aeneas"

log() { printf '\033[36m[build-aeneas]\033[0m %s\n' "$*"; }

# --- Fast path: both binaries present -> nothing to do. ---------------------
if [[ -x "$charon_bin" && -x "$aeneas_bin" ]]; then
  log "charon + aeneas binaries already present — skipping build."
  log "  charon: $charon_bin"
  log "  aeneas: $aeneas_bin"
  exit 0
fi

[[ -d "$aeneas_dir/.git" ]] \
  || { echo "ERROR: aeneas checkout missing at $aeneas_dir (check_lean_pins.sh prints the clone command)." >&2; exit 1; }

# GNU make is required — the aeneas Makefile uses GNU-only constructs (ifdef,
# .PHONY, pattern rules). On macOS that is `gmake`; on Linux `make` is GNU.
MAKE="$(command -v gmake || command -v make || true)"
[[ -n "$MAKE" ]] || { echo "ERROR: GNU make not found (install 'make'/'gmake')." >&2; exit 1; }

# --- OCaml/opam preflight ---------------------------------------------------
# Single source of truth for the opam dependency set — bump it alongside the
# Aeneas pin in check_lean_pins.sh if a new release needs more packages.
OPAM_DEPS=(dune menhir ppx_deriving visitors zarith yojson easy_logging \
           unionFind ocamlgraph core_unix progress domainslib ppxlib odoc)

if ! command -v opam >/dev/null 2>&1; then
  cat >&2 <<'EOF'
ERROR: opam not found. Aeneas builds with OCaml 5.2 via opam. Install opam, then:
    opam switch create aeneas-5.2 ocaml-base-compiler.5.2.1
    eval "$(opam env --switch=aeneas-5.2)"
  (on macOS also: brew install pkgconf gmp   # zarith depext)
EOF
  exit 1
fi

# Make `dune` and the installed libraries visible to this shell (no-op if the
# caller already ran `opam env`; required in CI right after `ocaml/setup-ocaml`).
eval "$(opam env 2>/dev/null)" || true

log "ensuring opam dependencies are installed (idempotent)…"
opam install -y "${OPAM_DEPS[@]}"

# --- Build charon (Rust frontend + charon-ml OCaml library) -----------------
# charon/rust-toolchain pins the nightly (with rustc-dev); rustup auto-installs
# it on first invocation. `aeneas/src/charon` symlinks to this checkout, so
# building here is what makes the `charon` OCaml library available to aeneas.
#
# charon's Rust Makefile runs `cargo fmt` as a build prerequisite, but the pinned
# nightly's component list (rustc-dev/llvm-tools/rust-src/miri) omits rustfmt —
# nix supplies it out-of-band, so the opam/rustup path must add it explicitly or
# `make` dies with "'cargo-fmt' is not installed for the toolchain". Idempotent;
# run from the crate dir so the rust-toolchain override selects the right nightly.
if command -v rustup >/dev/null 2>&1; then
  log "ensuring rustfmt is present for charon's pinned nightly…"
  ( cd "$aeneas_dir/charon/charon" && rustup component add rustfmt )
fi

log "building charon (rust + charon-ml) — first run pulls the pinned rust nightly…"
( cd "$aeneas_dir/charon" && "$MAKE" )
[[ -x "$charon_bin" ]] \
  || { echo "ERROR: charon build finished but $charon_bin is absent." >&2; exit 1; }
log "charon built: $charon_bin"

# --- Build aeneas (OCaml LLBC -> Lean backend) ------------------------------
# `make build` == build-dev -> build-bin/build-lib/build-bin-dir, which copies
# the compiled binary into aeneas/bin/. build-bin depends on check-charon, which
# is why charon must be built first.
log "building aeneas (ocaml backend)…"
( cd "$aeneas_dir" && "$MAKE" build )
[[ -x "$aeneas_bin" ]] \
  || { echo "ERROR: aeneas build finished but $aeneas_bin is absent." >&2; exit 1; }
log "aeneas built: $aeneas_bin"

log "done — charon + aeneas binaries ready."
