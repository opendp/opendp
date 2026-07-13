#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "$ROOT/third_party"
cd "$ROOT/third_party"

if [ ! -d differential-privacy/.git ]; then
  git clone --depth 1 --filter=blob:none --sparse https://github.com/google/differential-privacy.git
  (cd differential-privacy && git sparse-checkout set learning python)
fi

if [ ! -d google-research/.git ]; then
  git clone --depth 1 --filter=blob:none --sparse https://github.com/google-research/google-research.git
  (cd google-research && git sparse-checkout set hst_clustering)
fi

# Optional: PE-means uses synthetic scale datasets described as generated with
# clusterGeneration in the FastLloyd line of work. Clone the repo so you can
# inspect/reuse scripts/generator.R if you want exact FastLloyd-style data.
if [ ! -d FastLloyd/.git ]; then
  git clone --depth 1 https://github.com/D-Diaa/FastLloyd.git || true
fi

cat <<MSG
Fetched third-party sources under:
  $ROOT/third_party/differential-privacy
  $ROOT/third_party/google-research
  $ROOT/third_party/FastLloyd  (optional, may fail if repo layout changes)
MSG
