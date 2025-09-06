#!/bin/bash

# exit immediately upon failure, unset vars
set -e -u

function usage() {
  echo "Usage: $(basename "$0") [-irtn] [-p <PLATFORM>] [-c <TOOLCHAIN>] [-g <TARGET>] [-f <FEATURES>]" >&2
}

INIT=false
RELEASE_MODE=false
TEST=false
CHECK=false
PLATFORM=UNSET
TARGET=UNSET
TOOLCHAIN=stable
FEATURES=default
while getopts ":irtnp:c:g:f:" OPT; do
  case "$OPT" in
  i) INIT=true ;;
  r) RELEASE_MODE=true ;;
  t) TEST=true ;;
  n) CHECK=true ;;
  p) PLATFORM="$OPTARG" ;;
  c) TOOLCHAIN="$OPTARG" ;;
  g) TARGET="$OPTARG" ;;
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
  # If we run rustup immediately, it sometimes fails with "/c/Users/runneradmin/.cargo/bin/rustup: Device or resource busy".
  # So we hack around it by sleeping to let things settle. (Possible cause: https://github.com/rust-lang/rustup/issues/3189)
  run sleep 5
  run rustup toolchain install stable-x86_64-pc-windows-gnu
  run sleep 5
  run rustup set default-host x86_64-pc-windows-gnu
  run sleep 5
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
}

function run_cargo() {
  local ACTION=$1
  local CMD=(cargo)
  [[ $TOOLCHAIN != UNSET ]] && CMD+=(+"$TOOLCHAIN")
  CMD+=(--verbose --verbose --color=always $ACTION)
  [[ $TARGET != UNSET ]] && CMD+=(--target "$TARGET")
  CMD+=(--manifest-path=rust/Cargo.toml --features="$FEATURES")
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
  *)       echo "Unknown platform $PLATFORM" >&2 && exit 1 ;;
  esac

  [[ $TARGET != UNSET ]] && rustup target add $TARGET
fi

if [[ $TEST == true ]]; then
  log "***** RUNNING TEST *****"
  run_cargo test
fi

if [[ $CHECK == true ]]; then
  log "***** RUNNING CHECK *****"
  run_cargo check
else
  log "***** RUNNING BUILD *****"
  run_cargo build
fi
