# DP clustering benchmark harness

This is a small drop-in benchmark harness for comparing OpenDP PR #2664 against:

- `opendp_kmeans` and `opendp_kmedians` from `dp.sklearn.cluster` on your local clustering branch.
- Google Differential Privacy's LSH private coreset implementation.
- Google Research's HST clustering implementation from the KDD 2022 paper.
- A local, benchmark-oriented PE-means implementation based on the 2026 paper description.
- Nonprivate sklearn baselines.

The third-party research repos are not vendored. Run `scripts/fetch_third_party.sh` from this directory to clone them into `third_party/`, which is intentionally ignored by git.

## Quick start

From your OpenDP clustering branch:

```bash
cd path/to/opendp
python -m venv .venv-cluster-bench
source .venv-cluster-bench/bin/activate
pip install -e python
pip install -r path/to/dp_clustering_bench/requirements.txt
cd path/to/dp_clustering_bench
./scripts/fetch_third_party.sh
python -m dpclustbench.benchmark \
  --algorithms sklearn,opendp_kmeans,google_lsh,pe_means \
  --n 5000 --d 8 --k 8 --runs 5 \
  --epsilon 1.0 --delta 1e-6 \
  --radius 1.0 \
  --out results.csv
```

HST is slower and pulls in Apache Beam. Run it separately at first:

```bash
python -m dpclustbench.benchmark \
  --algorithms hst \
  --n 2000 --d 2 --k 10 --runs 2 \
  --epsilon 1.0 --delta 1e-6 \
  --radius 1.0 \
  --out hst_results.csv
```

## Notes on fairness

The harness uses one public bounded domain for all algorithms:

- Data are generated inside an L2 ball of radius `--radius`.
- OpenDP receives coordinate-wise `lower=-radius`, `upper=radius` by default.
- Google LSH receives the L2 radius.
- HST receives scalar coordinate bounds derived from the same range.

For real data, standardize or center nonprivately only when this matches your intended benchmark assumption. The code is set up for synthetic oracle-bound experiments first, because that is the cleanest way to compare algorithmic behavior.

## PE-means caveat

`pe_means.py` is an independent implementation written from the paper description, not official author code. It uses the paper's core structure: nearest-neighbor vote histogram with sensitivity one, Gaussian noise composed across PE iterations using a GDP conversion, MLE-like histogram truncation, weighted k-means selection, and Levy-flight mutation. Treat it as a useful frontier baseline/prototype, not as a proof-vetted privacy library.

## Files

- `dpclustbench/benchmark.py`: CLI runner.
- `dpclustbench/datasets.py`: synthetic mixture generation and clipping.
- `dpclustbench/metrics.py`: objectives and optional label metrics.
- `dpclustbench/algorithms/`: wrappers and local algorithms.
- `scripts/fetch_third_party.sh`: clone sparse third-party repos.

## Paper benchmark datasets

The dataset prep utility writes normalized `.npz` files under `data/prepared/` and caches raw downloads under `data/raw/`.
Both directories are ignored by git.

List available paper suites and individual datasets:

```bash
python -m dpclustbench.prepare_datasets --list
```

Prepare the Google-LSH datasets:

```bash
python -m dpclustbench.prepare_datasets google_lsh
```

Prepare the HST datasets. HIGGS is huge, so use `--max-rows` while iterating:

```bash
python -m dpclustbench.prepare_datasets skintype shuttle covertype hst_synthetic
python -m dpclustbench.prepare_datasets higgs --max-rows 1000000
```

Prepare the PE-means Table 1 shapes:

```bash
python -m dpclustbench.prepare_datasets pe_means
```

Or run the curated script, which skips full MNIST and full HIGGS by default:

```bash
./scripts/prepare_paper_datasets.sh
```

Run a prepared dataset directly:

```bash
python -m dpclustbench.benchmark \
  --dataset prepared --dataset-name letter \
  --algorithms sklearn,opendp_kmeans,google_lsh,pe_means \
  --runs 20 --epsilon 1.0 --delta 1e-6 \
  --out results/letter.csv
```

Notes:

- Google-LSH used synthetic data, MNIST neural embeddings, UCI Letter, and UCI Gas. The harness generates the synthetic set exactly from the published parameters. The MNIST loaders produce PCA surrogates (`mnist_pca40` and `mnist_pca84`) because the papers used trained neural embeddings.
- HST used SKYNTYPE/Skin Segmentation, Shuttle, Covertype, HIGGS, and a 2-D synthetic visualization dataset. The Skin public data has three BGR features plus a label; the HST paper reports `d=4`, so treat that entry carefully.
- PE-means Table 1 includes real data (`birch2`, `iris`, `adult`, `mnist`, `letter`, `gas`), G2, scale, and sklearn synthetic suites. The real datasets are downloaded where public loaders exist. The G2/scale/sklearn families are same-shape generated surrogates unless you replace them with the exact original generated files.
- All prepared data are centered, standardized, and scaled into a public L2 ball of radius `--radius` by default. Use `--no-center` and/or `--no-standardize` if you want to control preprocessing separately.
