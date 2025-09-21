#!/usr/bin/env python3
"""
Parking Lot Predictive Demo for the Toeplitz Mechanism (predictive focus)

This demo evaluates two predictive applications using continual DP aggregates
from the Toeplitz mechanism over the parking datasets:

A) Single-hour rolling-average predictor (per spot, per hour-of-day):
   - For a chosen spot and hour, predicts availability for the next day’s same hour
     using a rolling window of the previous Y days.
   - Training statistic: the average availability at that hour across the last Y days,
     obtained from DP prefix sums via the Toeplitz continual release.

B) Per-hour classifier trained via DP sums (per spot):
   - For each hour-of-day h, continually maintain DP sums of availability labels.
   - Predict availability for an incoming (hour=h) example using the DP mean for that hour
     computed from a rolling window of the last Y occurrences of that hour.

We report streaming prediction performance (accuracy, Brier score, log-loss) over epsilons.

Notes:
- This demo calls the Rust Toeplitz mechanism via the OpenDP Python FFI.
- Sensitivity is 1 per appended label (event-level adjacency).

Run examples:
  python rust/src/measurements/toeplitz/demo/parking_lot_predictive_demo.py \
    --app both --epsilons 0.2 0.5 1.0 --trials 10 --window-days 14 \
    --spot-id 5 --monotonic

  python rust/src/measurements/toeplitz/demo/parking_lot_predictive_demo.py \
    --app single-hour --hour 8 --window-days 14 --epsilons 0.5 --trials 20
"""

from __future__ import annotations

import argparse
import csv
import importlib
import math
import random
import sys
from dataclasses import dataclass
from datetime import date, timedelta
from pathlib import Path
from typing import Any, Dict, List, Mapping, Tuple


def _ensure_local_opendp_on_path() -> None:
    """Allow running the demo from a source checkout without installing the Python package."""
    for parent in Path(__file__).resolve().parents:
        python_src = parent / "python" / "src"
        if (python_src / "opendp").exists():
            python_src_str = str(python_src)
            if python_src_str not in sys.path:
                sys.path.insert(0, python_src_str)
            break


def _import_opendp_module(module_name: str):
    """Import an OpenDP module, falling back to the local source tree."""
    try:
        return importlib.import_module(module_name)
    except ModuleNotFoundError:
        _ensure_local_opendp_on_path()
        try:
            return importlib.import_module(module_name)
        except ModuleNotFoundError as exc:
            raise ModuleNotFoundError(
                f"Unable to import '{module_name}'. Ensure OpenDP is installed or build the repo (pip install -e python)."
            ) from exc


dp: Any = _import_opendp_module("opendp.prelude")
_opendp_mod = _import_opendp_module("opendp.mod")
Measurement = _opendp_mod.Measurement


EXPERIMENTAL_CONFIG = {
    "epsilons": [0.2, 0.5, 1.0],
    "window_days": 14,
    "trials": 10,
    "monotonic": False,
    "seed": 0,
}


# -------------------------------
# Data loading and aggregation
# -------------------------------

Timestamp = Tuple[int, int, int, int, int]  # (Year, Month, Date, Hour, Minute)


def load_spot_hourly_labels(dataset_dir: Path, spot_id: int) -> List[Tuple[Timestamp, int]]:
    """
    Load rows for a specific spot and derive an hourly availability label.

    For each day/hour (Y, M, D, H), take the last reading in that hour and map
    Status -> availability label as: available=1 if Status==0, else 0.

    Returns a chronological list of ((Y, M, D, H), label) entries.
    """
    if not dataset_dir.exists():
        raise FileNotFoundError(f"Dataset directory not found: {dataset_dir}")

    # Map (Y, M, D, H, M) -> (second, status) to keep last reading within the minute
    last_by_slot: Dict[Tuple[int, int, int, int, int], Tuple[int, int]] = {}

    csv_files = sorted(dataset_dir.glob("*.csv"))
    if not csv_files:
        raise FileNotFoundError(f"No CSV files found in {dataset_dir}")

    for day_idx, path in enumerate(csv_files):
        with path.open("r", newline="") as f:
            reader = csv.DictReader(f)
            for row in reader:
                try:
                    sid = int(row["SpotID"])  # which spot
                    if sid != spot_id:
                        continue
                    year = int(row["Year"])  # e.g., 2025
                    month = int(row["Month"])  # 1..12
                    day = int(row["Date"])  # 1..31
                    hour = int(row["Hour"])  # 0..23
                    minute = int(row["Minute"])  # 0..59
                    second = int(row["Second"])  # 0..59
                    status = int(row["Status"])  # 0/1
                except Exception:
                    continue

                # Treat each CSV file as a successive day to provide a richer timeline
                adjusted_date = date(year, month, day) + timedelta(days=day_idx)
                key = (
                    adjusted_date.year,
                    adjusted_date.month,
                    adjusted_date.day,
                    hour,
                    minute,
                )
                prev = last_by_slot.get(key)
                if prev is None or second >= prev[0]:
                    last_by_slot[key] = (second, status)

    # Build chronological sequence
    keys_sorted = sorted(last_by_slot.keys())
    result: List[Tuple[Timestamp, int]] = []
    for (y, m, d, h, minute) in keys_sorted:
        _, status = last_by_slot[(y, m, d, h, minute)]
        # availability label: 1 if status==0 (vacant), else 0 (occupied)
        label = 1 if status == 0 else 0
        result.append(((y, m, d, h, minute), label))

    return result


def _dp_sum_last_y(vec: List[int], epsilon: float, monotonic: bool, cache: Dict[int, Measurement]) -> int:
    """Compute DP sum of a vector via Rust Toeplitz (one-shot), returning the last prefix value."""
    n = len(vec)
    if n == 0:
        return 0
    if n not in cache:
        domain = dp.vector_domain(dp.atom_domain(T=int), size=n)
        metric = dp.l1_distance(T=int)
        # enforce_monotonicity toggles isotonic regression in the Rust measurement
        cache[n] = dp.m.make_toeplitz(
            domain,
            metric,
            scale=1.0 / epsilon,
            enforce_monotonicity=monotonic,
        )
    meas = cache[n]
    prefix = meas.invoke(vec)
    return int(prefix[-1])


# -------------------------------
# Evaluation utilities
# -------------------------------

@dataclass
class Metrics:
    accuracy: float
    brier: float
    logloss: float


def brier_score(probs: List[float], labels: List[int]) -> float:
    n = len(labels)
    if n == 0:
        return 0.0
    return sum((p - y) ** 2 for p, y in zip(probs, labels)) / n


def log_loss(probs: List[float], labels: List[int], eps: float = 1e-6) -> float:
    n = len(labels)
    if n == 0:
        return 0.0
    s = 0.0
    for p, y in zip(probs, labels):
        p = min(max(p, eps), 1 - eps)
        s += - (y * math.log(p) + (1 - y) * math.log(1 - p))
    return s / n


def evaluate_single_hour_prediction(
    labels_by_day: List[int],
    config: Mapping[str, Any],
) -> Dict[float, Metrics]:
    """Application A: single-hour rolling-average predictor using DP sums (via Rust FFI)."""
    results: Dict[float, Metrics] = {}
    epsilons = config["epsilons"]
    window_days = config["window_days"]
    trials = config["trials"]
    monotonic = config["monotonic"]
    seed = config.get("seed", 0)

    base_rng = random.Random(seed)
    for eps in epsilons:
        if eps <= 0:
            continue
        # cache a measurement per window length to avoid re-allocating
        meas_cache: Dict[int, Measurement] = {}
        acc_vals: List[float] = []
        brier_vals: List[float] = []
        log_vals: List[float] = []
        for _ in range(trials):
            preds: List[float] = []
            trues: List[int] = []
            for t, y in enumerate(labels_by_day):
                # predict for day t using previous window
                denom = min(window_days, t)
                if denom > 0:
                    start = t - denom
                    vec = labels_by_day[start:t]
                    dp_sum = _dp_sum_last_y(vec, eps, monotonic, meas_cache)
                    p_hat = max(0.0, min(1.0, dp_sum / float(denom)))
                else:
                    p_hat = 0.5
                preds.append(p_hat)
                trues.append(y)
            # compute metrics across all prediction points (skip first few if desired)
            acc = sum((1 if p >= 0.5 else 0) == y for p, y in zip(preds, trues)) / len(trues)
            brier = brier_score(preds, trues)
            ll = log_loss(preds, trues)
            acc_vals.append(acc)
            brier_vals.append(brier)
            log_vals.append(ll)

        results[eps] = Metrics(
            accuracy=sum(acc_vals) / len(acc_vals),
            brier=sum(brier_vals) / len(brier_vals),
            logloss=sum(log_vals) / len(log_vals),
        )
    return results


def evaluate_per_hour_classifier(
    hourly_events: List[Tuple[int, int]],
    config: Mapping[str, Any],
) -> Dict[float, Metrics]:
    """Application B: per-hour classifier trained via DP sums (via Rust FFI).

    For each hour h, predict using DP mean over the last Y occurrences of hour h.
    """
    results: Dict[float, Metrics] = {}
    epsilons = config["epsilons"]
    window_days = config["window_days"]
    trials = config["trials"]
    monotonic = config["monotonic"]
    seed = config.get("seed", 0)

    base_rng = random.Random(seed)
    for eps in epsilons:
        if eps <= 0:
            continue
        meas_cache_by_hour: List[Dict[int, Measurement]] = [dict() for _ in range(24)]
        acc_vals: List[float] = []
        brier_vals: List[float] = []
        log_vals: List[float] = []
        for _ in range(trials):
            preds: List[float] = []
            trues: List[int] = []
            per_hour_labels: List[List[int]] = [[] for _ in range(24)]
            for h, y in hourly_events:
                hist = per_hour_labels[h]
                denom = min(window_days, len(hist))
                if denom > 0:
                    vec = hist[-denom:]
                    dp_sum = _dp_sum_last_y(vec, eps, monotonic, meas_cache_by_hour[h])
                    p_hat = max(0.0, min(1.0, dp_sum / float(denom)))
                else:
                    p_hat = 0.5
                preds.append(p_hat)
                trues.append(y)
                hist.append(y)
            acc = sum((1 if p >= 0.5 else 0) == y for p, y in zip(preds, trues)) / len(trues)
            brier = brier_score(preds, trues)
            ll = log_loss(preds, trues)
            acc_vals.append(acc)
            brier_vals.append(brier)
            log_vals.append(ll)
        results[eps] = Metrics(
            accuracy=sum(acc_vals) / len(acc_vals),
            brier=sum(brier_vals) / len(brier_vals),
            logloss=sum(log_vals) / len(log_vals),
        )
    return results


# -------------------------------
# CLI
# -------------------------------


def main() -> None:
    parser = argparse.ArgumentParser(description="Toeplitz DP predictive demo on parking lot datasets")
    parser.add_argument(
        "--dataset-dir",
        type=Path,
        default=Path(__file__).parent / "datasets",
        help="Path to datasets directory containing CSV files",
    )
    parser.add_argument(
        "--app",
        choices=["single-hour", "per-hour", "both"],
        default="both",
        help="Which application to run: single-hour rolling average or per-hour classifier",
    )
    parser.add_argument(
        "--epsilons",
        type=float,
        nargs="+",
        default=list(EXPERIMENTAL_CONFIG["epsilons"]),
        help="List of epsilon values to evaluate (sensitivity=1)",
    )
    parser.add_argument(
        "--trials",
        type=int,
        default=EXPERIMENTAL_CONFIG["trials"],
        help="Monte Carlo trials per epsilon",
    )
    parser.add_argument(
        "--monotonic",
        action="store_true",
        default=EXPERIMENTAL_CONFIG["monotonic"],
        help="Use monotonic post-processing (isotonic regression)",
    )
    parser.add_argument("--spot-id", type=int, default=5, help="SpotID to analyze")
    parser.add_argument("--hour", type=int, default=7, help="Hour-of-day (0-23) for single-hour app")
    parser.add_argument(
        "--window-days",
        type=int,
        default=EXPERIMENTAL_CONFIG["window_days"],
        help="Rolling window size in days/occurrences for training",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=EXPERIMENTAL_CONFIG["seed"],
        help="Base RNG seed for reproducibility",
    )

    args = parser.parse_args()

    # Ensure necessary feature flags are enabled in Python
    dp.enable_features("contrib", "contrib-continual")

    # Monkey-patch make_toeplitz if the local OpenDP build does not expose it yet
    if not hasattr(dp.m, "make_toeplitz"):
        import ctypes

        opendp_lib = _import_opendp_module("opendp._lib")
        lib = opendp_lib.lib
        FfiResult = opendp_lib.FfiResult
        unwrap = opendp_lib.unwrap

        def _ffi_make_toeplitz(input_domain, input_metric, *, scale: float, enforce_monotonicity: bool = True) -> Measurement:
            fn = lib.opendp_measurements__make_toeplitz
            fn.argtypes = [type(input_domain), type(input_metric), ctypes.c_double, ctypes.c_bool, ctypes.c_char_p]
            fn.restype = FfiResult
            res = fn(
                input_domain,
                input_metric,
                scale,
                enforce_monotonicity,
                b"ZeroConcentratedDivergence",
            )
            return unwrap(res, Measurement)

        setattr(dp.m, "make_toeplitz", _ffi_make_toeplitz)

    # Load per-spot hourly labels
    entries = load_spot_hourly_labels(args.dataset_dir, args.spot_id)
    if not entries:
        print(f"No entries found for SpotID={args.spot_id} in {args.dataset_dir}")
        return
    print(f"Loaded {len(entries)} hourly labels for SpotID={args.spot_id}")

    # Prepare sequences for both applications
    # A) single-hour: extract labels for selected hour across days
    labels_single_hour: List[int] = [
        label for ((_, _, _, h, _), label) in entries if h == args.hour
    ]
    print(f"Single-hour app: hour={args.hour}, samples={len(labels_single_hour)}")

    # B) per-hour classifier: chronological (hour, label) pairs
    hourly_events: List[Tuple[int, int]] = [
        (h, label) for ((_, _, _, h, _), label) in entries
    ]

    experiment_cfg = dict(EXPERIMENTAL_CONFIG)
    experiment_cfg.update(
        {
            "epsilons": list(args.epsilons),
            "window_days": args.window_days,
            "trials": args.trials,
            "monotonic": args.monotonic,
            "seed": args.seed,
        }
    )

    if args.app in ("single-hour", "both"):
        res_a = evaluate_single_hour_prediction(
            labels_by_day=labels_single_hour,
            config=experiment_cfg,
        )
        variant = "Monotonic" if args.monotonic else "Baseline"
        print(f"\nApplication A: Single-hour rolling predictor ({variant})")
        print("epsilon | scale | accuracy | brier | logloss")
        for eps in sorted(res_a.keys()):
            scale = 1.0 / eps
            m = res_a[eps]
            print(f"{eps:7.3f} | {scale:5.2f} | {m.accuracy:8.3f} | {m.brier:5.3f} | {m.logloss:7.3f}")

    if args.app in ("per-hour", "both"):
        res_b = evaluate_per_hour_classifier(
            hourly_events=hourly_events,
            config=experiment_cfg,
        )
        variant = "Monotonic" if args.monotonic else "Baseline"
        print(f"\nApplication B: Per-hour classifier via DP sums ({variant})")
        print("epsilon | scale | accuracy | brier | logloss")
        for eps in sorted(res_b.keys()):
            scale = 1.0 / eps
            m = res_b[eps]
            print(f"{eps:7.3f} | {scale:5.2f} | {m.accuracy:8.3f} | {m.brier:5.3f} | {m.logloss:7.3f}")


if __name__ == "__main__":
    main()
