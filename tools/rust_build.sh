#!/bin/bash

# exit immediately upon failure, unset vars
set -e -u

function usage() {
  echo "Usage: $(basename "$0") [-irt] [-p <PLATFORM>] [-c <TOOLCHAIN>] [-f <FEATURES>]" >&2
}

INIT=false
RELEASE_MODE=false
TEST=false
PLATFORM=UNSET
TOOLCHAIN=stable
FEATURES=default
while getopts ":irtp:c:f:" OPT; do
  case "$OPT" in
  i) INIT=true ;;
  r) RELEASE_MODE=true ;;
  t) TEST=true ;;
  p) PLATFORM="$OPTARG" ;;
  c) TOOLCHAIN="$OPTARG" ;;
  f) FEATURES="$OPTARG" ;;
  *) usage && exit 1 ;;
  esac
done

shift $((OPTIND - 1))
if (($# != 0)); then usage && exit 1; fi

function log() {
  local FORMAT="$1"; shift
  local MESSAGE
  MESSAGE=$(printf "$FORMAT" "$@")
  echo "$MESSAGE" >&2
}

function run() {
  local CMD=("$@")
  log "$ %s" "${CMD[*]}"
  eval "${CMD[@]}"
}

function guess_platform() {
  case "$OSTYPE" in
    msys*|cygwin*) echo "windows" ;;
    darwin*)       echo "macos" ;;
    linux*)        echo "linux" ;;
    *)             echo "$OSTYPE" ;;
  esac
}

function init_windows() {
  log "Install Rust toolchain"
  export PATH="$PATH:/c/Rust/.cargo/bin"
  run rustup toolchain install stable-x86_64-pc-windows-gnu
  run rustup set default-host x86_64-pc-windows-gnu

# The lines below copy components from the platform installation of mingw into the Rust toolchain,
# and build the gmp & mpfr native libraries manually. This isn't necessary anymore, as the build script
# in the gmp-mpfr-sys crate works with the current version of msys2.
#
#  log "Patch the Rust compiler"
#  run cp -rp "$RUSTUP_DIR"/toolchains/stable-x86_64-pc-windows-gnu rust/toolchain
#  run rustup toolchain link "$TOOLCHAIN" rust/toolchain
#  run cp -f /mingw64/x86_64-w64-mingw32/lib/{*.a,*.o} rust/toolchain/lib/rustlib/x86_64-pc-windows-gnu/lib/self-contained
#
#  log "Prepare patches for binary dependencies"
#  run \(cd rust/windows '&&' bash 1_download_and_patch.sh\)
#
#  log "Build binary dependencies"
#  export RUSTFLAGS="-L native=D:\a\opendp\opendp\rust\toolchain\lib\rustlib\x86_64-pc-windows-gnu\lib\self-contained"
#  run \(cd rust/windows '&&' bash 2_build_dependencies.sh\)
}

function init_macos() {
  log "No prep for macos"
}

function init_linux() {
  log "Install Rust if necessary"
  if ! [ -x "$(command -v cargo)" ]; then
    run curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
    source ~/.cargo/env
  fi
  log "Install Perl IPC-CMD"
  run yum -y install perl-IPC-Cmd
}

function run_cargo() {
  local ACTION=$1
  local CMD=(cargo)
  [[ $TOOLCHAIN != UNSET ]] && CMD+=(+"$TOOLCHAIN")
  CMD+=(--verbose --verbose --color=always $ACTION --manifest-path=rust/Cargo.toml --features="$FEATURES")
  [[ $RELEASE_MODE == true ]] && CMD+=(--release)
  run "${CMD[@]}"
}

if [[ $PLATFORM == UNSET ]]; then
  PLATFORM=`guess_platform`
fi

if [[ $INIT == true ]]; then
  log "***** INITIALIZING *****"
  case     "$PLATFORM" in
  windows) init_windows ;;
  macos)   init_macos ;;
  linux)   init_linux ;;
  *)       echo"Unknown platform $PLATFORM" >&2 && exit 1 ;;
  esac
fi

if [[ $TEST == true ]]; then
  log "***** RUNNING TEST *****"
  run_cargo test
fi

log "***** RUNNING BUILD *****"
run_cargo build
