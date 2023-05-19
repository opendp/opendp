#!/bin/bash

# to use this as a pre-commit hook, edit .git/hooks/pre-commit (a text file) to contain this line:
# source tools/pre-commit.sh

echo "pre-commit started"
branch="$(git rev-parse --abbrev-ref HEAD)"

if [ "$branch" = "main" ]; then
  echo "Don't commit directly to main!"
  exit 1
fi

source tools/local-smoke-test.sh