#!/usr/bin/env bash
set -euo pipefail

# Prepare the manageable paper datasets. HIGGS and full MNIST are omitted by default
# because they are large/slow; run them explicitly if needed.
python -m dpclustbench.prepare_datasets \
  google_synthetic letter gas_google \
  skintype shuttle covertype hst_synthetic \
  birch2 iris adult gas \
  g2_4 g2_16 g2_64 g2_128 \
  pe_means_scale pe_means_sklearn

# Optional heavy datasets:
# python -m dpclustbench.prepare_datasets mnist_pca40 mnist_pca84
# python -m dpclustbench.prepare_datasets higgs --max-rows 1000000
