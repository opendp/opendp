from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable, Optional

import numpy as np
from scipy import sparse


@dataclass(frozen=True)
class FeatureFamily:
    name: str
    n_tags: int
    row_weight: float
    core_tags_per_cluster: int
    core_hit_prob: float


DEFAULT_FAMILIES: tuple[FeatureFamily, ...] = (
    FeatureFamily("commerce", 24_000, 0.32, 30, 0.22),
    FeatureFamily("psychographics", 14_000, 0.18, 22, 0.18),
    FeatureFamily("demographics", 7_000, 0.08, 8, 0.32),
    FeatureFamily("geography", 5_000, 0.06, 7, 0.26),
    FeatureFamily("automotive", 6_000, 0.08, 12, 0.20),
    FeatureFamily("media", 5_000, 0.08, 12, 0.18),
    FeatureFamily("household", 4_000, 0.07, 8, 0.24),
    FeatureFamily("finance", 4_000, 0.06, 8, 0.16),
    FeatureFamily("retail_visitation", 6_688, 0.07, 14, 0.20),
)


def _family_offsets(families: Iterable[FeatureFamily]) -> tuple[np.ndarray, np.ndarray, list[str]]:
    families = tuple(families)
    starts = np.cumsum([0] + [f.n_tags for f in families[:-1]])
    stops = starts + np.array([f.n_tags for f in families], dtype=int)
    return starts.astype(int), stops.astype(int), [f.name for f in families]


def _zipf_probs(k: int, skew: float) -> np.ndarray:
    if skew <= 0:
        return np.full(k, 1.0 / k)
    w = 1.0 / np.arange(1, k + 1, dtype=float) ** skew
    return w / w.sum()


def _sample_without_replacement_weighted(
    rng: np.random.Generator,
    population: np.ndarray,
    weights: np.ndarray,
    size: int,
) -> np.ndarray:
    if size <= 0:
        return np.empty(0, dtype=int)
    size = min(size, population.size)
    p = weights / weights.sum()
    return rng.choice(population, size=size, replace=False, p=p).astype(int)


def build_family_tag_arrays(families: tuple[FeatureFamily, ...]) -> tuple[np.ndarray, list[dict]]:
    tag_family = np.empty(sum(f.n_tags for f in families), dtype=np.int16)
    starts, stops, _ = _family_offsets(families)
    meta = []
    for fid, (f, start, stop) in enumerate(zip(families, starts, stops)):
        tag_family[start:stop] = fid
        meta.append({"family_id": fid, "name": f.name, "start": int(start), "stop": int(stop), **asdict(f)})
    return tag_family, meta


def make_idgraph_tag_mixture(
    *,
    n: int = 50_000,
    k: int = 16,
    families: tuple[FeatureFamily, ...] = DEFAULT_FAMILIES,
    mean_active_tags: int = 64,
    active_tag_dispersion: float = 0.35,
    audience_skew: float = 0.55,
    shared_core_fraction: float = 0.15,
    background_hot_tags: int = 3_000,
    background_hot_multiplier: float = 8.0,
    seed: int = 0,
) -> tuple[sparse.csr_matrix, np.ndarray, dict]:
    if n <= 0 or k <= 0:
        raise ValueError("n and k must be positive")
    if mean_active_tags <= 0:
        raise ValueError("mean_active_tags must be positive")

    rng = np.random.default_rng(seed)
    d = sum(f.n_tags for f in families)
    starts, stops, family_names = _family_offsets(families)
    tag_family, family_meta = build_family_tag_arrays(families)

    family_weights = np.array([f.row_weight for f in families], dtype=float)
    family_weights = family_weights / family_weights.sum()

    tag_weights = np.ones(d, dtype=float)
    hot = rng.choice(d, size=min(background_hot_tags, d), replace=False)
    tag_weights[hot] *= background_hot_multiplier

    cluster_cores: list[np.ndarray] = []
    shared_cores: list[np.ndarray] = []
    for f, start, stop in zip(families, starts, stops):
        fam_tags = np.arange(start, stop, dtype=int)
        n_shared = int(round(shared_core_fraction * f.core_tags_per_cluster * k))
        shared_cores.append(rng.choice(fam_tags, size=min(n_shared, fam_tags.size), replace=False))

    for c in range(k):
        parts = []
        for fid, (f, start, stop) in enumerate(zip(families, starts, stops)):
            fam_tags = np.arange(start, stop, dtype=int)
            shared = shared_cores[fid]
            n_shared_pick = min(shared.size, max(1, int(shared_core_fraction * f.core_tags_per_cluster)))
            if n_shared_pick > 0:
                parts.append(rng.choice(shared, size=n_shared_pick, replace=False))
            n_unique = max(1, f.core_tags_per_cluster - n_shared_pick)
            parts.append(rng.choice(fam_tags, size=n_unique, replace=False))
        cluster_cores.append(np.unique(np.concatenate(parts)).astype(int))

    cluster_probs = _zipf_probs(k, audience_skew)
    labels = rng.choice(k, size=n, p=cluster_probs).astype(np.int32)

    rows: list[np.ndarray] = []
    cols: list[np.ndarray] = []
    row_active_counts = np.empty(n, dtype=np.int16)

    family_ids = np.arange(len(families))
    for i, c in enumerate(labels):
        shape = 1.0 / max(active_tag_dispersion, 1e-9) ** 2
        scale = mean_active_tags / shape
        active_budget = int(max(1, rng.poisson(rng.gamma(shape, scale))))
        fam_counts = rng.multinomial(active_budget, family_weights)

        selected_parts = []
        for fid, count in zip(family_ids, fam_counts):
            if count == 0:
                continue
            start, stop = starts[fid], stops[fid]
            pop = np.arange(start, stop, dtype=int)
            selected_parts.append(
                _sample_without_replacement_weighted(rng, pop, tag_weights[start:stop], int(count))
            )

        core = cluster_cores[int(c)]
        if core.size:
            probs = np.array([families[int(tag_family[j])].core_hit_prob for j in core], dtype=float)
            hit = rng.random(core.size) < probs
            selected_parts.append(core[hit])

        if selected_parts:
            selected = np.unique(np.concatenate(selected_parts)).astype(np.int32)
        else:
            selected = np.empty(0, dtype=np.int32)

        rows.append(np.full(selected.size, i, dtype=np.int32))
        cols.append(selected)
        row_active_counts[i] = min(selected.size, np.iinfo(np.int16).max)

    row_idx = np.concatenate(rows) if rows else np.empty(0, dtype=np.int32)
    col_idx = np.concatenate(cols) if cols else np.empty(0, dtype=np.int32)
    data = np.ones(row_idx.size, dtype=np.float32)
    x = sparse.csr_matrix((data, (row_idx, col_idx)), shape=(n, d), dtype=np.float32)
    x.sum_duplicates()
    x.data[:] = 1.0

    metadata = {
        "generator": "idgraph_tag_mixture",
        "n": int(n),
        "d": int(d),
        "k": int(k),
        "mean_active_tags": int(mean_active_tags),
        "actual_mean_active_tags": float(row_active_counts.mean()),
        "actual_max_active_tags": int(row_active_counts.max()),
        "audience_skew": float(audience_skew),
        "feature_families": family_meta,
        "family_names": family_names,
        "cluster_probs": cluster_probs.tolist(),
        "cluster_core_tags": [core.tolist() for core in cluster_cores],
        "tag_family_dtype": str(tag_family.dtype),
        "neighboring": "add_remove_user_rows",
        "privacy_unit": "consumer_row",
    }
    metadata["tag_family"] = tag_family.tolist()
    return x, labels, metadata


def save_sparse_dataset(path: str | Path, x: sparse.csr_matrix, labels: np.ndarray, metadata: dict) -> Path:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if not sparse.isspmatrix_csr(x):
        x = x.tocsr()
    np.savez_compressed(
        path,
        data=x.data,
        indices=x.indices,
        indptr=x.indptr,
        shape=np.array(x.shape, dtype=np.int64),
        y=np.asarray(labels),
        metadata_json=json.dumps(metadata, sort_keys=True),
    )
    return path


def load_sparse_dataset(path: str | Path) -> tuple[sparse.csr_matrix, Optional[np.ndarray], dict]:
    z = np.load(path, allow_pickle=True)
    x = sparse.csr_matrix((z["data"], z["indices"], z["indptr"]), shape=tuple(z["shape"]), dtype=np.float32)
    labels = np.asarray(z["y"]) if "y" in z else None
    meta = json.loads(str(z["metadata_json"])) if "metadata_json" in z else {}
    return x, labels, meta


def make_feature_hash_view(
    x: sparse.csr_matrix,
    *,
    n_features: int = 256,
    radius: float = 1.0,
    seed: int = 0,
) -> np.ndarray:
    if n_features <= 0:
        raise ValueError("n_features must be positive")
    x = x.tocsr()
    rng = np.random.default_rng(seed)
    buckets = rng.integers(0, n_features, size=x.shape[1], dtype=np.int64)
    signs = rng.choice(np.array([-1.0, 1.0], dtype=np.float32), size=x.shape[1])
    rows = np.repeat(np.arange(x.shape[0], dtype=np.int64), np.diff(x.indptr))
    cols = buckets[x.indices]
    data = signs[x.indices] * x.data
    hashed = sparse.csr_matrix((data, (rows, cols)), shape=(x.shape[0], n_features), dtype=np.float32)
    dense = hashed.toarray().astype(np.float32)
    norms = np.linalg.norm(dense, axis=1, keepdims=True)
    dense = dense * np.minimum(1.0, radius / np.maximum(norms, 1e-12))
    return dense


def _make_sparse_rows_from_sets(rows: list[np.ndarray], d: int) -> sparse.csr_matrix:
    if not rows:
        return sparse.csr_matrix((0, d), dtype=np.float32)
    indptr = np.zeros(len(rows) + 1, dtype=np.int64)
    sizes = np.array([len(r) for r in rows], dtype=np.int64)
    indptr[1:] = np.cumsum(sizes)
    indices = np.concatenate([np.asarray(r, dtype=np.int32) for r in rows]) if sizes.sum() else np.empty(0, dtype=np.int32)
    data = np.ones(indices.size, dtype=np.float32)
    mat = sparse.csr_matrix((data, indices, indptr), shape=(len(rows), d), dtype=np.float32)
    mat.sum_duplicates()
    mat.data[:] = 1.0
    return mat


def public_seed_prototypes_from_metadata(
    metadata: dict,
    *,
    variants_per_cluster: int = 4,
    recall: float = 0.45,
    noise_tags: int = 40,
    min_active_tags: int = 32,
    max_active_tags: int = 160,
    seed: int = 0,
) -> sparse.csr_matrix:
    rng = np.random.default_rng(seed)
    d = int(metadata.get("d"))
    cores = [np.array(c, dtype=np.int32) for c in metadata.get("cluster_core_tags", [])]
    rows: list[np.ndarray] = []
    for core in cores:
        for _ in range(variants_per_cluster):
            if core.size:
                keep = core[rng.random(core.size) < recall]
            else:
                keep = np.empty(0, dtype=np.int32)
            add_n = max(0, int(rng.poisson(noise_tags)))
            noise = rng.choice(d, size=min(add_n, d), replace=False).astype(np.int32)
            tags = np.unique(np.concatenate([keep, noise]))
            if tags.size < min_active_tags:
                fill = rng.choice(d, size=min(min_active_tags - tags.size, d), replace=False).astype(np.int32)
                tags = np.unique(np.concatenate([tags, fill]))
            if tags.size > max_active_tags:
                tags = rng.choice(tags, size=max_active_tags, replace=False).astype(np.int32)
            rows.append(np.sort(tags))
    return _make_sparse_rows_from_sets(rows, d)


def main(argv: Optional[list[str]] = None) -> None:
    import argparse

    p = argparse.ArgumentParser(description="Prepare representative sparse ID-Graph-style synthetic tags.")
    p.add_argument("--out", type=Path, default=Path("data/prepared/idgraph_tags_sparse.npz"))
    p.add_argument("--dense-out", type=Path, default=Path("data/prepared/idgraph_tags_hashed256.npz"))
    p.add_argument("--n", type=int, default=50_000)
    p.add_argument("--k", type=int, default=16)
    p.add_argument("--mean-active-tags", type=int, default=64)
    p.add_argument("--hash-features", type=int, default=256)
    p.add_argument("--radius", type=float, default=1.0)
    p.add_argument("--seed", type=int, default=20260713)
    p.add_argument("--no-dense", action="store_true")
    args = p.parse_args(argv)

    x, y, meta = make_idgraph_tag_mixture(
        n=args.n,
        k=args.k,
        mean_active_tags=args.mean_active_tags,
        seed=args.seed,
    )
    save_sparse_dataset(args.out, x, y, meta)
    print(f"wrote sparse {args.out} shape={x.shape} nnz={x.nnz} mean_active={x.nnz / x.shape[0]:.2f}")

    if not args.no_dense:
        dense = make_feature_hash_view(x, n_features=args.hash_features, radius=args.radius, seed=args.seed)
        dense_meta = {
            "generator": "idgraph_tag_mixture_feature_hash_view",
            "source": str(args.out),
            "k": int(args.k),
            "radius": float(args.radius),
            "n_features": int(args.hash_features),
        }
        args.dense_out.parent.mkdir(parents=True, exist_ok=True)
        np.savez_compressed(args.dense_out, X=dense, y=y, metadata_json=json.dumps(dense_meta, sort_keys=True))
        print(f"wrote dense hashed view {args.dense_out} shape={dense.shape}")


if __name__ == "__main__":
    main()

