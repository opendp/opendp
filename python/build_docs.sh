#!/usr/bin/env bash
set -e

OPENDP_VERSION=0.1.0

rm -r docs_temp || true
sphinx-apidoc -fFe -H opendp -A "OpenDP" -V $OPENDP_VERSION -o docs_temp/source/ src/opendp --templatedir templates/
export OPENDP_LIB_DIR=/Users/michael/openDP/openDP/rust/target/debug
# destroy prior generated documentation and completely rebuild
rm -r docs || true
mkdir -p docs
sphinx-build -b html docs_temp/source/ docs
touch docs/.nojekyll

rm -r docs_temp || true