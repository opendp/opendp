#!/bin/bash

set -e

FAILS=0
RUNS=0
for R in `find docs/source -type f -name '*.R'`; do
    ((RUNS++))
    echo "$RUNS: $R"
    Rscript "$R" || ((FAILS++))
done
if [[ "$RUNS" -eq 0 ]]; then
    echo "FAIL: no examples found"
    exit 1
fi
if [[ "$FAILS" -ne 0 ]]; then
    echo "FAIL: $FAILS tests failed"
    exit 1
fi
echo "PASS"