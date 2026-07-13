from __future__ import annotations

from typing import Optional, Sequence
import numpy as np

from .base import ClusterAlgorithm, FitContext
from ..common import FitResult, Timer


class OpenDPKMeans(ClusterAlgorithm):
    name = "opendp_kmeans"

    def __init__(
        self,
        *,
        scale: float = 1.0,
        max_depth: Optional[int] = 7,
        level_allocation: Optional[Sequence[float]] = None,
        random_state: Optional[int] = None,
        rho: Optional[float] = None,
    ):
        self.scale = scale
        self.max_depth = max_depth
        self.level_allocation = level_allocation
        self.random_state = random_state
        self.rho = rho

    def fit(self, x: np.ndarray, ctx: FitContext) -> FitResult:
        import opendp.prelude as dp

        dp.enable_features("contrib")
        output_measure = dp.zero_concentrated_divergence()
        scale = self.scale
        rho = self.rho if self.rho is not None else ctx.rho
        if rho is not None:
            if rho <= 0:
                raise ValueError("rho must be positive")
            scale = float(np.sqrt(1.0 / (2.0 * rho)))
        kwargs = dict(
            n_features=x.shape[1],
            n_clusters=ctx.k,
            scale=scale,
            lower=ctx.bounds.lower,
            upper=ctx.bounds.upper,
            random_state=self.random_state if self.random_state is not None else ctx.seed,
            output_measure=output_measure,
        )
        if self.level_allocation is not None:
            kwargs["level_allocation"] = list(self.level_allocation)
        elif self.max_depth is not None:
            kwargs["max_depth"] = int(self.max_depth)
        with Timer() as timer:
            model = dp.sklearn.cluster.KMeans(**kwargs)
            model.fit(x)
        return self._result(np.asarray(model.cluster_centers_, dtype=float), timer.elapsed, kwargs)


class OpenDPKMedians(OpenDPKMeans):
    name = "opendp_kmedians"

    def fit(self, x: np.ndarray, ctx: FitContext) -> FitResult:
        import opendp.prelude as dp

        dp.enable_features("contrib")
        output_measure = dp.zero_concentrated_divergence()
        scale = self.scale
        rho = self.rho if self.rho is not None else ctx.rho
        if rho is not None:
            if rho <= 0:
                raise ValueError("rho must be positive")
            scale = float(np.sqrt(1.0 / (2.0 * rho)))
        kwargs = dict(
            n_features=x.shape[1],
            n_clusters=ctx.k,
            scale=scale,
            lower=ctx.bounds.lower,
            upper=ctx.bounds.upper,
            random_state=self.random_state if self.random_state is not None else ctx.seed,
            output_measure=output_measure,
        )
        if self.level_allocation is not None:
            kwargs["level_allocation"] = list(self.level_allocation)
        elif self.max_depth is not None:
            kwargs["max_depth"] = int(self.max_depth)
        with Timer() as timer:
            model = dp.sklearn.cluster.KMedians(**kwargs)
            model.fit(x)
        return self._result(np.asarray(model.cluster_centers_, dtype=float), timer.elapsed, kwargs)
