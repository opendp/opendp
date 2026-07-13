from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Optional

import numpy as np
import pandas as pd

from .algorithms.base import FitContext, SklearnKMeans, SklearnKMedians
from .audience_synthetic import (
    make_feature_hash_view,
    make_idgraph_tag_mixture,
)
from .common import Bounds, BenchmarkRow, ensure_dir
from .metrics import kmeans_loss, kmedians_loss, labels_from_centers, optional_label_metrics


def _parse_algorithms(value: str) -> list[str]:
    return [part.strip() for part in value.split(",") if part.strip()]


def make_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Benchmark non-DP baselines on sparse audience tags.")
    p.add_argument("--algorithms", default="sklearn,sklearn_kmedians_like")
    p.add_argument("--runs", type=int, default=3)
    p.add_argument("--seed", type=int, default=20260713)
    p.add_argument("--out", type=Path, default=Path("results/audience_baseline.csv"))
    p.add_argument("--json-out", type=Path, default=None)

    p.add_argument("--n", type=int, default=50_000)
    p.add_argument("--k", type=int, default=16)
    p.add_argument("--mean-active-tags", type=int, default=64)
    p.add_argument("--hash-dim", type=int, default=256)
    p.add_argument("--radius", type=float, default=1.0)
    p.add_argument("--seed-prototypes-per-cluster", type=int, default=4)

    p.add_argument("--sklearn-n-init", type=int, default=10)
    return p


def _run_one(name: str, x_dense: np.ndarray, true_labels: np.ndarray, ctx: FitContext, args) -> BenchmarkRow:
    if name == "sklearn":
        alg = SklearnKMeans(n_init=args.sklearn_n_init)
    elif name == "sklearn_kmedians_like":
        alg = SklearnKMedians()
    else:
        raise ValueError(f"unknown algorithm: {name}")

    result = alg.fit(x_dense, ctx)
    centers = np.asarray(result.centers, dtype=float)
    km_loss = kmeans_loss(x_dense, centers)
    kmed_loss = kmedians_loss(x_dense, centers)
    predicted = labels_from_centers(x_dense, centers)
    label_metrics = optional_label_metrics(true_labels, predicted)
    return BenchmarkRow(
        algorithm=name,
        run=0,
        n=x_dense.shape[0],
        d=x_dense.shape[1],
        k=ctx.k,
        epsilon=ctx.epsilon,
        delta=ctx.delta,
        radius=ctx.bounds.radius,
        seed=ctx.seed,
        elapsed_s=result.elapsed_s,
        kmeans_loss_sum=km_loss,
        kmeans_loss_avg=km_loss / x_dense.shape[0],
        kmedians_loss_sum=kmed_loss,
        kmedians_loss_avg=kmed_loss / x_dense.shape[0],
        ari=label_metrics["ari"],
        nmi=label_metrics["nmi"],
        extra=json.dumps(result.extra, sort_keys=True, default=str),
    )


def main(argv: Optional[list[str]] = None) -> None:
    args = make_parser().parse_args(argv)
    selected = _parse_algorithms(args.algorithms)
    allowed = {"sklearn", "sklearn_kmedians_like"}
    unknown = sorted(set(selected) - allowed)
    if unknown:
        raise ValueError(f"unknown algorithms: {unknown}; choices are {sorted(allowed)}")

    rows = []
    for run_idx in range(args.runs):
        seed = args.seed + run_idx
        x_sparse, true_labels, meta = make_idgraph_tag_mixture(
            n=args.n,
            k=args.k,
            mean_active_tags=args.mean_active_tags,
            seed=seed,
        )
        x_dense = make_feature_hash_view(
            x_sparse,
            n_features=args.hash_dim,
            radius=args.radius,
            seed=seed,
        )
        ctx = FitContext(
            k=args.k,
            rho=None,
            epsilon=0.0,
            delta=0.0,
            bounds=Bounds.symmetric(x_dense.shape[1], args.radius),
            seed=seed,
            repo_root=None,
        )
        for name in selected:
            print(f"run={run_idx} algorithm={name}", flush=True)
            row = _run_one(name, x_dense, true_labels, ctx, args)
            row.run = run_idx
            rows.append(row.to_dict())
            print(
                f"  loss_avg={row.kmeans_loss_avg:.6g} "
                f"kmed_avg={row.kmedians_loss_avg:.6g} "
                f"ari={row.ari} elapsed={row.elapsed_s:.3f}s",
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

    ok = df[df["status"] == "ok"] if "status" in df.columns else df
    if not ok.empty:
        summary = ok.groupby("algorithm").agg(
            kmeans_loss_avg_mean=("kmeans_loss_avg", "mean"),
            ari_mean=("ari", "mean"),
            nmi_mean=("nmi", "mean"),
            elapsed_s_mean=("elapsed_s", "mean"),
        )
        print("\nSummary:")
        print(summary.sort_values("kmeans_loss_avg_mean"))


if __name__ == "__main__":
    main()
