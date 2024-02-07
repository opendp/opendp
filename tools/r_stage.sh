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

  run rm -f r/opendp/src/libopendp.a
  if [ -f "r/opendp/src/rust/Cargo.toml" ]; then
    run cargo clean --manifest-path r/opendp/src/rust/Cargo.toml
  fi
  run rm -rf r/opendp/src/rust r/opendp/src/binary
  run rm -rf r/opendp/opendp.Rcheck
  run rm -rf r/opendp/man
  run rm -f r/opendp/src/*.tar.xz
  run rm -f r/opendp/README.md
  run rm -f r/opendp/inst/AUTHORS
  run rm -f r/opendp/LICENSE.note
  run rm -f r/opendp/src/*.o r/opendp/src/opendp.so
  run rm -f r/opendp/opendp_*.tar.gz r/opendp/src/Makevars
  run rm -rf vendor
  run rm -rf r/opendp-docs
  rm -rf r/opendp/docs
  Rscript -e 'try(remove.packages("opendp"), silent=TRUE)'
}

function binary_tar() {
  log "***** BINARY *****"
  if [[ -f "rust/target/debug/libopendp.a" ]]; then
    log "    Detected debug library, using it to simulate precompiled binaries"
    mkdir -p binary/$(uname -m)/
    cp rust/target/debug/libopendp.a binary/$(uname -m)/
  fi

  if [[ -d "binary" ]]; then
    log "Tar binaries into:     r/opendp/src/binary.tar.xz"
    tar --create --xz --no-xattrs --file=r/opendp/src/binary.tar.xz binary
  fi
}

function source_tar() {
  log "***** SOURCE *****"
  log "Tar lib sources into:  r/opendp/src/source.tar.xz"

  [ -d "rust/target" ] && mv rust/target target || true
  # tar everything because R CMD build ignores arbitrary file patterns like .*old (like threshold...)
  tar --create --xz --no-xattrs --file=r/opendp/src/source.tar.xz rust
  [ -d "target" ] && mv target rust/target || true
}

function vendor_tar() {
  log "***** VENDOR *****"
  log "Vendor dependencies"
  run cargo vendor --manifest-path rust/Cargo.toml

  log "Tar dependencies into: r/opendp/src/vendor.tar.xz"
  tar --create --xz --no-xattrs --file=r/opendp/src/vendor.tar.xz vendor
}

function notes() {
  log "***** NOTES *****"
  if [[ ! -d "vendor" ]] && [[ -f "r/opendp/src/vendor.tar.xz" ]]; then
    mkdir vendor
    tar --extract --xz -f r/opendp/src/vendor.tar.xz -C ./vendor
  fi

  log "Prepare inst/AUTHORS and LICENSE.note"
  run Rscript tools/update_notes.R
}

function docs() {
  clean

  log "***** DOCS *****"
  # We don't directly expose any APIs from compiled code, 
  # so we don't actually have to build the binary in order to build docs.
  # To avoid the overhead of building the binary, 
  # stage the docs build in a separate package where binaries are stripped out.

  log "stage docs version of package in r/opendp-docs"
  run cp -r r/opendp r/opendp-docs
  run rm -rf r/opendp-docs/src

  log "copy README and CHANGELOG into the docs"
  run cp README.md r/opendp-docs/
  # https://pkgdown.r-lib.org/reference/build_news.html
  sed "s|^## |# Version |" CHANGELOG.md > r/opendp-docs/NEWS.md

  log "remove all traces of compiled code from the package"
  sed "/#' @useDynLib opendp, .registration = TRUE/d" r/opendp-docs/r/opendp-package.R > r/opendp-docs/r/opendp-package.R
  rm -f r/opendp-docs/configure
  rm -f r/opendp-docs/NAMESPACE

  log "build the docs, and then website"
  Rscript -e 'devtools::document("r/opendp-docs")'
  Rscript -e 'pkgdown::build_site("r/opendp-docs")'

  log "move docs to the main package"
  mv r/opendp-docs/docs r/opendp
  rm -rf r/opendp-docs

  log "R package docs are ready in r/opendp/docs/index.html"
}

if (($# == 0)); then
  clean
  binary_tar
  source_tar
  vendor_tar
  notes

  log "Build documentation"
  Rscript -e 'devtools::document("r/opendp")'

  echo "R package is staged. Run R CMD build r/opendp to build the package."
  exit 0
fi

while getopts ":cbsvnd" OPT; do
  case "$OPT" in
    c) clean ;;
    b) binary_tar ;;
    s) source_tar ;;
    v) vendor_tar ;;
    n) notes ;;
    d) docs ;;
    *) usage && exit 1 ;;
  esac
done

shift $((OPTIND - 1))
if (($# != 0)); then usage && exit 1; fi
