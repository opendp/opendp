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
  run rm -f R/opendp/src/libopendp.a
  if [ -f "R/opendp/src/rust/Cargo.toml" ]; then
    run cargo clean --manifest-path R/opendp/src/rust/Cargo.toml
  fi
  run rm -rf R/opendp/src/rust
  run rm -rf R/opendp/opendp.Rcheck
  run rm -rf R/opendp/man
  run rm -f R/opendp/src/opendp.tar.xz R/opendp/src/vendor.tar.xz
  run rm -f R/opendp/README.md
  run rm -f R/opendp/inst/AUTHORS
  run rm -f R/opendp/LICENSE.note
  run rm -f R/opendp/src/*.o R/opendp/src/opendp.so
  run rm -f R/opendp/opendp_*.tar.gz R/opendp/src/Makevars
  run rm -rf vendor
  run rm -rf R/opendp-docs
  rm -rf R/opendp/docs
}

function docs() {
  clean
  # We don't directly expose any APIs from compiled code, 
  # so we don't actually have to build the binary in order to build docs.
  # To avoid the overhead of building the binary, 
  # stage the docs build in a separate package where binaries are stripped out.

  log "stage docs version of package in R/opendp-docs"
  run cp -r R/opendp R/opendp-docs
  run rm -rf R/opendp-docs/src

  log "copy README and CHANGELOG into the docs"
  run cp README.md R/opendp-docs/
  # run cp CHANGELOG.md R/opendp-docs/NEWS.md
  sed "s|^## |# Version |" CHANGELOG.md > R/opendp-docs/NEWS.md

  log "remove all traces of compiled code from the package"
  sed "/#' @useDynLib opendp, .registration = TRUE/d" R/opendp-docs/R/opendp-package.R > R/opendp-docs/R/opendp-package.R
  rm -f R/opendp-docs/configure
  rm -f R/opendp-docs/NAMESPACE

  log "build the docs, and then website"
  Rscript -e 'devtools::document("R/opendp-docs")'
  Rscript -e 'pkgdown::build_site("R/opendp-docs")'

  log "move docs to the main package"
  mv R/opendp-docs/docs R/opendp
  rm -rf R/opendp-docs

  log "R package docs are ready in R/opendp/docs/index.html"
}

function stage() {
  clean

  log "Vendor dependencies"
  run cargo vendor --manifest-path rust/Cargo.toml

  log "Tar library sources into R/opendp/src"
  mkdir -p R/opendp/src/rust
  [ -d rust/target ] && mv rust/target target
  # tar everything because R CMD build ignores arbitrary file patterns like .*old (like threshold...)
  tar --create --xz --no-xattrs --file=R/opendp/src/opendp.tar.xz rust
  [ -d target ] && mv target rust/target

  log "Tar dependencies into R/opendp/src"
  tar --create --xz --no-xattrs --file=R/opendp/src/vendor.tar.xz vendor

  log "Prepare inst/AUTHORS and LICENSE.note"
  run Rscript tools/update_authors.R

  log "Copy header file to R/opendp/src"
  run cp rust/opendp.h R/opendp/src/

  echo "R package is staged. Run R CMD build R/opendp to build the package."
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
