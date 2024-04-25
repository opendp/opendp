#!/bin/bash

# exit immediately upon failure, unset vars
set -e -u

function usage() {
  echo "Usage: $(basename "$0") [-p <PY_PLATFORM>]" >&2
  echo "Note that <PY_PLATFORM> is the whole platform tag (e.g., cp310-cp310-macosx_10_9_x86_64, not just macos)." >&2
}

PY_PLATFORM=ALL
while getopts ":ip:" OPT; do
  case "$OPT" in
  p) PY_PLATFORM="$OPTARG" ;;
  *) usage && exit 1 ;;
  esac
done

shift $((OPTIND - 1))
if (($# != 0)); then usage && exit 1; fi

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

function build() {
  log "Copy shared libraries"
  run mkdir -p python/src/opendp/lib
  run ls rust/target/
  for LIB in release/opendp.dll release/libopendp.dylib release/libopendp.so; do
    [[ -f rust/target/$LIB ]] && run cp rust/target/$LIB python/src/opendp/lib
  done
  for ARCH in x86_64 aarch64; do
    [[ -f rust/target/$ARCH-pc-windows-gnu/release/opendp.dll ]] && run cp rust/target/$ARCH-pc-windows-gnu/release/opendp.dll python/src/opendp/lib/opendp-$ARCH.dll
    [[ -f rust/target/$ARCH-apple-darwin/release/libopendp.dylib ]] && run cp rust/target/$ARCH-apple-darwin/release/libopendp.dylib python/src/opendp/lib/libopendp-$ARCH.dylib
    [[ -f rust/target/$ARCH-unknown-linux-gnu/release/libopendp.so ]] && run cp rust/target/$ARCH-unknown-linux-gnu/release/libopendp.so python/src/opendp/lib/libopendp-$ARCH.so
  done

  log "Copy README.md"
  run cp README.md python/README.md

  log "Build bdist"
  if [[ $PY_PLATFORM != ALL ]]; then
    run \(cd python '&&' python setup.py bdist_wheel -d wheelhouse --plat-name="$PY_PLATFORM" --py-limited-api=cp38\)
  else
    run \(cd python '&&' python setup.py bdist_wheel -d wheelhouse --py-limited-api=cp38\)
  fi

  log "Restore README.md"
  run git restore python/README.md
}

log "***** RUNNING BUILD *****"
build
