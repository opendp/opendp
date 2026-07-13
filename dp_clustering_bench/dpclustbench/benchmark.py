from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import List, Optional

import numpy as np
import pandas as pd

from .algorithms.base import FitContext
from .algorithms.registry import build_algorithms
from .common import BenchmarkRow, Bounds, as_2d_float, ensure_dir
from .datasets import load_npz, make_gaussian_mixture
from .metrics import kmeans_loss, kmedians_loss, labels_from_centers, optional_label_metrics


def parse_algorithm_list(s: str) -> List[str]:
    return [part.strip() for part in s.split(",") if part.strip()]


def make_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Benchmark DP clustering algorithms.")
    p.add_argument("--algorithms", default="sklearn,opendp_kmeans,google_lsh,pe_means")
    p.add_argument("--runs", type=int, default=5)
    p.add_argument("--seed", type=int, default=20260713)
    p.add_argument("--out", type=Path, default=Path("results/dp_clustering_results.csv"))
    p.add_argument("--json-out", type=Path, default=None)

    # Synthetic dataset parameters.
    p.add_argument("--dataset", choices=["synthetic", "npz", "prepared"], default="synthetic")
    p.add_argument("--dataset-name", type=str, default=None, help="Name under --data-dir for --dataset prepared, e.g. letter")
    p.add_argument("--data-dir", type=Path, default=Path("data/prepared"))
    p.add_argument("--npz", type=str, default=None)
    p.add_argument("--npz-x-key", type=str, default="X")
    p.add_argument("--npz-y-key", type=str, default="y")
    p.add_argument("--n", type=int, default=5000)
    p.add_argument("--d", type=int, default=8)
    p.add_argument("--k", type=int, default=None, help="number of clusters; for --dataset prepared this defaults to the metadata k if available")
    p.add_argument("--radius", type=float, default=1.0)
    p.add_argument("--cluster-std", type=float, default=0.05)
    p.add_argument("--center-radius", type=float, default=0.75)
    p.add_argument("--imbalance", type=float, default=1.0)

    # Privacy parameters.
    p.add_argument("--epsilon", type=float, default=1.0)
    p.add_argument("--delta", type=float, default=1e-6)

    # OpenDP PR #2664.
    p.add_argument("--opendp-scale", type=float, default=1.0)
    p.add_argument("--opendp-max-depth", type=int, default=7)

    # Google LSH.
    p.add_argument("--google-lsh-max-depth", type=int, default=None)

    # HST.
    p.add_argument("--hst-layers", type=int, default=10)
    p.add_argument("--hst-num-buckets-beam", type=int, default=100)

    # PE-means.
    p.add_argument("--pe-iterations", type=int, default=20)
    p.add_argument("--pe-population", type=int, default=256)
    p.add_argument("--pe-mutation-scale", type=float, default=0.08)
    p.add_argument("--pe-levy-alpha", type=float, default=1.5)
    p.add_argument("--pe-noise-sigma", type=float, default=None)
    p.add_argument("--pe-no-adaptive-population", action="store_true")

    # Nonprivate sklearn.
    p.add_argument("--sklearn-n-init", type=int, default=10)

    return p


def _metadata_k(path: Path):
    try:
        data = np.load(path, allow_pickle=True)
        if "metadata_json" not in data:
            return None
        meta = json.loads(str(data["metadata_json"]))
        return meta.get("k")
    except Exception:
        return None


def load_dataset(args, run_seed: int):
    if args.dataset == "synthetic":
        k = args.k or 8
        return make_gaussian_mixture(
            n=args.n,
            d=args.d,
            k=k,
            radius=args.radius,
            cluster_std=args.cluster_std,
            center_radius=args.center_radius,
            imbalance=args.imbalance,
            seed=run_seed,
        )
    if args.dataset == "npz":
        if not args.npz:
            raise ValueError("--dataset npz requires --npz")
        npz_path = Path(args.npz)
        x, y, centers, bounds = load_npz(npz_path, x_key=args.npz_x_key, y_key=args.npz_y_key)
        x = as_2d_float(x)
        if args.radius is not None:
            bounds = Bounds.symmetric(x.shape[1], args.radius)
        if args.k is None:
            args.k = _metadata_k(npz_path)
        return x, y, centers, bounds
    if args.dataset == "prepared":
        if not args.dataset_name:
            raise ValueError("--dataset prepared requires --dataset-name")
        npz_path = args.data_dir / f"{args.dataset_name}.npz"
        x, y, centers, bounds = load_npz(npz_path, x_key=args.npz_x_key, y_key=args.npz_y_key)
        x = as_2d_float(x)
        if args.k is None:
            args.k = _metadata_k(npz_path)
        if args.k is None:
            raise ValueError(f"prepared dataset {npz_path} has no metadata k; pass --k")
        return x, y, centers, bounds
    raise ValueError(args.dataset)


def evaluate_algorithm(name, algorithm, x, true_labels, ctx, run_idx) -> BenchmarkRow:
    try:
        result = algorithm.fit(x, ctx)
        centers = as_2d_float(result.centers)
        km_loss = kmeans_loss(x, centers)
        kmed_loss = kmedians_loss(x, centers)
        predicted = labels_from_centers(x, centers)
        label_metrics = optional_label_metrics(true_labels, predicted)
        return BenchmarkRow(
            algorithm=name,
            run=run_idx,
            n=x.shape[0],
            d=x.shape[1],
            k=ctx.k,
            epsilon=ctx.epsilon,
            delta=ctx.delta,
            radius=ctx.bounds.radius,
            seed=ctx.seed,
            elapsed_s=result.elapsed_s,
            kmeans_loss_sum=km_loss,
            kmeans_loss_avg=km_loss / x.shape[0],
            kmedians_loss_sum=kmed_loss,
            kmedians_loss_avg=kmed_loss / x.shape[0],
            ari=label_metrics["ari"],
            nmi=label_metrics["nmi"],
            extra=json.dumps(result.extra, sort_keys=True, default=str),
        )
    except Exception as exc:
        return BenchmarkRow(
            algorithm=name,
            run=run_idx,
            n=x.shape[0],
            d=x.shape[1],
            k=ctx.k,
            epsilon=ctx.epsilon,
            delta=ctx.delta,
            radius=ctx.bounds.radius,
            seed=ctx.seed,
            elapsed_s=0.0,
            kmeans_loss_sum=float("nan"),
            kmeans_loss_avg=float("nan"),
            kmedians_loss_sum=float("nan"),
            kmedians_loss_avg=float("nan"),
            status="error",
            error=f"{type(exc).__name__}: {exc}",
        )


def main(argv: Optional[List[str]] = None) -> None:
    args = make_parser().parse_args(argv)
    selected = parse_algorithm_list(args.algorithms)
    registry = build_algorithms(args)
    unknown = sorted(set(selected) - set(registry))
    if unknown:
        raise ValueError(f"unknown algorithms: {unknown}; choices are {sorted(registry)}")

    repo_root = Path(__file__).resolve().parents[1]
    rows = []
    for run_idx in range(args.runs):
        run_seed = args.seed + run_idx
        x, true_labels, _, bounds = load_dataset(args, run_seed)
        ctx = FitContext(
            k=args.k or 8,
            rho=None,
            epsilon=args.epsilon,
            delta=args.delta,
            bounds=bounds,
            seed=run_seed,
            repo_root=str(repo_root),
        )
        for name in selected:
            print(f"run={run_idx} algorithm={name}", flush=True)
            row = evaluate_algorithm(name, registry[name], x, true_labels, ctx, run_idx)
            rows.append(row.to_dict())
            if row.status != "ok":
                print(f"  ERROR {row.error}", flush=True)
            else:
                print(
                    f"  loss_avg={row.kmeans_loss_avg:.6g} "
                    f"kmed_avg={row.kmedians_loss_avg:.6g} elapsed={row.elapsed_s:.3f}s",
                    flush=True,
                )

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
        summary = ok.groupby("algorithm").agg(
            kmeans_loss_avg_mean=("kmeans_loss_avg", "mean"),
            kmeans_loss_avg_p25=("kmeans_loss_avg", lambda s: float(np.quantile(s, 0.25))),
            kmeans_loss_avg_p75=("kmeans_loss_avg", lambda s: float(np.quantile(s, 0.75))),
            elapsed_s_mean=("elapsed_s", "mean"),
        )
        print("\nSummary:")
        print(summary.sort_values("kmeans_loss_avg_mean"))


if __name__ == "__main__":
    main()
