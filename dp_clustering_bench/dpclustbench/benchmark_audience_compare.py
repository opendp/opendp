from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Optional

import numpy as np
import pandas as pd
from scipy import sparse

from .algorithms.base import FitContext
from .algorithms.google_lsh import GoogleLSHCoresetKMeans
from .algorithms.hst_google import GoogleHSTClustering
from .algorithms.opendp_pe import OpenDPSparsePEMeans, OpenDPSparsePEMeansConfig
from .algorithms.dp_spherical_lloyd import DPSphericalLloyd, DPSphericalLloydConfig
from .algorithms.opendp_pr2664 import OpenDPKMeans, OpenDPKMedians
from .algorithms.pe_means import PEMeans, PEMeansConfig
from .algorithms.base import SklearnKMeans, SklearnKMedians
from .audience_synthetic import (
    make_feature_hash_view,
    make_idgraph_tag_mixture,
    public_seed_prototypes_from_metadata,
)
from .common import Bounds, ensure_dir
from .metrics import kmeans_loss, kmedians_loss, labels_from_centers, optional_label_metrics


def _parse_algorithms(value: str) -> list[str]:
    return [part.strip() for part in value.split(",") if part.strip()]


def _parse_floats(value: str) -> list[float]:
    return [float(part.strip()) for part in value.split(",") if part.strip()]


def _epsilon_from_rho(rho: float, delta: float) -> float:
    if rho <= 0:
        raise ValueError("rho must be positive")
    return float(rho + 2.0 * np.sqrt(rho * np.log(1.0 / delta)))


def _ensure_csr_binary(x) -> sparse.csr_matrix:
    if sparse.issparse(x):
        out = x.tocsr().astype(np.float32)
    else:
        out = sparse.csr_matrix(np.asarray(x) != 0, dtype=np.float32)
    out.sum_duplicates()
    out.data[:] = 1.0
    return out


def _row_nnz(x: sparse.csr_matrix) -> np.ndarray:
    return np.diff(x.indptr).astype(np.float64)


def sparse_hamming_loss(x: sparse.csr_matrix, centers: sparse.csr_matrix, *, batch_size: int = 8192) -> float:
    x = _ensure_csr_binary(x)
    centers = _ensure_csr_binary(centers)
    labels = np.empty(x.shape[0], dtype=np.int32)
    x_sizes = _row_nnz(x)
    c_sizes = _row_nnz(centers)
    for start in range(0, x.shape[0], batch_size):
        stop = min(start + batch_size, x.shape[0])
        inter = x[start:stop].dot(centers.T)
        inter = inter.toarray() if sparse.issparse(inter) else np.asarray(inter)
        dist = x_sizes[start:stop, None] + c_sizes[None, :] - 2.0 * inter
        labels[start:stop] = np.argmin(dist, axis=1)
    loss = 0.0
    for start in range(0, x.shape[0], batch_size):
        stop = min(start + batch_size, x.shape[0])
        assigned = centers[labels[start:stop]]
        inter = x[start:stop].multiply(assigned).sum(axis=1)
        inter = np.asarray(inter).ravel()
        loss += float(np.sum(x_sizes[start:stop] + c_sizes[labels[start:stop]] - 2.0 * inter))
    return loss


def cluster_core_overlap(centers: sparse.csr_matrix, metadata: dict) -> dict[str, float]:
    core_sets = [set(map(int, c)) for c in metadata.get("cluster_core_tags", [])]
    if not core_sets:
        return {
            "mean_best_core_jaccard": float("nan"),
            "mean_best_core_precision": float("nan"),
            "mean_best_core_recall": float("nan"),
        }
    jaccards = []
    precisions = []
    recalls = []
    for j in range(centers.shape[0]):
        pred = set(map(int, centers[j].indices))
        best = (0.0, 0.0, 0.0)
        for core in core_sets:
            inter = len(pred & core)
            union = len(pred | core)
            jac = inter / union if union else 0.0
            prec = inter / len(pred) if pred else 0.0
            rec = inter / len(core) if core else 0.0
            if jac > best[0]:
                best = (jac, prec, rec)
        jaccards.append(best[0])
        precisions.append(best[1])
        recalls.append(best[2])
    return {
        "mean_best_core_jaccard": float(np.mean(jaccards)),
        "mean_best_core_precision": float(np.mean(precisions)),
        "mean_best_core_recall": float(np.mean(recalls)),
    }


def _feature_prior_from_metadata(metadata: dict, d: int) -> np.ndarray:
    prior = np.ones(d, dtype=np.float64)
    families = metadata.get("feature_families", [])
    if not families:
        return prior
    prior[:] = 0.0
    for fam in families:
        start = int(fam["start"])
        stop = int(fam["stop"])
        width = max(1, stop - start)
        mass = float(fam.get("row_weight", 1.0))
        prior[start:stop] = mass / width
    if not np.isfinite(prior).all() or prior.sum() <= 0:
        prior[:] = 1.0
    return prior


def make_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Compare non-private, PE, and OpenDP-PE clustering on audience tags.")
    p.add_argument("--algorithms", default="sklearn,sklearn_kmedians_like,opendp_kmeans,opendp_kmedians,pe_means,opendp_pe_means")
    p.add_argument("--runs", type=int, default=3)
    p.add_argument("--seed", type=int, default=20260713)
    p.add_argument("--out", type=Path, default=Path("results/audience_compare.csv"))
    p.add_argument("--json-out", type=Path, default=None)
    p.add_argument("--rho-values", type=str, default="0.05,0.1,0.25,0.5,1.0")
    p.add_argument(
        "--dense-view",
        choices=["hash", "raw-binary"],
        default="hash",
        help="Dense view used for non-sparse algorithms. 'raw-binary' uses the original 0/1 tags.",
    )

    p.add_argument("--n", type=int, default=50_000)
    p.add_argument("--k", type=int, default=16)
    p.add_argument("--mean-active-tags", type=int, default=64)
    p.add_argument("--hash-dim", type=int, default=256)
    p.add_argument("--radius", type=float, default=1.0)

    p.add_argument("--delta", type=float, default=1e-6)

    p.add_argument("--sklearn-n-init", type=int, default=10)
    p.add_argument("--opendp-scale", type=float, default=1.0)
    p.add_argument("--opendp-max-depth", type=int, default=7)
    p.add_argument("--google-lsh-max-depth", type=int, default=None)
    p.add_argument("--hst-layers", type=int, default=10)
    p.add_argument("--hst-num-buckets-beam", type=int, default=100)

    p.add_argument("--pe-iterations", type=int, default=20)
    p.add_argument("--pe-population", type=int, default=256)
    p.add_argument("--pe-mutation-scale", type=float, default=0.08)
    p.add_argument("--pe-levy-alpha", type=float, default=1.5)
    p.add_argument("--pe-noise-sigma", type=float, default=None)
    p.add_argument("--pe-no-adaptive-population", action="store_true")

    p.add_argument("--opendp-pe-iterations", type=int, default=16)
    p.add_argument("--opendp-pe-population", type=int, default=512)
    p.add_argument("--opendp-pe-center-active-tags", type=int, default=96)
    p.add_argument("--opendp-pe-min-active-tags", type=int, default=16)
    p.add_argument("--opendp-pe-max-active-tags", type=int, default=160)
    p.add_argument("--opendp-pe-mutation-drop-prob", type=float, default=0.18)
    p.add_argument("--opendp-pe-mutation-add-mean", type=float, default=18.0)
    p.add_argument("--opendp-pe-distance", choices=["weighted_jaccard", "weighted_hamming", "weighted_cosine", "jaccard", "hamming"], default="weighted_jaccard")
    p.add_argument("--opendp-pe-batch-size", type=int, default=8192)
    p.add_argument("--opendp-pe-backend", choices=["auto", "opendp", "numpy"], default="auto")
    p.add_argument("--opendp-pe-neighboring", choices=["add_remove", "replace_one"], default="add_remove")
    p.add_argument("--opendp-pe-noise-sigma", type=float, default=None)
    p.add_argument("--opendp-pe-noisy-candidate-weight-threshold-multiplier", type=float, default=1.0)
    p.add_argument("--opendp-pe-use-public-seeds", action="store_true")
    p.add_argument("--opendp-pe-no-public-structure", action="store_true")
    p.add_argument("--opendp-pe-seed-prototypes-per-cluster", type=int, default=4)
    p.add_argument("--opendp-pe-init-from-data-sample", action="store_true")

    # DP spherical Lloyd (new).
    p.add_argument("--dsl-iterations", type=int, default=5)
    p.add_argument("--dsl-center-active-tags", type=int, default=96)
    p.add_argument("--dsl-clip-active-tags", type=int, default=128)
    p.add_argument("--dsl-init-active-tags", type=int, default=96)
    p.add_argument("--dsl-batch-size", type=int, default=8192)
    p.add_argument("--dsl-noise-sigma", type=float, default=None)

    return p


def _dense_spec(args, rho: float):
    pe_cfg = PEMeansConfig(
        iterations=args.pe_iterations,
        population_size=args.pe_population,
        mutation_scale=args.pe_mutation_scale,
        levy_alpha=args.pe_levy_alpha,
        noise_sigma=args.pe_noise_sigma,
        rho=rho,
        adaptive_population=not args.pe_no_adaptive_population,
    )
    return {
        "sklearn": lambda: SklearnKMeans(n_init=args.sklearn_n_init),
        "sklearn_kmedians_like": lambda: SklearnKMedians(),
        "google_lsh": lambda: GoogleLSHCoresetKMeans(max_depth=args.google_lsh_max_depth),
        "hst": lambda: GoogleHSTClustering(layers=args.hst_layers, num_buckets_beam=args.hst_num_buckets_beam),
        "opendp_kmeans": lambda: OpenDPKMeans(scale=args.opendp_scale, max_depth=args.opendp_max_depth),
        "opendp_kmedians": lambda: OpenDPKMedians(scale=args.opendp_scale, max_depth=args.opendp_max_depth),
        "pe_means": lambda: PEMeans(pe_cfg),
    }


def _sparse_spec(args, metadata: dict, seed: int, d: int, rho: float):
    if args.opendp_pe_no_public_structure:
        feature_groups = None
        feature_prior = None
    else:
        feature_groups = np.asarray(metadata.get("tag_family"), dtype=np.int32) if metadata.get("tag_family") is not None else None
        feature_prior = _feature_prior_from_metadata(metadata, d)
    public_seeds = None
    if args.opendp_pe_use_public_seeds and not args.opendp_pe_no_public_structure:
        public_seeds = public_seed_prototypes_from_metadata(
            metadata,
            variants_per_cluster=args.opendp_pe_seed_prototypes_per_cluster,
            min_active_tags=args.opendp_pe_min_active_tags,
            max_active_tags=args.opendp_pe_max_active_tags,
            seed=seed + 12345,
        )
    pe_cfg = OpenDPSparsePEMeansConfig(
        iterations=args.opendp_pe_iterations,
        population_size=args.opendp_pe_population,
        center_active_tags=args.opendp_pe_center_active_tags,
        min_active_tags=args.opendp_pe_min_active_tags,
        max_active_tags=args.opendp_pe_max_active_tags,
        mutation_drop_prob=args.opendp_pe_mutation_drop_prob,
        mutation_add_mean=args.opendp_pe_mutation_add_mean,
        distance=args.opendp_pe_distance,
        batch_size=args.opendp_pe_batch_size,
        backend=args.opendp_pe_backend,
        neighboring=args.opendp_pe_neighboring,
        noise_sigma=args.opendp_pe_noise_sigma,
        noisy_candidate_weight_threshold_multiplier=args.opendp_pe_noisy_candidate_weight_threshold_multiplier,
        feature_prior=feature_prior,
        feature_groups=feature_groups,
        public_seed_candidates=public_seeds,
        init_from_data_sample=args.opendp_pe_init_from_data_sample,
    )
    dsl_cfg = DPSphericalLloydConfig(
        iterations=args.dsl_iterations,
        center_active_tags=args.dsl_center_active_tags,
        clip_active_tags=args.dsl_clip_active_tags,
        init_active_tags=args.dsl_init_active_tags,
        batch_size=args.dsl_batch_size,
        noise_sigma=args.dsl_noise_sigma,
    )
    return {
        "opendp_pe_means": lambda: OpenDPSparsePEMeans(rho=rho, delta=args.delta, random_state=seed, config=pe_cfg),
        "dp_spherical_lloyd": lambda: DPSphericalLloyd(rho=rho, delta=args.delta, random_state=seed, config=dsl_cfg),
    }


def _evaluate_dense(name, algorithm, x_dense, true_labels, ctx):
    result = algorithm.fit(x_dense, ctx)
    centers = np.asarray(result.centers, dtype=float)
    objective_sum = kmeans_loss(x_dense, centers)
    objective_kind = "kmeans"
    predicted = labels_from_centers(x_dense, centers)
    label_metrics = optional_label_metrics(true_labels, predicted)
    row = {
        "algorithm": name,
        "view": "dense",
        "objective_kind": objective_kind,
        "objective_sum": objective_sum,
        "objective_avg": objective_sum / x_dense.shape[0],
        "elapsed_s": result.elapsed_s,
        "ari": label_metrics["ari"],
        "nmi": label_metrics["nmi"],
        "rho": ctx.rho,
        "centers_nnz_mean": float("nan"),
        "core_jaccard": float("nan"),
        "extra": json.dumps(result.extra, sort_keys=True, default=str),
        "status": "ok",
        "error": "",
    }
    return row


def _evaluate_sparse(name, algorithm, x_sparse, true_labels, ctx, metadata):
    result = algorithm.fit(x_sparse, ctx)
    centers = result.centers.tocsr()
    objective_sum = sparse_hamming_loss(x_sparse, centers, batch_size=8192)
    predicted = algorithm.labels_
    if predicted is None:
        predicted = np.asarray(result.extra.get("labels")) if "labels" in result.extra else None
    if predicted is None:
        from opendp.extras.sklearn.cluster import nearest_sparse_labels

        predicted = nearest_sparse_labels(x_sparse, centers, batch_size=8192)
    label_metrics = optional_label_metrics(true_labels, predicted)
    overlap = cluster_core_overlap(centers, metadata)
    row = {
        "algorithm": name,
        "view": "sparse",
        "objective_kind": "hamming",
        "objective_sum": objective_sum,
        "objective_avg": objective_sum / x_sparse.shape[0],
        "elapsed_s": result.elapsed_s,
        "ari": label_metrics["ari"],
        "nmi": label_metrics["nmi"],
        "rho": ctx.rho,
        "centers_nnz_mean": float(np.mean(np.diff(centers.indptr))),
        "core_jaccard": overlap["mean_best_core_jaccard"],
        "extra": json.dumps({**result.extra, **overlap}, sort_keys=True, default=str),
        "status": "ok",
        "error": "",
    }
    return row


def main(argv: Optional[list[str]] = None) -> None:
    args = make_parser().parse_args(argv)
    selected = _parse_algorithms(args.algorithms)
    rho_values = _parse_floats(args.rho_values)
    rows = []

    for run in range(args.runs):
        seed = args.seed + run
        x_sparse, y, meta = make_idgraph_tag_mixture(
            n=args.n,
            k=args.k,
            mean_active_tags=args.mean_active_tags,
            seed=seed,
        )
        if args.dense_view == "hash":
            x_dense = make_feature_hash_view(
                x_sparse,
                n_features=args.hash_dim,
                radius=args.radius,
                seed=seed,
            )
        else:
            x_dense = x_sparse.toarray().astype(np.float64, copy=False)
            norms = np.linalg.norm(x_dense, axis=1, keepdims=True)
            x_dense = x_dense / np.maximum(norms, 1.0)
            x_dense *= args.radius
        for rho in rho_values:
            eps_equiv = _epsilon_from_rho(rho, args.delta)
            dense_algorithms = _dense_spec(args, rho)
            dense_ctx = FitContext(
                k=args.k,
                rho=rho,
                epsilon=eps_equiv,
                delta=args.delta,
                bounds=Bounds.symmetric(x_dense.shape[1], args.radius),
                seed=seed,
                repo_root=None,
            )
            sparse_ctx = FitContext(
                k=args.k,
                rho=rho,
                epsilon=eps_equiv,
                delta=args.delta,
                bounds=Bounds.symmetric(x_sparse.shape[1], 1.0),
                seed=seed,
                repo_root=None,
            )
            sparse_algorithms = _sparse_spec(args, meta, seed, x_sparse.shape[1], rho)

            for name in selected:
                print(f"run={run} rho={rho} algorithm={name}", flush=True)
                try:
                    if name in dense_algorithms:
                        row = _evaluate_dense(name, dense_algorithms[name](), x_dense, y, dense_ctx)
                    elif name in sparse_algorithms:
                        row = _evaluate_sparse(name, sparse_algorithms[name](), x_sparse, y, sparse_ctx, meta)
                    else:
                        raise ValueError(
                            f"unknown algorithm: {name}; choices are {sorted(set(dense_algorithms) | set(sparse_algorithms))}"
                        )
                    row["run"] = run
                    row["n"] = int(x_sparse.shape[0])
                    row["d"] = int(x_sparse.shape[1])
                    row["k"] = int(args.k)
                    row["epsilon"] = float(eps_equiv)
                    row["delta"] = float(args.delta)
                    row["seed"] = int(seed)
                    rows.append(row)
                    if name == "opendp_pe_means":
                        extra = json.loads(row["extra"])
                        print(
                            f"  rho={rho} view={row['view']} objective_avg={row['objective_avg']:.6g} "
                            f"ari={row['ari']} nmi={row['nmi']} elapsed={row['elapsed_s']:.3f}s "
                            f"rho_total={extra.get('rho_total')} rho_step={extra.get('rho_step')} "
                            f"T={extra.get('iterations')} scale={extra.get('vote_noise_scale')} "
                            f"tau={extra.get('candidate_weight_threshold')}",
                            flush=True,
                        )
                    else:
                        print(
                            f"  rho={rho} view={row['view']} objective_avg={row['objective_avg']:.6g} "
                            f"ari={row['ari']} nmi={row['nmi']} elapsed={row['elapsed_s']:.3f}s",
                            flush=True,
                        )
                except Exception as exc:
                    rows.append(
                        {
                            "algorithm": name,
                            "run": run,
                            "n": int(x_sparse.shape[0]),
                            "d": int(x_sparse.shape[1]),
                            "k": int(args.k),
                            "rho": float(rho),
                            "epsilon": float(eps_equiv),
                            "delta": float(args.delta),
                            "seed": int(seed),
                            "view": "error",
                            "objective_kind": "",
                            "objective_sum": float("nan"),
                            "objective_avg": float("nan"),
                            "elapsed_s": 0.0,
                            "ari": None,
                            "nmi": None,
                            "centers_nnz_mean": float("nan"),
                            "core_jaccard": float("nan"),
                            "extra": "{}",
                            "status": "error",
                            "error": f"{type(exc).__name__}: {exc}",
                        }
                    )
                    print(f"  ERROR {type(exc).__name__}: {exc}", flush=True)

    df = pd.DataFrame(rows)
    ensure_dir(args.out.parent)
    df.to_csv(args.out, index=False)
    print(f"wrote {args.out}")

    if args.json_out is not None:
        ensure_dir(args.json_out.parent)
        args.json_out.write_text(json.dumps(rows, indent=2), encoding="utf-8")
        print(f"wrote {args.json_out}")

    ok = df[df["status"] == "ok"]
    if not ok.empty:
        summary = ok.groupby(["algorithm", "rho", "view"]).agg(
            objective_avg_mean=("objective_avg", "mean"),
            ari_mean=("ari", "mean"),
            nmi_mean=("nmi", "mean"),
            elapsed_s_mean=("elapsed_s", "mean"),
        )
        print("\nSummary:")
        print(summary.sort_values("objective_avg_mean"))


if __name__ == "__main__":
    main()
