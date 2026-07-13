from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Optional

import numpy as np

from .base import ClusterAlgorithm, FitContext
from ..common import FitResult, Timer, resolve_repo_path


class GoogleHSTClustering(ClusterAlgorithm):
    """Subprocess wrapper for google-research/hst_clustering.

    The HST code is written as an Apache Beam pipeline plus a dynamic program.
    Running it out-of-process keeps absl flags and Beam setup isolated from the
    benchmark harness.
    """

    name = "hst"

    def __init__(
        self,
        *,
        repo_root: Optional[Path] = None,
        layers: int = 10,
        num_buckets_beam: int = 100,
        keep_workdir: bool = False,
    ):
        self.repo_root = Path(repo_root).resolve() if repo_root else None
        self.layers = int(layers)
        self.num_buckets_beam = int(num_buckets_beam)
        self.keep_workdir = keep_workdir

    def _repo(self, ctx: FitContext) -> Path:
        if self.repo_root is not None:
            root = self.repo_root
        else:
            bench_root = Path(ctx.repo_root or ".").resolve()
            root = resolve_repo_path(bench_root, "google-research", "GOOGLE_RESEARCH_REPO")
        if not (root / "hst_clustering" / "run_clustering.py").exists():
            raise RuntimeError(
                f"Could not find google-research/hst_clustering under {root}. "
                "Run scripts/fetch_third_party.sh or set GOOGLE_RESEARCH_REPO."
            )
        return root

    @staticmethod
    def _write_raw_data(path: Path, x: np.ndarray) -> None:
        with path.open("w", encoding="utf-8") as f:
            for i, row in enumerate(x):
                coords = "\t".join(f"{float(v):.17g}" for v in row)
                f.write(f"{i}\t{coords}\n")

    def fit(self, x: np.ndarray, ctx: FitContext) -> FitResult:
        root = self._repo(ctx)
        min_value = float(np.min(ctx.bounds.lower))
        max_value = float(np.max(ctx.bounds.upper))
        epsilon = float(ctx.epsilon)
        if ctx.rho is not None:
            if ctx.rho <= 0:
                raise ValueError("rho must be positive")
            epsilon = float(ctx.rho + 2.0 * np.sqrt(ctx.rho * np.log(1.0 / float(ctx.delta))))
        tmp_ctx = tempfile.TemporaryDirectory(prefix="hst_bench_")
        tmp = Path(tmp_ctx.name)
        raw_path = tmp / "raw.tsv"
        output_dir = tmp / "out"
        output_dir.mkdir(parents=True, exist_ok=True)
        self._write_raw_data(raw_path, np.asarray(x, dtype=float))

        cmd = [
            sys.executable,
            "-m",
            "hst_clustering.run_clustering",
            f"--raw_data={raw_path}",
            f"--output_dir={output_dir}",
            f"--dimensions={x.shape[1]}",
            f"--min_value_entry={min_value}",
            f"--max_value_entry={max_value}",
            f"--k_params={ctx.k}",
            f"--layers={self.layers}",
            f"--epsilon={epsilon}",
            f"--delta={ctx.delta}",
            f"--seed={ctx.seed}",
            f"--num_buckets_beam={self.num_buckets_beam}",
            "--runner=DirectRunner",
        ]
        env = os.environ.copy()
        env["PYTHONPATH"] = str(root) + os.pathsep + env.get("PYTHONPATH", "")

        with Timer() as timer:
            proc = subprocess.run(
                cmd,
                cwd=str(root),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )
        if proc.returncode != 0:
            if not self.keep_workdir:
                tmp_ctx.cleanup()
            raise RuntimeError(
                "HST subprocess failed with exit code "
                f"{proc.returncode}\nSTDOUT:\n{proc.stdout[-4000:]}\nSTDERR:\n{proc.stderr[-4000:]}"
            )
        centers_path = output_dir / "centers.npy"
        if not centers_path.exists():
            if not self.keep_workdir:
                tmp_ctx.cleanup()
            raise RuntimeError(f"HST finished but did not write {centers_path}")
        centers = np.load(centers_path)
        extra = {
            "layers": self.layers,
            "num_buckets_beam": self.num_buckets_beam,
            "workdir": str(tmp) if self.keep_workdir else "",
            "stdout_tail": proc.stdout[-1000:],
            "stderr_tail": proc.stderr[-1000:],
        }
        if not self.keep_workdir:
            tmp_ctx.cleanup()
        return self._result(np.asarray(centers, dtype=float), timer.elapsed, extra)
