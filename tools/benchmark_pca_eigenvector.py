"""Benchmark either checkout without changing the production sampler.

Example (use the SAME Python, shared library and script for both checkouts):
    OPENDP_LIB_DIR=/path/to/lib python tools/benchmark_pca_eigenvector.py \
        --source /path/to/checkout --output baseline.json

Timings use OpenDP's CSPRNG, without instrumentation. Separate seeded diagnostics
count proposals, not identical outputs across implementations. Each timed call
includes eigendecomposition, rejection sampling and Measurement output conversion;
constructor, matrix construction, imports and warmup are excluded.
"""

import argparse
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import sys
import time

# Set before importing NumPy; avoid dimension-dependent thread-pool overhead.
for variable in ("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS",
                 "VECLIB_MAXIMUM_THREADS", "BLIS_NUM_THREADS"):
    os.environ[variable] = "1"

import numpy as np


class CountingRNG:
    def __init__(self, seed):
        self.rng = np.random.default_rng(seed)
        self.proposals = 0

    def multivariate_normal(self, *args, **kwargs):
        self.proposals += 1
        return self.rng.multivariate_normal(*args, **kwargs)

    def standard_normal(self, *args, **kwargs):
        self.proposals += 1
        return self.rng.standard_normal(*args, **kwargs)

    def random(self):
        return self.rng.random()


def command(*args):
    return subprocess.check_output(args, text=True).strip()


def timeout_handler(signum, frame):
    raise TimeoutError("case exceeded time budget (no extrapolation)")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=31)
    parser.add_argument("--diagnostics", type=int, default=128)
    parser.add_argument("--case-seconds", type=int, default=120)
    args = parser.parse_args()
    source = args.source.resolve()
    sys.path.insert(0, str(source / "python/src"))
    import opendp.prelude as dp
    from opendp._lib import lib_path
    from opendp.extras.numpy import _nuclear_norm_distance, _symmetric_matrix_domain
    import opendp.extras.sklearn._make_eigenvector as eigenvector

    dp.enable_features("contrib", "idealized-numerics")
    config = io.StringIO()
    with contextlib.redirect_stdout(config):
        np.show_config()
    sampler_path = Path(eigenvector.__file__)
    result = {
        "sha": command("git", "-C", str(source), "rev-parse", "HEAD"),
        "diff": command("git", "-C", str(source), "diff", "--", str(sampler_path)),
        "sampler_sha256": hashlib.sha256(sampler_path.read_bytes()).hexdigest(),
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "python": sys.version, "executable": sys.executable,
        "numpy": np.__version__, "numpy_config": config.getvalue(),
        "platform": platform.platform(), "machine": platform.machine(),
        "cpu": command("sysctl", "-n", "machdep.cpu.brand_string")
        if sys.platform == "darwin" else platform.processor(),
        "logical_cpus": os.cpu_count(), "lib_path": str(lib_path),
        "lib_sha256": hashlib.sha256(Path(lib_path).read_bytes()).hexdigest(),
        "threads": {key: value for key, value in os.environ.items()
                    if key.endswith("NUM_THREADS") or key == "VECLIB_MAXIMUM_THREADS"},
        "repeats": args.repeats, "diagnostics": args.diagnostics,
        "timing_rng": "OpenDP CSPRNG (uninstrumented)",
        "diagnostic_rng": "NumPy default_rng, seed = 2026 + dimension",
        "cases": [],
    }
    original_rng = eigenvector.get_np_csprng
    signal.signal(signal.SIGALRM, timeout_handler)
    for d in (4, 16, 64, 128, 256):
        # Public fixed dense rotation, identical in both checkouts.
        q, _ = np.linalg.qr(np.random.default_rng(d).standard_normal((d, d)))
        for spectrum in ("isotropic", "moderate", "strong"):
            values = {"isotropic": np.ones(d),
                      "moderate": np.linspace(0.0, 1.0, d),
                      "strong": np.r_[np.linspace(0.0, 0.01, d - 1), 1.0]}[spectrum]
            matrix = np.eye(d) if spectrum == "isotropic" else (q * values) @ q.T
            matrix = (matrix + matrix.T) / 2
            for concentration in (0.0, 10.0, 1000.0):
                case = {"d": d, "spectrum": spectrum, "concentration": concentration,
                        "matrix_sha256": hashlib.sha256(matrix.tobytes()).hexdigest()}
                signal.alarm(args.case_seconds)
                try:
                    eigenvector.get_np_csprng = original_rng
                    measurement = eigenvector.make_private_eigenvector(
                        _symmetric_matrix_domain(num_features=d),
                        _nuclear_norm_distance(), concentration)
                    for _ in range(3):
                        measurement(matrix)
                    latencies = []
                    for _ in range(args.repeats):
                        start = time.perf_counter_ns()
                        measurement(matrix)
                        latencies.append((time.perf_counter_ns() - start) / 1e6)
                    case.update(latencies_ms=latencies,
                                median_ms=float(np.median(latencies)),
                                q25_ms=float(np.quantile(latencies, 0.25)),
                                q75_ms=float(np.quantile(latencies, 0.75)))
                    rng = CountingRNG(2026 + d)
                    eigenvector.get_np_csprng = lambda: rng
                    diagnostic = eigenvector.make_private_eigenvector(
                        _symmetric_matrix_domain(num_features=d),
                        _nuclear_norm_distance(), concentration)
                    counts = []
                    for _ in range(args.diagnostics):
                        before = rng.proposals
                        vector = diagnostic(matrix)
                        if not np.isfinite(vector).all() or not np.isclose(np.linalg.norm(vector), 1):
                            raise ValueError("non-finite or non-unit output")
                        counts.append(rng.proposals - before)
                    case.update(proposals=counts,
                                proposals_mean=float(np.mean(counts)),
                                acceptance_rate=len(counts) / sum(counts))
                except TimeoutError as error:
                    case["timeout"] = str(error)
                finally:
                    signal.alarm(0)
                    eigenvector.get_np_csprng = original_rng
                result["cases"].append(case)
                args.output.parent.mkdir(parents=True, exist_ok=True)
                args.output.write_text(json.dumps(result, indent=2) + "\n")
                print(case["d"], spectrum, concentration,
                      case.get("median_ms", case.get("timeout")),
                      case.get("proposals_mean"), flush=True)


if __name__ == "__main__":
    main()
