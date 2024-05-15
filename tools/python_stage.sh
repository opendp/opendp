#!/bin/bash

# copies rust sources into Python package
# (but not target)
# 
# vendors dependencies

# exit immediately upon failure, unset vars
set -e -u

function usage() {
  echo "Usage: $(basename "$0") [-c]" >&2
}

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

function clean() {
  log "***** CLEAN *****"

  if [ -f "python/src/opendp/rust/Cargo.toml" ]; then
    run cargo clean --manifest-path python/src/opendp/rust/Cargo.toml
  fi
  # "-x" removes ignored and untracked files
  run git clean -x --force python
  pip uninstall opendp -y

  log "Restore README.md"
  run git restore python/README.md
}

function source_dir() {
  log "***** SOURCE *****"
  log "Copy lib sources into:  python/src/opendp/rust/"

  [ -d "rust/target" ] && mv rust/target target || true
  run mkdir -p python/src/opendp/rust
  run cp -r rust python/src/opendp
  [ -d "target" ] && mv target rust/target || true
  
  log "Copy README.md"
  run cp README.md python/README.md
}

if (($# == 0)); then
  clean
  source_dir

  echo "Python package is staged. Run 'pip install python/.' to build the package."
  exit 0
fi

while getopts ":cs" OPT; do
  case "$OPT" in
    c) clean ;;
    s) source_dir ;;
    *) usage && exit 1 ;;
  esac
done

shift $((OPTIND - 1))
if (($# != 0)); then usage && exit 1; fi
