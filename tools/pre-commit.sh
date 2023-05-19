#!/bin/bash

branch="$(git rev-parse --abbrev-ref HEAD)"

if [ "$branch" = "main" ]; then
  echo "Don't commit directly to main!"
  exit 1
fi

source tools/local-smoke-test.sh