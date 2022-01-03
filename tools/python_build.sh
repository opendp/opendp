#!/bin/bash

# exit immediately upon failure, unset vars
set -e -u

function usage() {
  echo "Usage: $(basename "$0") [-i] [-p <PY_PLATFORM>]" >&2
  echo "Note that <PY_PLATFORM> is the whole platform tag (e.g., cp310-cp310-macosx_10_9_x86_64, not just macos)." >&2
  echo "(This is just a hook for future expansion, in case we want to do platform-specific builds.)" >&2
}

INIT=false
PY_PLATFORM=ALL
while getopts ":ip:" OPT; do
  case "$OPT" in
  i) INIT=true ;;
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

function init() {
  log "Install dependencies"
  run pip install wheel
}

function build() {
  log "Copy shared libraries"
  run mkdir -p python/src/opendp/lib
  for LIB in release/opendp_ffi.dll release/libopendp_ffi.dylib release/libopendp_ffi.so; do
    [[ -f rust/target/$LIB ]] && run cp rust/target/$LIB python/src/opendp/lib
  done

  log "Copy README.md"
  run cp README.md python/README.md

  log "Build bdist"
  if [[ $PY_PLATFORM != ALL ]]; then
    run \(cd python '&&' python setup.py bdist_wheel -d wheelhouse --plat-name="$PY_PLATFORM"\)
  else
    run \(cd python '&&' python setup.py bdist_wheel -d wheelhouse\)
  fi

  log "Restore README.md"
  run git restore python/README.md
}

if [[ $INIT == true ]]; then
  log "***** INITIALIZING *****"
  init
fi

log "***** RUNNING BUILD *****"
build
