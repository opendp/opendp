from __future__ import annotations
from typing import Literal, Tuple
from opendp._lib import import_optional_dependency, lib_path
from opendp.mod import assert_features


pl = import_optional_dependency("polars", raise_error=False)


if pl is None:
    def _register_expr_namespace(_name):
        return lambda c: c
else:
    _register_expr_namespace = pl.api.register_expr_namespace


@_register_expr_namespace("dp")
class DPExpr(object):
    def __init__(self, expr):
        self.expr = expr

    def noise(
        self,
        scale: float | None = None,
        distribution: Literal["Laplace"] | Literal["Gaussian"] | None = None,
    ):
        """Add noise to the expression.

        If scale is None it is filled by `global_scale` in :py:func:`opendp.measurements.make_private_lazyframe`.
        If distribution is None, then the noise distribution will be chosen for you:

        * Pure-DP: Laplace noise, where `scale` == standard_deviation / sqrt(2)
        * zCDP: Gaussian noise, where `scale` == standard_devation

        :param scale: Scale parameter for the noise distribution.
        :param distribution: Either Laplace, Gaussian or None.
        """
        return pl.plugins.register_plugin_function(
            plugin_path=lib_path,
            function_name="noise",
            kwargs={"scale": scale, "distribution": distribution},
            args=self.expr,
            is_elementwise=True,
        )

    def laplace(self, scale: float | None = None):
        """Add Laplace noise to the expression.

        If scale is None it is filled by `global_scale` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
        """
        return self.noise(scale=scale, distribution="Laplace")

    def gaussian(self, scale: float | None = None):
        """Add Gaussian noise to the expression.

        If scale is None it is filled by `global_scale` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: Noise scale parameter for the Gaussian distribution. `scale` == standard_deviation.
        """
        return self.noise(scale=scale, distribution="Gaussian")

    def sum(self, bounds: Tuple[float, float], scale: float | None = None):
        """Compute the differentially private sum.

        If scale is None it is filled by `global_scale` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param bounds: The bounds of the input data.
        :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
        """
        return self.expr.clip(*bounds).sum().dp.noise(scale)

    def mean(self, bounds: Tuple[float, float], scale: float | None = None):
        """Compute the differentially private mean.

        The amount of noise to be added to the sum is determined by the scale.
        If scale is None it is filled by `global_scale` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param bounds: The bounds of the input data.
        :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
        """
        return self.expr.dp.sum(bounds, scale) / pl.len()

    def _discrete_quantile_score(self, alpha: float, candidates: list[float]):
        """Score the utility of each candidate for representing the true quantile.

        Candidates closer to the true quantile are assigned scores closer to zero.
        Lower scores are better.

        :param alpha: a value in $[0, 1]$. Choose 0.5 for median
        :param candidates: Set of possible quantiles to evaluate the utility of.
        """
        return pl.plugins.register_plugin_function(
            plugin_path=lib_path,
            function_name="discrete_quantile_score",
            kwargs={"alpha": alpha, "candidates": candidates},
            args=self.expr,
            returns_scalar=True,
        )

    def _report_noisy_max_gumbel(
        self, optimize: Literal["min"] | Literal["max"], scale: float | None = None
    ):
        """Report the argmax or argmin after adding Gumbel noise.

        The scale calibrates the level of entropy when selecting an index.

        :param optimize: Distinguish between argmax and argmin.
        :param scale: Noise scale parameter for the Gumbel distribution.
        """
        return pl.plugins.register_plugin_function(
            plugin_path=lib_path,
            function_name="report_noisy_max_gumbel",
            kwargs={"optimize": optimize, "scale": scale},
            args=self.expr,
            is_elementwise=True,
        )

    def _index_candidates(self, candidates: list[float]):
        """Index into a candidate set.

        Typically used after `rnm_gumbel` to map selected indices to candidates.

        :param candidates: The values that each selected index corresponds to.
        """
        return pl.plugins.register_plugin_function(
            plugin_path=lib_path,
            function_name="index_candidates",
            kwargs={"candidates": candidates},
            args=self.expr,
            is_elementwise=True,
        )

    def quantile(
        self, alpha: float, candidates: list[float], scale: float | None = None
    ):
        """Compute a differentially private quantile.

        The scale calibrates the level of entropy when selecting a candidate.

        :param alpha: a value in $[0, 1]$. Choose 0.5 for median
        :param candidates: Potential quantiles to select from.
        :param scale: How much noise to add to the scores of candidate.
        """
        dq_score = self.expr.dp._discrete_quantile_score(alpha, candidates)
        noisy_idx = dq_score.dp._report_noisy_max_gumbel("min", scale)
        return noisy_idx.dp._index_candidates(candidates)

    def median(self, candidates: list[float], scale: float | None = None):
        """Compute a differentially private median.

        The scale calibrates the level of entropy when selecting a candidate.

        :param candidates: Potential quantiles to select from.
        :param scale: How much noise to add to the scores of candidate.
        """
        return self.expr.dp.quantile(0.5, candidates, scale)


class OnceFrame(object):
    def __init__(self, queryable):
        """OnceFrame is a Polars LazyFrame that may only be collected into a DataFrame once.

        The APIs on this class mimic those that can be found in Polars.

        Differentially private guarantees on a given LazyFrame require the LazyFrame to be evaluated at most once.
        The purpose of this class is to protect against repeatedly evaluating the LazyFrame.
        """
        self.queryable = queryable

    def collect(self):
        """Collects a DataFrame from a OnceFrame, exhausting the OnceFrame."""
        from opendp._data import onceframe_collect

        return onceframe_collect(self.queryable)

    def lazy(self):
        """Extracts a LazyFrame from a OnceFrame,
        circumventing protections against multiple evaluations.

        Each collection consumes the entire allocated privacy budget.
        To remain DP at the advertised privacy level, only collect the LazyFrame once.

        **Features:**

        * `honest-but-curious` - LazyFrames can be collected an unlimited number of times.
        """
        from opendp._data import onceframe_lazy

        assert_features("honest-but-curious")
        return onceframe_lazy(self.queryable)
