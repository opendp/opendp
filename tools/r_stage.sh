#!/bin/bash

# copies rust sources into R package
# (but not target)
# 
# vendors dependencies
# zips contents to avoid paths

# exit immediately upon failure, unset vars
set -e -u

function usage() {
  echo "Usage: $(basename "$0") [-c]" >&2
}

CLEAN=false
while getopts ":ci" OPT; do
  case "$OPT" in
  c) CLEAN=true ;;
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

function clean() {
  log "Clean staging files"
  run rm -f R/opendpbase/src/libopendp.a
  if [ -f "R/opendpbase/src/rust/Cargo.toml" ]; then
    run cargo clean --manifest-path R/opendpbase/src/rust/Cargo.toml
  fi
  run rm -rf R/opendpbase/src/rust
  run rm -rf R/opendpbase/opendpbase.Rcheck
  run rm -rf R/opendpbase/man
  run rm -f R/opendpbase/src/opendp.tar.xz R/opendpbase/src/vendor.tar.xz
  run rm -f R/opendpbase/README.md
  run rm -f R/opendpbase/inst/AUTHORS
  run rm -f R/opendpbase/LICENSE.note
  run rm -f R/opendpbase/src/*.o R/opendpbase/src/opendpbase.so
  run rm -f R/opendpbase/opendpbase_*.tar.gz R/opendpbase/src/Makevars
  run rm -rf vendor
}

function stage() {
  clean

  mkdir -p R/opendpbase/src/rust

  log "Tar library sources into R/opendpbase/src"
  [ -d rust/target ] && mv rust/target target
  # tar everything because R CMD build ignores arbitrary file patterns like .*old (like threshold...)
  tar --create --xz --no-xattrs --file=R/opendpbase/src/opendp.tar.xz rust
  [ -d target ] && mv target rust/target

  log "Vendor dependencies"
  run cargo vendor --manifest-path rust/Cargo.toml

  log "Tar dependencies into R/opendpbase/src"
  tar --create --xz --no-xattrs --file=R/opendpbase/src/vendor.tar.xz vendor

  log "Prepare inst/AUTHORS and LICENSE.note"
  run Rscript tools/update_authors.R

  log "Copy header file to R/opendpbase/src"
  run cp rust/opendp.h R/opendpbase/src/

  log "Copy README.md to R/opendpbase"
  run cp README.md R/opendpbase/

  log "Copy CHANGELOG.md to R/opendpbase"
  run cp CHANGELOG.md R/opendpbase/NEWS.md

  echo "R package is staged. Run R CMD build R/opendpbase to build the package."
}

if [[ $CLEAN == true ]]; then
  log "***** CLEAN *****"
  clean
else
  log "***** STAGE *****"
  stage
fi
