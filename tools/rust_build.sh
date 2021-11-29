#!/bin/bash

# exit immediately upon failure, unset vars
set -e -u

function usage() {
  echo "Usage: $(basename "$0") [-irt] [-p <PLATFORM>] [-f <FEATURES>]" >&2
}

INIT=false
RELEASE_MODE=false
TEST=false
PLATFORM=macos
FEATURES=default
while getopts ":irtp:f:" OPT; do
  case "$OPT" in
  i) INIT=true ;;
  r) RELEASE_MODE=true ;;
  t) TEST=true ;;
  p) PLATFORM="$OPTARG" ;;
  f) FEATURES="$OPTARG" ;;
  *) usage && exit 1 ;;
  esac
done

shift $((OPTIND - 1))
if (($# != 0)); then usage && exit 1; fi

case "$PLATFORM" in
windows | macos | linux) true ;;
*) echo "Unknown platform $PLATFORM" >&2 && exit 1 ;;
esac

function log() {
  local FORMAT="$1"
  shift
  local MESSAGE
  MESSAGE=$(printf "$FORMAT" "$@")
  echo "$MESSAGE" >&2
}

function run() {
  local ARGS=("$@")
  log "$ %s" "${ARGS[*]}"
  eval "${ARGS[@]}"
}

function init_windows() {
  log "Set up rust, 7zip, curl"
  rustup self uninstall -y
  choco install rust
  choco install 7zip
  choco install curl

  log "Patch the Rust compiler"
  cp -f /mingw64/x86_64-w64-mingw32/lib/{*.a,*.o} /c/ProgramData/chocolatey/lib/rust/tools/lib/rustlib/x86_64-pc-windows-gnu/lib/self-contained

  log "Prepare patches for binary dependencies"
  run \(cd rust/windows '&&' bash 1_download_and_patch.sh\)

  log "Build binary dependencies"
  run \(cd rust/windows '&&' bash 2_build_dependencies.sh\)
}

function init_macos() {
  log "No prep for macos"
}

function init_linux() {
  log "Install Rust if necessary"
  if ! [ -x "$(command -v cargo)" ]; then
    run curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
    export PATH="${HOME}/.cargo/bin:${PATH}"
  fi
}

function run_cargo() {
  local ACTION="$1"
  export CARGO_TERM_COLOR=always
  if [[ $RELEASE_MODE == true ]]; then
    run cargo +stable "$ACTION" --verbose --manifest-path=rust/Cargo.toml --features="$FEATURES" --release
  else
    run cargo +stable "$ACTION" --verbose --manifest-path=rust/Cargo.toml --features="$FEATURES"
  fi
}

if [[ $INIT == true ]]; then
  log "***** INITIALIZING *****"
  case "$PLATFORM" in
  windows) init_windows ;;
  macos) init_macos ;;
  linux) init_linux ;;
  esac
fi

if [[ $TEST == true ]]; then
  log "***** RUNNING TEST *****"
  run_cargo test
fi

log "***** RUNNING BUILD *****"
run_cargo build
