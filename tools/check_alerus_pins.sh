#!/usr/bin/env bash
# Check the pinned alerus checkout used for local Verus/Alerus experiments.
set -euo pipefail

git_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
alerus_dir="$git_root/alerus"

ALERUS_URL="https://github.com/ahuoguo/alerus.git"
ALERUS_COMMIT="26809e8a27f046072739557af8fb12556d06be11"
RUST_TOOLCHAIN="1.96.0"
VERUS_RELEASE="0.2026.06.14.4ea7d0f"

rev() { git -C "$1" rev-parse HEAD 2>/dev/null; }

require_checkout() {
  local dir="$1"
  if [[ ! -e "$dir/.git" ]]; then
    echo "" >&2
    echo "missing dependency: alerus (expected at $dir)" >&2
    echo "clone it with:" >&2
    echo "  git clone $ALERUS_URL \"$dir\" && git -C \"$dir\" checkout $ALERUS_COMMIT" >&2
    exit 1
  fi

  local head
  head="$(rev "$dir")"
  if [[ "$head" != "$ALERUS_COMMIT" ]]; then
    echo "" >&2
    echo "alerus is at the wrong commit: have ${head:0:12}, need ${ALERUS_COMMIT:0:12}" >&2
    echo "realign it with:" >&2
    echo "  git -C \"$dir\" fetch origin && git -C \"$dir\" checkout $ALERUS_COMMIT" >&2
    exit 1
  fi
}

echo "Checking alerus setup pins..."
require_checkout "$alerus_dir"
printf "  \033[32m%s\033[0m %s\n" "alerus checkout:" "$ALERUS_COMMIT"
printf "  \033[32m%s\033[0m %s\n" "rust toolchain:" "$RUST_TOOLCHAIN"
printf "  \033[32m%s\033[0m %s\n" "verus release:" "$VERUS_RELEASE"
