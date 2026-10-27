"""Benchmark OpenDP's AIM with Gem+ generator and/or PGM on Criteo, as in the Gem+ paper.

Data: CriteoPrivateAd, https://huggingface.co/datasets/criteo/CriteoPrivateAd
(CC BY-SA 4.0), one folder per day. Download one day, e.g.
  huggingface-cli download criteo/CriteoPrivateAd --repo-type dataset \\
      --include "data/day_int=1/*" --local-dir CriteoPrivateAd

Preprocessing, from the Gem+ paper (MGCB25, Section 4):
  "we select an entire day's data with approximately 2.5 million rows"
  "After removing columns that are IDs or arrays, we retain 123 columns"
  "Missing (null) values are zero-imputed and numerical columns are uniformly
   discretized into 32 bins"

Assumptions (the paper does not say):
  day      --input; the row count is printed, the paper's day had ~2.5M.
  columns  the first 30 columns left after dropping IDs and arrays.
  coding   a numeric column with more than 32 distinct values is cut into 32
           equal-width bins; any other column keeps its own values, mapped to
           0..k-1, as in the AIM paper's Adult.
  privacy  one row (one ad display) per record.

Settings: 30 columns, eps 1, delta 1e-6, 3 trials (Gem+ paper, Table 1:
AIM 0.0904, Gem+ 0.0573). AIM uses OpenDP's defaults with the AIM reference
code's one-way split. Gem+ --config paper: steps 1500, batch 25000 (Gem+
paper); dim 512, hidden 1024x1024, Adam 1e-4 with cosine decay to 1e-8 each
round (GEM code).

Error, as in the AIM paper: for each k-way marginal, L1 distance between the
model's counts and the true counts divided by N; err3 is the mean over all
3-way marginals, err2 over all 2-way ones.

The coded data is cached next to --out and reused. PGM may run out of memory
at 30 columns; a failed run is reported and the rest continue.

Examples:
  # smoke test
  python criteo_benchmark.py --input CriteoPrivateAd/data/day_int=1 --columns 5 \\
      --estimator gem --config cpu --rounds 2 --trials 1
  # Gem+ paper, Table 1
  python criteo_benchmark.py --input CriteoPrivateAd/data/day_int=1 --out criteo_table1.csv

Each run is appended to --out (CSV) as soon as it finishes.
"""

from __future__ import annotations

import argparse
import csv
import functools
import itertools
import json
import math
import statistics
import time
import traceback
from pathlib import Path

import jax

jax.config.update("jax_enable_x64", True)  # before mbi is imported; both estimators need float64

import numpy as np
import optax
import polars as pl

import opendp.prelude as dp
from opendp.extras.mbi._gem import gem_plus

dp.enable_features("contrib")


CONFIGS = {
    "paper": dict(steps=1500, batch=25000, dim=512, hidden=(1024, 1024)),
    "cpu": dict(steps=1500, batch=1000, dim=32, hidden=(128, 128)),
}
LEARNING_RATE = 1e-4
ETA_MIN = 1e-8
BINS = 32

# Identifier columns named in the CriteoPrivateAd paper (Section 4.2), plus the
# day partition. The Gem+ paper keeps 123 columns; the count is checked below.
ID_COLUMNS = ["id", "uuid", "campaign_id", "publisher_id", "display_order", "day_int"]
PAPER_COLUMNS = 123


# AIM's epsilon -> rho conversion (private-pgm mechanisms/cdp2adp.py)


def cdp_delta(rho, eps):
    if rho == 0:
        return 0
    amin, amax = 1.01, (eps + 1) / (2 * rho) + 2
    for _ in range(1000):
        alpha = (amin + amax) / 2
        derivative = (2 * alpha - 1) * rho - eps + math.log1p(-1.0 / alpha)
        if derivative < 0:
            amin = alpha
        else:
            amax = alpha
    delta = math.exp((alpha - 1) * (alpha * rho - eps) + alpha * math.log1p(-1 / alpha)) / (alpha - 1.0)
    return min(delta, 1.0)


def cdp_rho(eps, delta):
    rhomin, rhomax = 0.0, eps + 1
    for _ in range(1000):
        rho = (rhomin + rhomax) / 2
        if cdp_delta(rho, eps) <= delta:
            rhomin = rho
        else:
            rhomax = rho
    return rhomin


# Data


def encode(col: pl.Series) -> tuple[pl.Series, int]:
    """Zero-imputes, then codes as 0..k-1. Returns (codes, k)."""
    if col.dtype == pl.Boolean:
        col = col.cast(pl.Int8)
    if col.dtype.is_numeric():
        if col.dtype.is_float():
            col = col.fill_nan(None)
        col = col.fill_null(0)
        if col.n_unique() > BINS:
            lo, hi = col.min(), col.max()
            codes = ((col - lo) / (hi - lo) * BINS).floor().clip(0, BINS - 1)
            return codes.cast(pl.Int32), BINS
    else:
        col = col.cast(pl.String).fill_null("0")
    return col.rank("dense").cast(pl.Int32) - 1, col.n_unique()


def prepare(input_path: str, n_columns: int, cache: Path) -> tuple[str, str, dict[str, int]]:
    """Codes the first n_columns usable columns, cached as CSV. Returns (name, path, sizes)."""
    name = f"criteo-{Path(input_path).name.replace('=', '')}-first{n_columns}"
    data_path, domain_path = cache / f"{name}.csv", cache / f"{name}-domain.json"
    if data_path.exists() and domain_path.exists():
        return name, str(data_path), json.loads(domain_path.read_text())

    p = Path(input_path)
    df = pl.read_parquet(str(p / "*.parquet") if p.is_dir() else str(p))
    print(f"{input_path}: {df.height:,} rows, {df.width} columns")

    # Drop IDs and arrays (Gem+ paper)
    usable = [
        c for c, dtype in df.schema.items()
        if c not in ID_COLUMNS and not isinstance(dtype, (pl.List, pl.Array, pl.Struct))
    ]
    print(f"dropped {df.width - len(usable)} ID/array columns, kept {len(usable)} (paper: {PAPER_COLUMNS})")
    if n_columns > len(usable):
        raise SystemExit(f"--columns {n_columns} is more than the {len(usable)} usable columns")

    coded, sizes = {}, {}
    for c in usable[:n_columns]:
        coded[c], sizes[c] = encode(df[c])
    cache.mkdir(parents=True, exist_ok=True)
    pl.DataFrame(coded).write_csv(data_path)
    domain_path.write_text(json.dumps(sizes, indent=1))
    print(f"cached {data_path}")
    return name, str(data_path), sizes


def marginal_error(table, truth: pl.DataFrame, clique, n: int) -> float:
    attrs = list(clique)
    fitted = table.project_melted(attrs).rename({"len": "fitted"})
    joined = fitted.join(
        truth.rename({"len": "true"}), on=attrs, how="full", coalesce=True, nulls_equal=True
    ).fill_null(0)
    diff = joined["fitted"].cast(pl.Float64) - joined["true"].cast(pl.Float64)
    return float(diff.abs().sum() / n)


# Estimators


class Counted:
    """Wraps an estimator to count calls: the table's first fit plus one per AIM round."""

    def __init__(self, fn):
        self.fn = fn
        self.calls = 0

    def __call__(self, domain, loss_fn, **kwargs):
        self.calls += 1
        return self.fn(domain, loss_fn, **kwargs)


def make_estimator(name, config, seed):
    if name == "pgm":
        return Counted(dp.mbi.mirror_descent)
    setting = CONFIGS[config]
    schedule = optax.cosine_decay_schedule(
        LEARNING_RATE, decay_steps=setting["steps"], alpha=ETA_MIN / LEARNING_RATE
    )
    return Counted(functools.partial(gem_plus, optimizer=optax.adam(schedule), seed=seed, **setting))


# One release


def run_once(path, sizes, estimator, rho, rounds):
    columns = list(sizes)
    aim_kwargs = {"estimator": estimator, "oneway_split": 0.9 * len(columns) / (16 * len(columns))}
    if rounds is not None:
        aim_kwargs["rounds"] = rounds
    context = dp.Context.compositor(
        data=pl.scan_csv(path).select(columns),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=rho),
    )
    start = time.perf_counter()
    table = (
        context.query(rho=rho)
        .contingency_table(
            keys={c: list(range(k)) for c, k in sizes.items()},
            algorithm=dp.mbi.AIM(**aim_kwargs),
        )
        .release()
    )
    return table, time.perf_counter() - start


# Main


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--input", required=True, help="one day: a day_int=N folder, or a parquet file")
    p.add_argument("--columns", type=int, default=30, help="use the first N usable columns (Table 1: 30)")
    p.add_argument("--estimator", nargs="+", choices=["gem", "pgm"], default=["gem", "pgm"])
    p.add_argument("--config", choices=list(CONFIGS), default="paper", help="Gem+ settings")
    p.add_argument("--eps", nargs="+", type=float, default=[1.0])
    p.add_argument("--delta", type=float, default=1e-6, help="Gem+ paper, Table 1: 1e-6")
    p.add_argument("--trials", type=int, default=3, help="Gem+ paper: 3")
    p.add_argument("--rounds", type=int, default=None, help="cap on AIM rounds, for smoke tests")
    p.add_argument("--out", default="criteo_results.csv")
    args = p.parse_args()

    out_path = Path(args.out)
    name, path, sizes = prepare(args.input, args.columns, out_path.parent / "criteo_cache")
    columns = list(sizes)
    df = pl.read_csv(path)
    n = df.height
    pairs = list(itertools.combinations(columns, 2))
    triples = list(itertools.combinations(columns, 3))
    budgets = [(eps, cdp_rho(eps, args.delta)) for eps in args.eps]

    print(f"devices: {jax.devices()}")
    print(f"{name}: {n:,} rows, {len(columns)} columns, domain sizes {sorted(sizes.values())}")
    print(f"AIM rounds T = {args.rounds or 16 * len(columns)}; budgets (eps, rho): "
          f"{[(e, round(r, 6)) for e, r in budgets]}")
    print(f"computing true marginals ({len(pairs)} pairs, {len(triples)} triples)...", flush=True)
    truth = {cl: df.group_by(list(cl)).len() for cl in pairs + triples}

    fields = ["dataset", "rows", "columns", "estimator", "config", "eps", "delta", "rho",
              "trial", "err2", "err3", "rounds", "seconds"]
    new_file = not out_path.exists()
    rows = []
    with out_path.open("a", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        if new_file:
            writer.writeheader()

        for eps, rho in budgets:
            for est in args.estimator:
                for trial in range(args.trials):
                    estimator = make_estimator(est, args.config, seed=trial)
                    try:
                        table, seconds = run_once(path, sizes, estimator, rho, args.rounds)
                    except Exception:
                        print(f"FAILED {est} eps={eps} trial {trial + 1}", flush=True)
                        traceback.print_exc()
                        continue
                    row = {
                        "dataset": name, "rows": n, "columns": len(columns),
                        "estimator": est, "config": args.config if est == "gem" else "",
                        "eps": eps, "delta": args.delta, "rho": round(rho, 6), "trial": trial,
                        "err2": round(float(np.mean([marginal_error(table, truth[c], c, n) for c in pairs])), 5),
                        "err3": round(float(np.mean([marginal_error(table, truth[c], c, n) for c in triples])), 5),
                        "rounds": estimator.calls - 1,
                        "seconds": round(seconds, 1),
                    }
                    writer.writerow(row)
                    f.flush()
                    rows.append(row)
                    print(f"{est} eps={eps} trial {trial + 1}/{args.trials}  err2={row['err2']:.4f}  "
                          f"err3={row['err3']:.4f}  rounds={row['rounds']}  {row['seconds']:.0f}s", flush=True)

    print("\nsummary (mean ± sd over trials)")
    for eps, _ in budgets:
        for est in args.estimator:
            group = [r for r in rows if r["eps"] == eps and r["estimator"] == est]
            if not group:
                print(f"  eps={eps}  {est}: no finished runs")
                continue
            err3 = [r["err3"] for r in group]
            sd = statistics.stdev(err3) if len(err3) > 1 else 0.0
            print(f"  eps={eps}  {est}: err3 {statistics.mean(err3):.4f} ± {sd:.4f} over {len(group)} runs, "
                  f"{statistics.mean(r['seconds'] for r in group):.0f}s per run")
    print(f"\nresults appended to {out_path}")


if __name__ == "__main__":
    main()
