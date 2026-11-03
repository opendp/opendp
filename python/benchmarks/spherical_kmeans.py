"""Manual synthetic/public-data benchmark; not a statistical acceptance test.

Run with the full-dispatch native library and scikit-learn extra installed:
    python benchmarks/spherical_kmeans.py --baseline /path/to/old/_spherical_lloyd_binary.py --output /tmp/spherical.json

Each fit runs in a fresh subprocess. Seeds control only public data generation
and initialization, never DP noise. Public initial centers come from the fixed
synthetic generator, not estimates computed from the sampled records.
"""

import argparse
import importlib.util
import json
import multiprocessing
import platform
import resource
import time
import tracemalloc
from pathlib import Path

import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from opendp.extras.sklearn.cluster import SphericalKMeans

np = import_optional_dependency("numpy")
csr_array = import_optional_dependency("scipy.sparse").csr_array
linear_sum_assignment = import_optional_dependency(
    "scipy.optimize"
).linear_sum_assignment


def dataset(case):
    rng = np.random.default_rng(2026)
    d = 20000 if case == "high-dimensional" else 2048
    sizes = [2400, 500, 100] if case == "unequal-clusters" else [1000, 1000, 1000]
    # All three support sets are public, including both low and high feature IDs.
    supports = [np.arange(32), np.arange(d // 2, d // 2 + 32), np.arange(d - 32, d)]
    background = np.setdiff1d(np.arange(d), np.concatenate(supports))
    rows: list[int] = []
    cols: list[int] = []
    truth: list[int] = []
    for cluster, n in enumerate(sizes):
        for _ in range(n):
            r = int(rng.choice([8, 32, 256])) if case != "high-dimensional" else 16
            features = np.concatenate(
                (
                    rng.choice(supports[cluster], 8, replace=False),
                    rng.choice(background, r - 8, replace=False),
                )
            )
            rows.extend([len(truth)] * r)
            cols.extend(features)
            truth.append(cluster)
    x = csr_array((np.ones(len(cols), bool), (rows, cols)), shape=(len(truth), d))
    public = csr_array(
        (np.ones(96), (np.repeat(np.arange(3), 32), np.concatenate(supports))),
        shape=(3, d),
    )
    return x, np.asarray(truth), public


def worker(connection, case, kind, variant, seed, baseline):
    try:
        dp.enable_features("contrib")
        x, truth, public = dataset(case)
        kwargs = dict(n_clusters=3, max_iter=5, random_state=seed)
        cls = SphericalKMeans
        if variant in ("previous-PR", "normalized-sparse"):
            spec = importlib.util.spec_from_file_location(
                "spherical_baseline", baseline
            )
            assert spec is not None and spec.loader is not None
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            if variant == "previous-PR":
                cls = module.SphericalKMeans
        if variant != "previous-PR":
            kwargs["max_features_per_center"] = {
                "partition-top32": 32,
                "partition-top128": 128,
            }.get(variant)
            kwargs["init"] = public if variant.startswith("public") else "random"
            if variant == "normalized-sparse":
                kwargs["init"] = module._initial_centers(3, x.shape[1], 96, seed)
            kwargs["max_l1_contribution"] = 4.0 if variant.endswith("capped") else None
        model = cls(**kwargs)
        measure, budget = (
            (dp.max_divergence(), 4.0)
            if kind == "epsilon"
            else (dp.zero_concentrated_divergence(), 0.5)
        )
        tracemalloc.start()
        start = time.perf_counter()
        if variant == "previous-PR":
            # The previous composition can exceed its nominal budget by a few ulps.
            # Apply negligible calibration slack and verify the actual outer budget.
            carrier = dp.sklearn.SklearnFit(x)
            measurement = model.make(
                dp.domain_of(carrier, infer=True),
                dp.symmetric_distance(),
                measure,
                1,
                budget * (1 - 1e-12),
            )
            assert measurement.check(1, budget)
            model._ingest_release(measurement(carrier))
        else:
            model.fit(
                x,
                privacy_unit=(dp.symmetric_distance(), 1),
                privacy_loss=(measure, budget),
            )
        elapsed = time.perf_counter() - start
        peak = tracemalloc.get_traced_memory()[1]
        tracemalloc.stop()
        predicted = model.predict(x)
        confusion = np.zeros((3, 3), int)
        np.add.at(confusion, (truth, predicted), 1)
        a, b = linear_sum_assignment(-confusion)
        accuracy = float(confusion[a, b].sum() / len(truth))
        balanced_accuracy = float(np.mean(confusion[a, b] / confusion.sum(axis=1)[a]))
        objective = float(1 - np.min(model.transform(x), axis=1).mean())
        rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        rss_mb = rss / (1024**2 if platform.system() == "Darwin" else 1024)
        connection.send(
            dict(
                case=case,
                kind=kind,
                variant=variant,
                seed=seed,
                accuracy=accuracy,
                balanced_accuracy=balanced_accuracy,
                spherical_objective=objective,
                fit_seconds=elapsed,
                traced_peak_mb=peak / 1024**2,
                process_peak_rss_mb=rss_mb,
            )
        )
    except Exception as exc:
        connection.send(dict(error=repr(exc), case=case, kind=kind, variant=variant))
    finally:
        connection.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--repeats", type=int, default=3)
    variants = [
        "previous-PR",
        "normalized-sparse",
        "partition",
        "partition-top32",
        "partition-top128",
        "partition-capped",
        "public",
        "public-capped",
    ]
    parser.add_argument("--variants", nargs="+", choices=variants, default=variants)
    args = parser.parse_args()
    results = []
    context = multiprocessing.get_context("spawn")
    for case in ["mixed-lengths", "unequal-clusters", "high-dimensional"]:
        for kind in ["epsilon", "rho"]:
            for variant in args.variants:
                for seed in range(args.repeats):
                    parent, child = context.Pipe(duplex=False)
                    process = context.Process(
                        target=worker,
                        args=(child, case, kind, variant, seed, args.baseline),
                    )
                    process.start()
                    child.close()
                    result = parent.recv()
                    process.join()
                    if process.exitcode or "error" in result:
                        raise RuntimeError(result)
                    results.append(result)
                    Path(args.output).write_text(
                        json.dumps(
                            dict(
                                platform=platform.platform(),
                                numpy=np.__version__,
                                results=results,
                            ),
                            indent=2,
                        )
                        + "\n"
                    )
                    print(json.dumps(result), flush=True)


if __name__ == "__main__":
    main()
