"""Benchmark OpenDP's AIM with Gem+ and/or PGM as the estimator.

Any Gem+ setting can be given several values; every combination is run.

Datasets (--dataset):
  adult  UCI Adult, integer-coded, in DATA_DIR/adult.csv with column sizes in
         DATA_DIR/adult-domain.json. Point --data-dir at aim_adult/ for the AIM
         paper's version (32 bins for numeric columns), or at the folder with
         GEM's version. One row per person.
  lfs    5 columns of OpenDP's France LFS example (YEAR, QUARTER, SEX, AGE,
         ILOSTAT). Keys are read from the data, which is fine offline but not
         for a real release. Each person contributes up to 36 rows.

Error, as in the AIM paper: for each k-way marginal, L1 distance between the
model's counts and the true counts divided by N; err3 is the mean over all
3-way marginals (the paper's ALL-3WAY workload), err2 over all 2-way ones.

Settings and their sources:
  AIM        OpenDP defaults, which match the AIM paper: all 3-way workload,
             T = 16 * columns rounds, max_size 80 MB, measure_split 0.9.
  budget     --eps is converted to zCDP rho with AIM's cdp_rho at --delta
             (1e-9 in the AIM paper); or give --rho directly.
  one-ways   --oneway aim-paper spends 0.9 * columns / T of the budget on the
             one-way marginals, as AIM's reference code does (mechanisms/aim.py);
             --oneway opendp keeps OpenDP's default split.
  Gem+ --config paper:
             steps 1500 and batch 25000          Gem+ paper, Table 1
             dim 512, hidden 1024x1024           GEM code (gem.py)
             Adam lr 1e-4, cosine decay to 1e-8,
             restarted every round               GEM code (gem.py lines 204-205)
             loss: unweighted L1                 Gem+ paper, Algorithm 1 line 8
  Gem+ --config cpu: the paper config with a smaller network (dim 32,
             hidden 128x128) and GEM's default batch (syndata_size 1000).
             Label results from it as reduced for CPU.
  --loss weighted is mbi's stddev-weighted L1: an experiment, not part of Gem+.

Examples:
  # smoke test
  python aim_benchmark.py --dataset adult --data-dir aim_adult --config cpu --eps 1 --rounds 2
  # overnight, laptop
  python aim_benchmark.py --dataset adult --data-dir aim_adult --config cpu --eps 1 --out adult_eps1.csv
  # LFS, comparing step counts and losses (4 Gem+ combinations, plus PGM)
  python aim_benchmark.py --dataset lfs --estimator gem pgm --config cpu --eps 1 \\
      --steps 100 1500 --loss unweighted weighted --trials 3
  # GPU, paper settings
  python aim_benchmark.py --dataset adult --data-dir aim_adult --config paper \\
      --eps 0.1 1 10 --trials 5 --out gpu_results.csv

On a GPU, install JAX with CUDA first, e.g.  pip install -U "jax[cuda12]"
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

LFS_COLUMNS = ["YEAR", "QUARTER", "SEX", "AGE", "ILOSTAT"]
CONTRIBUTIONS = {"adult": 1, "lfs": 36}


# ---------------------------------------------------------------------------
# Privacy budget: AIM's epsilon -> rho conversion
# (private-pgm mechanisms/cdp2adp.py, from Canonne, Kamath and Steinke 2020)
# ---------------------------------------------------------------------------


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


# ---------------------------------------------------------------------------
# Data
# ---------------------------------------------------------------------------


def load_data(dataset: str, data_dir: str):
    """Returns (path, columns, dataframe, keys)."""
    if dataset == "adult":
        folder = Path(data_dir)
        sizes = json.loads((folder / "adult-domain.json").read_text())
        columns = list(sizes)
        path = str(folder / "adult.csv")
        df = pl.read_csv(path).select(columns)
        keys = {c: list(range(sizes[c])) for c in columns}
        return path, columns, df, keys

    path = dp.examples.get_france_lfs_path()
    columns = LFS_COLUMNS
    df = pl.scan_csv(path).select(columns).collect()
    keys = {c: df[c].drop_nulls().unique().sort().to_list() for c in columns}
    return path, columns, df, keys


def marginal_error(table, truth: pl.DataFrame, clique, n: int) -> float:
    attrs = list(clique)
    fitted = table.project_melted(attrs).rename({"len": "fitted"})
    joined = fitted.join(
        truth.rename({"len": "true"}), on=attrs, how="full", coalesce=True, nulls_equal=True
    ).fill_null(0)
    diff = joined["fitted"].cast(pl.Float64) - joined["true"].cast(pl.Float64)
    return float(diff.abs().sum() / n)


# ---------------------------------------------------------------------------
# Estimators
# ---------------------------------------------------------------------------


class Counted:
    """Wraps an estimator to count calls: the table's first fit plus one per AIM round."""

    def __init__(self, fn):
        self.fn = fn
        self.calls = 0

    def __call__(self, domain, loss_fn, **kwargs):
        self.calls += 1
        return self.fn(domain, loss_fn, **kwargs)


def make_estimator(setting, seed):
    if setting is None:
        return Counted(dp.mbi.mirror_descent)
    schedule = optax.cosine_decay_schedule(
        LEARNING_RATE, decay_steps=setting["steps"], alpha=ETA_MIN / LEARNING_RATE
    )
    return Counted(
        functools.partial(
            gem_plus,
            optimizer=optax.adam(schedule),
            steps=setting["steps"],
            batch=setting["batch"],
            dim=setting["dim"],
            hidden=setting["hidden"],
            seed=seed,
            weighted=setting["loss"] == "weighted",
        )
    )


def label(setting):
    if setting is None:
        return "pgm"
    hidden = "x".join(map(str, setting["hidden"]))
    return f"gem steps={setting['steps']} batch={setting['batch']} dim={setting['dim']} hidden={hidden} {setting['loss']}"


# ---------------------------------------------------------------------------
# One release
# ---------------------------------------------------------------------------


def run_once(path, columns, keys, contributions, estimator, rho, rounds, oneway):
    aim_kwargs = {"estimator": estimator}
    if rounds is not None:
        aim_kwargs["rounds"] = rounds
    if oneway == "aim-paper":
        # Use the full T even when --rounds caps the run: with a small cap the
        # formula would exceed the whole budget.
        T = 16 * len(columns)
        aim_kwargs["oneway_split"] = 0.9 * len(columns) / T

    context = dp.Context.compositor(
        data=pl.scan_csv(path).select(columns),
        privacy_unit=dp.unit_of(contributions=contributions),
        privacy_loss=dp.loss_of(rho=rho),
    )
    start = time.perf_counter()
    table = (
        context.query(rho=rho)
        .contingency_table(keys=keys, algorithm=dp.mbi.AIM(**aim_kwargs))
        .release()
    )
    return table, time.perf_counter() - start


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def parse_hidden(text: str) -> tuple[int, ...]:
    return tuple(int(w) for w in text.split("x"))


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--dataset", choices=["adult", "lfs"], default="adult")
    p.add_argument("--data-dir", default="aim_adult", help="adult only: folder with adult.csv and adult-domain.json")
    p.add_argument("--estimator", nargs="+", choices=["gem", "pgm"], default=["gem", "pgm"])

    budget = p.add_mutually_exclusive_group()
    budget.add_argument("--eps", nargs="+", type=float, help="converted to rho at --delta (default: 1)")
    budget.add_argument("--rho", nargs="+", type=float, help="zCDP budget, used as given")
    p.add_argument("--delta", type=float, default=1e-9, help="AIM paper: 1e-9")

    p.add_argument("--trials", type=int, default=1, help="AIM paper: 5; Gem+ paper: 3")
    p.add_argument("--rounds", type=int, default=None, help="cap on AIM rounds (default 16 per column)")
    p.add_argument("--oneway", choices=["aim-paper", "opendp"], default="aim-paper")

    p.add_argument("--config", choices=list(CONFIGS), default="paper", help="base Gem+ settings")
    p.add_argument("--steps", nargs="+", type=int, help="override; several values run each")
    p.add_argument("--batch", nargs="+", type=int, help="override; several values run each")
    p.add_argument("--dim", nargs="+", type=int, help="override; several values run each")
    p.add_argument("--hidden", nargs="+", type=parse_hidden, help="override, e.g. 128x128 1024x1024")
    p.add_argument("--loss", nargs="+", choices=["unweighted", "weighted"], default=["unweighted"])

    p.add_argument("--out", default="results.csv")
    args = p.parse_args()

    # Budgets: (eps, rho) pairs
    if args.rho:
        budgets = [("", rho) for rho in args.rho]
    else:
        budgets = [(eps, cdp_rho(eps, args.delta)) for eps in (args.eps or [1.0])]

    # Every combination of Gem+ settings
    base = CONFIGS[args.config]
    grid = itertools.product(
        args.steps or [base["steps"]],
        args.batch or [base["batch"]],
        args.dim or [base["dim"]],
        args.hidden or [base["hidden"]],
        args.loss,
    )
    gem_settings = [
        dict(steps=s, batch=b, dim=d, hidden=h, loss=loss) for s, b, d, h, loss in grid
    ]
    for setting in gem_settings:
        if setting["steps"] % 50:
            p.error(f"steps must be multiples of 50, got {setting['steps']}")

    settings = []
    for name in args.estimator:
        settings += [None] if name == "pgm" else gem_settings

    path, columns, df, keys = load_data(args.dataset, args.data_dir)
    n = df.height
    pairs = list(itertools.combinations(columns, 2))
    triples = list(itertools.combinations(columns, 3))
    truth = {cl: df.group_by(list(cl)).len() for cl in pairs + triples}

    print(f"devices: {jax.devices()}")
    print(f"{args.dataset}: {n:,} rows, {len(columns)} columns, contributions {CONTRIBUTIONS[args.dataset]}")
    print(f"AIM rounds T = {args.rounds or 16 * len(columns)}, one-ways: {args.oneway}")
    print(f"budgets (eps, rho): {[(e, round(r, 6)) for e, r in budgets]}")
    print(f"Adam {LEARNING_RATE}, cosine to {ETA_MIN} each round")
    print(f"{len(budgets) * len(settings) * args.trials} runs:")
    for s in settings:
        print(f"  {label(s)}")
    print(flush=True)

    out_path = Path(args.out)
    fields = [
        "dataset", "estimator", "eps", "delta", "rho", "trial", "err2", "err3", "rounds", "seconds",
        "config", "steps", "batch", "dim", "hidden", "loss", "oneway",
    ]
    new_file = not out_path.exists()
    rows = []
    with out_path.open("a", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        if new_file:
            writer.writeheader()

        for eps, rho in budgets:
            for setting in settings:
                for trial in range(args.trials):
                    estimator = make_estimator(setting, seed=trial)
                    table, seconds = run_once(
                        path, columns, keys, CONTRIBUTIONS[args.dataset],
                        estimator, rho, args.rounds, args.oneway,
                    )
                    gem = setting is not None
                    row = {
                        "dataset": args.dataset,
                        "estimator": "gem" if gem else "pgm",
                        "eps": eps,
                        "delta": args.delta if eps != "" else "",
                        "rho": round(rho, 6),
                        "trial": trial,
                        "err2": round(float(np.mean([marginal_error(table, truth[c], c, n) for c in pairs])), 5),
                        "err3": round(float(np.mean([marginal_error(table, truth[c], c, n) for c in triples])), 5),
                        "rounds": estimator.calls - 1,
                        "seconds": round(seconds, 1),
                        "config": args.config if gem else "",
                        "steps": setting["steps"] if gem else "",
                        "batch": setting["batch"] if gem else "",
                        "dim": setting["dim"] if gem else "",
                        "hidden": "x".join(map(str, setting["hidden"])) if gem else "",
                        "loss": setting["loss"] if gem else "",
                        "oneway": args.oneway,
                    }
                    writer.writerow(row)
                    f.flush()
                    rows.append((label(setting), row))
                    print(
                        f"rho={rho:.6f}  {label(setting)}  trial {trial + 1}/{args.trials}  "
                        f"err2={row['err2']:.4f}  err3={row['err3']:.4f}  "
                        f"rounds={row['rounds']}  {row['seconds']:.0f}s",
                        flush=True,
                    )

    print("\nsummary (mean ± sd over trials)")
    for eps, rho in budgets:
        print(f"\neps={eps}  rho={rho:.6f}")
        print(f"  {'err2':>17}{'err3':>18}{'rounds':>9}{'seconds':>10}   setting")
        for setting in settings:
            group = [r for lab, r in rows if lab == label(setting) and r["rho"] == round(rho, 6)]

            def fmt(key, digits):
                vals = [r[key] for r in group]
                sd = statistics.stdev(vals) if len(vals) > 1 else 0.0
                return f"{statistics.mean(vals):.{digits}f} ± {sd:.{digits}f}"

            print(
                f"  {fmt('err2', 4):>17}{fmt('err3', 4):>18}"
                f"{statistics.mean(r['rounds'] for r in group):>9.1f}"
                f"{statistics.mean(r['seconds'] for r in group):>10.0f}   {label(setting)}"
            )
    print(f"\nresults appended to {out_path}")


if __name__ == "__main__":
    main()