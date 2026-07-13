from __future__ import annotations

import argparse
import gzip
import io
import json
import re
import sys
import urllib.request
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable, Optional

import numpy as np
import pandas as pd

from .datasets import (
    clip_l2,
    make_google_lsh_synthetic,
    make_scale_surrogate,
    make_sklearn_blobs_paper_shape,
    sample_ball,
)


@dataclass(frozen=True)
class PreparedDataset:
    name: str
    suite: str
    paper: str
    n: Optional[int]
    d: Optional[int]
    k: Optional[int]
    source: str
    note: str = ""


CATALOG: dict[str, PreparedDataset] = {
    # Chang-Kamath / Google LSH benchmarks.
    "google_synthetic": PreparedDataset(
        "google_synthetic",
        "google_lsh",
        "Chang-Kamath / Google DP LSH private coreset",
        100_000,
        100,
        64,
        "generated",
        "Exact shape and generation parameters from the Google DP clustering README.",
    ),
    "mnist_pca40": PreparedDataset(
        "mnist_pca40",
        "google_lsh",
        "Chang-Kamath / Google DP LSH private coreset",
        70_000,
        40,
        10,
        "OpenML mnist_784 + PCA surrogate",
        "Paper used 40-D neural embeddings; this is a cheap PCA40 surrogate unless you replace it with your own embeddings.",
    ),
    "letter": PreparedDataset(
        "letter",
        "google_lsh,pe_means",
        "Google-LSH and PE-means",
        20_000,
        16,
        26,
        "OpenML/UCI Letter Recognition",
        "Centered/rescaled in preparation step.",
    ),
    "gas_google": PreparedDataset(
        "gas_google",
        "google_lsh",
        "Chang-Kamath / Google DP LSH private coreset",
        36_733,
        11,
        None,
        "UCI Gas Turbine CO and NOx Emission",
        "Drops year if present to match the Google README's 11 sensor-measure description.",
    ),
    # HST paper datasets.
    "skintype": PreparedDataset(
        "skintype",
        "hst",
        "Cohen-Addad et al. HST clustering",
        245_057,
        3,
        2,
        "UCI Skin Segmentation",
        "HST paper reports d=4, but the public UCI data has three BGR features plus label. This loader uses the three features.",
    ),
    "shuttle": PreparedDataset(
        "shuttle",
        "hst",
        "Cohen-Addad et al. HST clustering",
        58_000,
        9,
        7,
        "OpenML/UCI Statlog Shuttle",
    ),
    "covertype": PreparedDataset(
        "covertype",
        "hst",
        "Cohen-Addad et al. HST clustering",
        581_012,
        54,
        7,
        "sklearn fetch_covtype / UCI Covertype",
    ),
    "higgs": PreparedDataset(
        "higgs",
        "hst",
        "Cohen-Addad et al. HST clustering",
        11_000_000,
        28,
        2,
        "UCI HIGGS",
        "Huge; use --max-rows while iterating.",
    ),
    "hst_synthetic": PreparedDataset(
        "hst_synthetic",
        "hst",
        "Cohen-Addad et al. HST clustering",
        5_000,
        2,
        None,
        "generated surrogate",
        "Public HST paper cites a 2-D synthetic visualization set; this generates a transparent 2-D surrogate.",
    ),
    # PE-means real datasets.
    "birch2": PreparedDataset(
        "birch2",
        "pe_means",
        "PE-means",
        25_000,
        2,
        100,
        "clustering-data-v1/sipu birch2",
        "Samples 25k from the 100k Birch2 set, as described by PE-means.",
    ),
    "iris": PreparedDataset("iris", "pe_means", "PE-means", 150, 4, 3, "sklearn iris"),
    "adult": PreparedDataset(
        "adult",
        "pe_means",
        "PE-means",
        48_842,
        6,
        3,
        "OpenML Adult numeric columns",
        "Uses six numeric Adult columns, matching the d=6 table entry; labels are income classes, not k=3.",
    ),
    "mnist_pca84": PreparedDataset(
        "mnist_pca84",
        "pe_means",
        "PE-means",
        70_000,
        84,
        10,
        "OpenML mnist_784 + PCA surrogate",
        "PE-means reports d=84 and says LeNet5 representations; this is a PCA84 surrogate unless replaced by LeNet embeddings.",
    ),
    "gas": PreparedDataset(
        "gas",
        "pe_means",
        "PE-means",
        36_733,
        12,
        6,
        "UCI Gas Turbine CO and NOx Emission",
        "Keeps all numeric columns after merging yearly CSVs, including year when present.",
    ),
}

for dim in (4, 16, 64, 128):
    CATALOG[f"g2_{dim}"] = PreparedDataset(
        f"g2_{dim}",
        "pe_means",
        "PE-means",
        2048,
        dim,
        2,
        "generated G2 surrogate",
        "The paper references G2 sets from a clustering repository; this generates a same-shape two-Gaussian surrogate.",
    )
for k in (4, 16, 64):
    for dim in (4, 16, 64, 128):
        n = 9999 if (k, dim) == (4, 128) else 10000
        if (k, dim) in {(16, 128), (64, 16), (64, 64)}:
            n = 10001
        if (k, dim) == (64, 4):
            n = 10002
        if (k, dim) == (64, 128):
            n = 10016
        CATALOG[f"scale_{k}_{dim}"] = PreparedDataset(
            f"scale_{k}_{dim}",
            "pe_means",
            "PE-means",
            n,
            dim,
            k,
            "generated scale surrogate",
            "Same shape as PE-means Table 1; exact paper data was generated with R clusterGeneration/FastLloyd scripts.",
        )
for dim in (4, 16, 64, 128):
    for k in (4, 16, 64):
        CATALOG[f"sklearn_{k}_{dim}"] = PreparedDataset(
            f"sklearn_{k}_{dim}",
            "pe_means",
            "PE-means",
            20_000,
            dim,
            k,
            "generated sklearn blobs",
            "Same shape as PE-means Table 1; generated with sklearn-style isotropic Gaussian blobs.",
        )

SUITES = {
    "google_lsh": ["google_synthetic", "mnist_pca40", "letter", "gas_google"],
    "hst": ["skintype", "shuttle", "covertype", "higgs", "hst_synthetic"],
    "pe_means_real": ["birch2", "iris", "adult", "mnist_pca84", "letter", "gas"],
    "pe_means_g2": [f"g2_{dim}" for dim in (4, 16, 64, 128)],
    "pe_means_scale": [f"scale_{k}_{dim}" for k in (4, 16, 64) for dim in (4, 16, 64, 128)],
    "pe_means_sklearn": [f"sklearn_{k}_{dim}" for dim in (4, 16, 64, 128) for k in (4, 16, 64)],
}
SUITES["pe_means"] = (
    SUITES["pe_means_real"]
    + SUITES["pe_means_g2"]
    + SUITES["pe_means_scale"]
    + SUITES["pe_means_sklearn"]
)
SUITES["all"] = sorted(CATALOG)


RAW_ROOT = "https://raw.githubusercontent.com/gagolews/clustering-data-v1/master"


def download_bytes(url: str, cache_path: Path, *, force: bool = False) -> bytes:
    cache_path.parent.mkdir(parents=True, exist_ok=True)
    if cache_path.exists() and not force:
        return cache_path.read_bytes()
    print(f"download {url}", file=sys.stderr)
    req = urllib.request.Request(url, headers={"User-Agent": "dp-clustering-bench/0.1"})
    with urllib.request.urlopen(req, timeout=120) as r:
        data = r.read()
    cache_path.write_bytes(data)
    return data


def as_numeric_frame(df: pd.DataFrame) -> pd.DataFrame:
    out = df.copy()
    for col in out.columns:
        out[col] = pd.to_numeric(out[col], errors="coerce")
    out = out.dropna(axis=1, how="all").dropna(axis=0, how="any")
    return out


def encode_labels(y) -> Optional[np.ndarray]:
    if y is None:
        return None
    from sklearn.preprocessing import LabelEncoder

    return LabelEncoder().fit_transform(np.asarray(y).astype(str))


def center_and_bound(
    x: np.ndarray,
    *,
    radius: float,
    standardize: bool,
    center: bool,
) -> tuple[np.ndarray, float]:
    x = np.asarray(x, dtype=float)
    if center:
        x = x - np.mean(x, axis=0, keepdims=True)
    if standardize:
        scale = np.std(x, axis=0, keepdims=True)
        scale = np.where(scale <= 1e-12, 1.0, scale)
        x = x / scale
    norms = np.linalg.norm(x, axis=1)
    max_norm = float(np.max(norms)) if norms.size else 1.0
    if max_norm <= 0.0:
        max_norm = 1.0
    x = x / max_norm * radius
    x = clip_l2(x, radius)
    return x.astype(np.float64), radius


def save_npz(
    out_path: Path,
    *,
    name: str,
    x: np.ndarray,
    y: Optional[np.ndarray],
    radius: float,
    k: Optional[int],
    source: str,
    note: str,
    raw_shape: Optional[tuple[int, int]] = None,
):
    out_path.parent.mkdir(parents=True, exist_ok=True)
    meta = {
        "name": name,
        "radius": radius,
        "k": k,
        "source": source,
        "note": note,
        "raw_shape": raw_shape,
        "prepared_shape": list(x.shape),
    }
    payload = {"X": x, "metadata_json": json.dumps(meta, sort_keys=True)}
    if y is not None:
        payload["y"] = np.asarray(y)
    np.savez_compressed(out_path, **payload)
    print(f"wrote {out_path} shape={x.shape} k={k} radius={radius}")


def load_openml(name: str, *, version: Optional[int] = None):
    from sklearn.datasets import fetch_openml

    data = fetch_openml(name=name, version=version, as_frame=True, parser="auto")
    x = data.data
    y = data.target
    return x, y


def prepare_letter(args):
    x, y = load_openml("letter", version=1)
    x = as_numeric_frame(x).to_numpy(dtype=float)
    return x, encode_labels(y)


def prepare_shuttle(args):
    x, y = load_openml("shuttle", version=1)
    x = as_numeric_frame(x).to_numpy(dtype=float)
    return x, encode_labels(y)


def prepare_adult(args):
    x, y = load_openml("adult", version=2)
    numeric_cols = ["age", "fnlwgt", "education-num", "capital-gain", "capital-loss", "hours-per-week"]
    present = [c for c in numeric_cols if c in x.columns]
    if len(present) != len(numeric_cols):
        # Fallback: use all numeric columns; do not one-hot categoricals.
        x = as_numeric_frame(x)
    else:
        x = x[present]
    return as_numeric_frame(x).to_numpy(dtype=float), encode_labels(y)


def prepare_iris(args):
    from sklearn.datasets import load_iris

    data = load_iris()
    return data.data.astype(float), data.target.astype(int)


def prepare_covertype(args):
    from sklearn.datasets import fetch_covtype

    data = fetch_covtype()
    return data.data.astype(float), encode_labels(data.target)


def prepare_mnist_pca(args, *, n_components: int):
    from sklearn.datasets import fetch_openml
    from sklearn.decomposition import PCA

    mnist = fetch_openml("mnist_784", version=1, as_frame=False, parser="auto")
    x = np.asarray(mnist.data, dtype=np.float32) / 255.0
    y = encode_labels(mnist.target)
    pca = PCA(n_components=n_components, random_state=args.seed, svd_solver="randomized")
    z = pca.fit_transform(x)
    return z.astype(float), y


def prepare_skin(args):
    url = "https://archive.ics.uci.edu/ml/machine-learning-databases/00229/Skin_NonSkin.txt"
    raw = download_bytes(url, args.raw_dir / "uci" / "Skin_NonSkin.txt", force=args.force)
    df = pd.read_csv(io.BytesIO(raw), sep=r"\s+", header=None)
    x = df.iloc[:, :3].to_numpy(dtype=float)
    y = encode_labels(df.iloc[:, 3])
    return x, y


def prepare_higgs(args):
    url = "https://archive.ics.uci.edu/ml/machine-learning-databases/00280/HIGGS.csv.gz"
    raw_path = args.raw_dir / "uci" / "HIGGS.csv.gz"
    if not raw_path.exists() or args.force:
        download_bytes(url, raw_path, force=True)
    nrows = args.max_rows
    df = pd.read_csv(raw_path, header=None, nrows=nrows)
    y = encode_labels(df.iloc[:, 0])
    x = df.iloc[:, 1:].to_numpy(dtype=float)
    return x, y


def prepare_gas(args, *, drop_year: bool):
    url = "https://archive.ics.uci.edu/static/public/551/gas+turbine+co+and+nox+emission+data+set.zip"
    raw = download_bytes(url, args.raw_dir / "uci" / "gas_turbine.zip", force=args.force)
    frames = []
    with zipfile.ZipFile(io.BytesIO(raw)) as zf:
        for name in sorted(zf.namelist()):
            if name.lower().endswith((".csv", ".xlsx")):
                with zf.open(name) as f:
                    if name.lower().endswith(".csv"):
                        frames.append(pd.read_csv(f))
                    else:
                        frames.append(pd.read_excel(f))
    if not frames:
        raise RuntimeError("no CSV/XLSX files found in gas turbine zip")
    df = pd.concat(frames, ignore_index=True)
    if drop_year:
        for col in list(df.columns):
            if str(col).strip().lower() == "year":
                df = df.drop(columns=[col])
    xdf = as_numeric_frame(df)
    return xdf.to_numpy(dtype=float), None


def prepare_sipu_dataset(args, dataset: str, *, sample: Optional[int] = None):
    data_url = f"{RAW_ROOT}/sipu/{dataset}.data.gz"
    label_url = f"{RAW_ROOT}/sipu/{dataset}.labels0.gz"
    data_raw = download_bytes(data_url, args.raw_dir / "clustering-data-v1" / "sipu" / f"{dataset}.data.gz", force=args.force)
    label_raw = download_bytes(label_url, args.raw_dir / "clustering-data-v1" / "sipu" / f"{dataset}.labels0.gz", force=args.force)
    x = np.loadtxt(io.BytesIO(gzip.decompress(data_raw)), dtype=float)
    y = np.loadtxt(io.BytesIO(gzip.decompress(label_raw)), dtype=int)
    if sample is not None and sample < x.shape[0]:
        rng = np.random.default_rng(args.seed)
        idx = rng.choice(x.shape[0], size=sample, replace=False)
        x = x[idx]
        y = y[idx]
    return x, y


def prepare_g2_surrogate(args, *, d: int):
    # Two-component Gaussian with moderate separation. Same table shape as PE-means.
    rng = np.random.default_rng(args.seed)
    n = 2048
    k = 2
    centers = np.zeros((2, d), dtype=float)
    centers[0, 0] = -0.35
    centers[1, 0] = 0.35
    labels = np.repeat([0, 1], n // 2)
    if labels.size < n:
        labels = np.append(labels, rng.integers(0, 2, size=n - labels.size))
    rng.shuffle(labels)
    x = centers[labels] + rng.normal(scale=0.22, size=(n, d))
    return clip_l2(x, args.radius), labels.astype(int)


def prepare_hst_synthetic(args):
    x, y, _, _ = make_sklearn_blobs_paper_shape(n=5000, d=2, k=25, radius=args.radius, seed=args.seed)
    return x, y


def prepare_generated(name: str, args):
    if name == "google_synthetic":
        x, y, _, _ = make_google_lsh_synthetic(seed=args.seed)
        return x, y
    if name.startswith("g2_"):
        return prepare_g2_surrogate(args, d=int(name.split("_")[1]))
    if name.startswith("scale_"):
        _, k_s, d_s = name.split("_")
        spec = CATALOG[name]
        x, y, _, _ = make_scale_surrogate(n=spec.n or 10000, d=int(d_s), k=int(k_s), radius=args.radius, seed=args.seed)
        return x, y
    if name.startswith("sklearn_"):
        _, k_s, d_s = name.split("_")
        spec = CATALOG[name]
        x, y, _, _ = make_sklearn_blobs_paper_shape(n=spec.n or 20000, d=int(d_s), k=int(k_s), radius=args.radius, seed=args.seed)
        return x, y
    if name == "hst_synthetic":
        return prepare_hst_synthetic(args)
    raise KeyError(name)


def prepare_one(name: str, args) -> Path:
    if name not in CATALOG:
        raise KeyError(f"unknown dataset {name!r}; run --list")
    spec = CATALOG[name]
    if name in {"google_synthetic", "hst_synthetic"} or name.startswith(("g2_", "scale_", "sklearn_")):
        x, y = prepare_generated(name, args)
    elif name == "letter":
        x, y = prepare_letter(args)
    elif name == "shuttle":
        x, y = prepare_shuttle(args)
    elif name == "adult":
        x, y = prepare_adult(args)
    elif name == "iris":
        x, y = prepare_iris(args)
    elif name == "covertype":
        x, y = prepare_covertype(args)
    elif name == "mnist_pca40":
        x, y = prepare_mnist_pca(args, n_components=40)
    elif name == "mnist_pca84":
        x, y = prepare_mnist_pca(args, n_components=84)
    elif name == "skintype":
        x, y = prepare_skin(args)
    elif name == "higgs":
        x, y = prepare_higgs(args)
    elif name == "gas_google":
        x, y = prepare_gas(args, drop_year=True)
    elif name == "gas":
        x, y = prepare_gas(args, drop_year=False)
    elif name == "birch2":
        x, y = prepare_sipu_dataset(args, "birch2", sample=25_000)
    else:
        raise KeyError(name)

    raw_shape = tuple(x.shape)
    if args.max_rows is not None and name != "higgs" and args.max_rows < x.shape[0]:
        rng = np.random.default_rng(args.seed)
        idx = rng.choice(x.shape[0], size=args.max_rows, replace=False)
        x = x[idx]
        y = None if y is None else np.asarray(y)[idx]

    x, radius = center_and_bound(x, radius=args.radius, standardize=not args.no_standardize, center=not args.no_center)
    out = args.out_dir / f"{name}.npz"
    save_npz(out, name=name, x=x, y=y, radius=radius, k=spec.k, source=spec.source, note=spec.note, raw_shape=raw_shape)
    return out


def expand_datasets(names: Iterable[str]) -> list[str]:
    out: list[str] = []
    for name in names:
        if name in SUITES:
            out.extend(SUITES[name])
        elif name in CATALOG:
            out.append(name)
        else:
            raise KeyError(f"unknown dataset or suite {name!r}; run --list")
    # Deduplicate while preserving order.
    seen = set()
    result = []
    for name in out:
        if name not in seen:
            seen.add(name)
            result.append(name)
    return result


def print_catalog():
    print("Suites:")
    for suite, names in SUITES.items():
        print(f"  {suite}: {', '.join(names)}")
    print("\nDatasets:")
    for name in sorted(CATALOG):
        s = CATALOG[name]
        print(f"  {name:18s} suite={s.suite:28s} n={s.n} d={s.d} k={s.k} source={s.source}")
        if s.note:
            print(f"    note: {s.note}")


def make_parser():
    p = argparse.ArgumentParser(description="Download/generate paper benchmark datasets as .npz files.")
    p.add_argument("names", nargs="*", help="dataset names or suites: google_lsh, hst, pe_means, all")
    p.add_argument("--list", action="store_true", help="list known suites and datasets")
    p.add_argument("--out-dir", type=Path, default=Path("data/prepared"))
    p.add_argument("--raw-dir", type=Path, default=Path("data/raw"))
    p.add_argument("--radius", type=float, default=1.0, help="public L2 radius after preprocessing")
    p.add_argument("--seed", type=int, default=20260713)
    p.add_argument("--max-rows", type=int, default=None, help="sample at most this many rows after loading; for HIGGS this is passed to read_csv")
    p.add_argument("--force", action="store_true", help="redownload cached raw files")
    p.add_argument("--no-standardize", action="store_true", help="skip per-column standardization before radius scaling")
    p.add_argument("--no-center", action="store_true", help="skip centering before radius scaling")
    return p


def main(argv: Optional[list[str]] = None):
    args = make_parser().parse_args(argv)
    if args.list:
        print_catalog()
        return
    if not args.names:
        raise SystemExit("pass dataset names/suites or --list")
    names = expand_datasets(args.names)
    for name in names:
        prepare_one(name, args)


if __name__ == "__main__":
    main()
