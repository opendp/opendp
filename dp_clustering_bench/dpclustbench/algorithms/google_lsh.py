from __future__ import annotations

import sys
from pathlib import Path
from typing import Optional

import numpy as np

from .base import ClusterAlgorithm, FitContext
from ..common import FitResult, Timer, resolve_repo_path


class GoogleLSHCoresetKMeans(ClusterAlgorithm):
    """Wrapper for google/differential-privacy learning/clustering.

    Expected layout after scripts/fetch_third_party.sh:
      third_party/differential-privacy/learning/clustering/...

    You can override with GOOGLE_DP_REPO=/path/to/differential-privacy.
    """

    name = "google_lsh"

    def __init__(self, *, repo_root: Optional[Path] = None, max_depth: Optional[int] = None):
        self.repo_root = Path(repo_root).resolve() if repo_root else None
        self.max_depth = max_depth

    def _add_import_path(self, ctx: FitContext) -> Path:
        root = self.repo_root
        if root is None:
            bench_root = Path(ctx.repo_root or ".").resolve()
            root = resolve_repo_path(bench_root, "differential-privacy", "GOOGLE_DP_REPO")
        learning = root / "learning"
        if not (learning / "clustering").exists():
            raise RuntimeError(
                f"Could not find Google DP learning/clustering under {learning}. "
                "Run scripts/fetch_third_party.sh or set GOOGLE_DP_REPO."
            )
        if str(learning) not in sys.path:
            sys.path.insert(0, str(learning))
        python_dir = root / "python"
        if python_dir.exists() and str(python_dir) not in sys.path:
            sys.path.insert(0, str(python_dir))
        accounting_dir = python_dir / "dp_accounting"
        if accounting_dir.exists() and str(accounting_dir) not in sys.path:
            sys.path.insert(0, str(accounting_dir))
        auditorium_dir = python_dir / "dp_auditorium"
        if auditorium_dir.exists() and str(auditorium_dir) not in sys.path:
            sys.path.insert(0, str(auditorium_dir))
        return root

    def fit(self, x: np.ndarray, ctx: FitContext) -> FitResult:
        self._add_import_path(ctx)
        from clustering import clustering_algorithm
        from clustering import clustering_params

        data = clustering_params.Data(np.asarray(x, dtype=float), float(ctx.bounds.radius))
        epsilon = float(ctx.epsilon)
        if ctx.rho is not None:
            if ctx.rho <= 0:
                raise ValueError("rho must be positive")
            epsilon = float(ctx.rho + 2.0 * np.sqrt(ctx.rho * np.log(1.0 / float(ctx.delta))))
        privacy = clustering_params.DifferentialPrivacyParam(epsilon, float(ctx.delta))
        tree_param = None
        if self.max_depth is not None:
            # Conservative defaults. Google's default_tree_param is often better;
            # this override is mostly for depth sweeps.
            tree_param = clustering_params.TreeParam(
                min_num_points_in_branching_node=1,
                min_num_points_in_node=1,
                max_depth=int(self.max_depth),
            )
        with Timer() as timer:
            result = clustering_algorithm.private_lsh_clustering(
                int(ctx.k), data, privacy, tree_param=tree_param
            )
        return self._result(
            np.asarray(result.centers, dtype=float),
            timer.elapsed,
            {"radius": float(ctx.bounds.radius), "max_depth": self.max_depth},
        )
