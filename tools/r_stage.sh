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
DOCS=false
while getopts ":cd" OPT; do
  case "$OPT" in
  c) CLEAN=true ;;
  d) DOCS=true ;;
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
  run rm -rf R/opendpbase-docs
  rm -rf R/opendpbase/docs
}

function docs() {
  clean
  # We don't directly expose any APIs from compiled code, 
  # so we don't actually have to build the binary in order to build docs.
  # To avoid the overhead of building the binary, 
  # stage the docs build in a separate package where binaries are stripped out.

  log "stage docs version of package in R/opendpbase-docs"
  run cp -r R/opendpbase R/opendpbase-docs
  run rm -rf R/opendpbase-docs/src

  log "copy README and CHANGELOG into the docs"
  run cp README.md R/opendpbase-docs/
  run cp CHANGELOG.md R/opendpbase-docs/NEWS.md

  log "remove all traces of compiled code from the package"
  sed "/#' @useDynLib opendpbase, .registration = TRUE/d" R/opendpbase-docs/R/opendpbase-package.R > R/opendpbase-docs/R/opendpbase-package.R
  rm -f R/opendpbase-docs/NAMESPACE

  log "build the docs, and then website"
  Rscript -e 'devtools::document("R/opendpbase-docs")'
  Rscript -e 'pkgdown::build_site("R/opendpbase-docs")'

  log "move docs to the main package"
  mv R/opendpbase-docs/docs R/opendpbase

  log "R package docs are ready in R/opendpbase/docs/index.html"
  # open R/opendpbase/docs/index.html
}

function stage() {
  clean

  log "Vendor dependencies"
  run cargo vendor --manifest-path rust/Cargo.toml

  log "Tar library sources into R/opendpbase/src"
  mkdir -p R/opendpbase/src/rust
  [ -d rust/target ] && mv rust/target target
  # tar everything because R CMD build ignores arbitrary file patterns like .*old (like threshold...)
  tar --create --xz --no-xattrs --file=R/opendpbase/src/opendp.tar.xz rust
  [ -d target ] && mv target rust/target

  log "Tar dependencies into R/opendpbase/src"
  tar --create --xz --no-xattrs --file=R/opendpbase/src/vendor.tar.xz vendor

  log "Prepare inst/AUTHORS and LICENSE.note"
  run Rscript tools/update_authors.R

  log "Copy header file to R/opendpbase/src"
  run cp rust/opendp.h R/opendpbase/src/

  echo "R package is staged. Run R CMD build R/opendpbase to build the package."
}

if [[ $CLEAN == true ]]; then
  log "***** CLEAN *****"
  clean
elif [[ $DOCS == true ]]; then
  log "***** DOCS *****"
  docs
else
  log "***** STAGE *****"
  stage
fi
