#!/bin/sh
set -eu

MANIFEST_PATH="${1:?expected manifest path}"
RUST_FEATURES="${2:-}"

case ",${RUST_FEATURES}," in
    *,polars,*|*,polars-ffi,*)
        exit 0
        ;;
esac

TMP_PATH="${MANIFEST_PATH}.tmp"
awk '
BEGIN { skip = 0 }
/^\[patch\.crates-io\]$/ { skip = 1; next }
skip && /^\[/ { skip = 0 }
!skip { print }
' "${MANIFEST_PATH}" > "${TMP_PATH}"
mv "${TMP_PATH}" "${MANIFEST_PATH}"
