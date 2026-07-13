from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, Dict, Optional

import numpy as np

from ..common import Bounds, FitResult, Timer, as_2d_float


@dataclass
class FitContext:
    k: int
    rho: Optional[float]
    epsilon: float
    delta: float
    bounds: Bounds
    seed: int
    repo_root: Optional[str] = None


class ClusterAlgorithm(ABC):
    name: str

    @abstractmethod
    def fit(self, x: np.ndarray, ctx: FitContext) -> FitResult:
        raise NotImplementedError

    def _result(self, centers: np.ndarray, elapsed_s: float, extra: Optional[Dict[str, Any]] = None) -> FitResult:
        centers = as_2d_float(centers)
        return FitResult(self.name, centers, elapsed_s, extra or {})


class SklearnKMeans(ClusterAlgorithm):
    name = "sklearn"

    def __init__(self, *, n_init: int = 10, max_iter: int = 300):
        self.n_init = n_init
        self.max_iter = max_iter

    def fit(self, x: np.ndarray, ctx: FitContext) -> FitResult:
        from sklearn.cluster import KMeans

        with Timer() as timer:
            model = KMeans(
                n_clusters=ctx.k,
                n_init=self.n_init,
                max_iter=self.max_iter,
                random_state=ctx.seed,
            )
            model.fit(x)
        return self._result(model.cluster_centers_, timer.elapsed, {"inertia": float(model.inertia_)})


class SklearnKMedians(ClusterAlgorithm):
    """Simple nonprivate Lloyd-style k-medians baseline.

    This is intentionally lightweight: assign to Euclidean-nearest centers, then
    update each center to the coordinate-wise median. It is not a replacement for
    a dedicated k-medians solver, but it is a useful sanity check for PR #2664's
    KMedians output.
    """

    name = "sklearn_kmedians_like"

    def __init__(self, *, max_iter: int = 100, n_init: int = 5):
        self.max_iter = max_iter
        self.n_init = n_init

    def fit(self, x: np.ndarray, ctx: FitContext) -> FitResult:
        from sklearn.cluster import KMeans

        rng = np.random.default_rng(ctx.seed)
        best_centers = None
        best_loss = np.inf
        start = __import__("time").perf_counter()
        for init in range(self.n_init):
            if init == 0:
                centers = KMeans(n_clusters=ctx.k, n_init=1, random_state=ctx.seed).fit(x).cluster_centers_
            else:
                centers = x[rng.choice(x.shape[0], size=ctx.k, replace=False)].copy()
            labels = None
            for _ in range(self.max_iter):
                dsq = ((x[:, None, :] - centers[None, :, :]) ** 2).sum(axis=2)
                new_labels = np.argmin(dsq, axis=1)
                if labels is not None and np.array_equal(labels, new_labels):
                    break
                labels = new_labels
                for j in range(ctx.k):
                    mask = labels == j
                    if np.any(mask):
                        centers[j] = np.median(x[mask], axis=0)
            loss = float(np.sum(np.sqrt(np.min(((x[:, None, :] - centers[None, :, :]) ** 2).sum(axis=2), axis=1))))
            if loss < best_loss:
                best_loss = loss
                best_centers = centers.copy()
        elapsed = __import__("time").perf_counter() - start
        return self._result(best_centers, elapsed, {"kmedians_loss": best_loss})
