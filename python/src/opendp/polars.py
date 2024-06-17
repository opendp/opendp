"""The ``polars`` module provides supporting utilities for making DP releases with the Polars library."""

from __future__ import annotations
from dataclasses import dataclass
import os
from typing import Literal, Tuple
from opendp._lib import lib_path
from opendp.mod import Domain, Measurement, assert_features
from opendp.domains import series_domain, lazyframe_domain, option_domain, atom_domain
from opendp.measurements import make_private_lazyframe


class DPExpr(object):
    def __init__(self, expr):
        """Apply a differentially private plugin to a Polars expression."""
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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        return register_plugin_function(
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
        import polars as pl  # type: ignore[import-not-found]
        return self.expr.dp.sum(bounds, scale) / pl.len()

    def _discrete_quantile_score(self, alpha: float, candidates: list[float]):
        """Score the utility of each candidate for representing the true quantile.

        Candidates closer to the true quantile are assigned scores closer to zero.
        Lower scores are better.

        :param alpha: a value in $[0, 1]$. Choose 0.5 for median
        :param candidates: Set of possible quantiles to evaluate the utility of.
        """
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        return register_plugin_function(
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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        return register_plugin_function(
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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        return register_plugin_function(
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


try:
    from polars.api import register_expr_namespace  # type: ignore[import-not-found]
    register_expr_namespace("dp")(DPExpr)
except ImportError:
    pass


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


def _lazyframe_domain_from_schema(schema) -> Domain:
    """Builds the broadest possible LazyFrameDomain that matches a given LazyFrame schema."""
    return lazyframe_domain(
        [_series_domain_from_field(field) for field in schema.items()]
    )


def _series_domain_from_field(field) -> Domain:
    """Builds the broadest possible SeriesDomain that matches a given field."""
    import polars as pl
    name, dtype = field
    T = {
        pl.UInt32: "u32",
        pl.UInt64: "u64",
        pl.Int8: "i8",
        pl.Int16: "i16",
        pl.Int32: "i32",
        pl.Int64: "i64",
        pl.Float32: "f32",
        pl.Float64: "f64",
        pl.Boolean: "bool",
        pl.String: "String",
    }.get(dtype)

    if T is None:
        raise ValueError(f"unrecognized dtype: {dtype}")

    element_domain = option_domain(atom_domain(T=T, nullable=T in {"f32", "f64"}))
    return series_domain(name, element_domain)


_LAZY_EXECUTION_METHODS = {
    "collect",
    "collect_async",
    "describe",
    "sink_parquet",
    "sink_ipc",
    "sink_csv",
    "sink_ndjson",
    "fetch",
}


# In our DataFrame APIs, we only need to make a few small adjustments to Polars LazyFrames.
# Therefore if Polars is installed, LazyFrameQuery subclasses LazyFrame.
#
# # Compatibility with MyPy
# If Polars is not installed, then a dummy LazyFrameQuery class is declared.
# This way other modules can import and include LazyFrameQuery in static type annotations.
# MyPy type-checks based on the first definition of a class,
# so the code is written such that the first definition of the class is the real thing.
#
# # Compatibility with Pylance
# If declared in branching code, Pylance may only provide code analysis based on the dummy class.
# Therefore preference is given to the real class via a try-except block.
#
# # Compatibility with Sphinx
# Even when Polars is installed, Sphinx fails to find the reference to the Polars LazyFrame class.
# I think this is because Polars is not a direct dependency.
# Therefore the following code uses OPENDP_HEADLESS to get Sphinx to document the dummy class instead.
# This may also be preferred behavior:
#     only the changes we make to the Polars LazyFrame API are documented, not the entire LazyFrame API.
try:
    if os.environ.get("OPENDP_HEADLESS"):
        raise ImportError(
            "Sphinx always fails to find a reference to LazyFrame. Falling back to dummy class."
        )
    from polars.lazyframe.frame import LazyFrame as _LazyFrame  # type: ignore[import-not-found]
    from polars.lazyframe.group_by import LazyGroupBy as _LazyGroupBy  # type: ignore[import-not-found]

    class LazyFrameQuery(_LazyFrame):
        """LazyFrameQuery mimics a Polars LazyFrame, but makes a few additions and changes as documented below."""

        def __init__(self, lf_plan: _LazyFrame | _LazyGroupBy, query):
            self._lf_plan = lf_plan
            self._query = query
            # do not initialize super() because inheritance is only used to mimic the API surface

        def __getattribute__(self, name):
            # Re-route all possible attribute access to self._lf_plan.
            # __getattribute__ is necessary because __getattr__ cannot intercept calls to inherited methods

            # We keep the query plan void of data anyways,
            # so running the computation doesn't affect privacy.
            # This doesn't have to cover all possible APIs that may execute the query,
            #    but it does give a simple sanity check for the obvious cases.
            if name in _LAZY_EXECUTION_METHODS:
                raise ValueError("You must call `.release()` before executing a query.")

            lf_plan = object.__getattribute__(self, "_lf_plan")
            query = object.__getattribute__(self, "_query")
            attr = getattr(lf_plan, name, None)

            # If not a valid attribute on self._lf_plan, then don't re-route
            if attr is None:
                return object.__getattribute__(self, name)

            # any callable attributes (like .with_columns or .select) will now also wrap their outputs in a LazyFrameQuery
            if callable(attr):

                def wrap(*args, **kwargs):
                    out = attr(*args, **kwargs)

                    # re-wrap any lazy outputs to keep the conveniences afforded by this class
                    if isinstance(out, (_LazyGroupBy, _LazyFrame)):
                        return LazyFrameQuery(out, query)

                    return out

                return wrap
            return attr

        def resolve(self) -> Measurement:
            """Resolve the query into a measurement."""
            from opendp.context import PartialChain

            # access attributes of self without getting intercepted by Self.__getattribute__
            lf_plan = object.__getattribute__(self, "_lf_plan")
            query = object.__getattribute__(self, "_query")
            input_domain, input_metric = query._chain

            # create an updated query and then resolve it into a measurement
            return query.new_with(
                chain=PartialChain(
                    lambda s: make_private_lazyframe(
                        input_domain=input_domain,
                        input_metric=input_metric,
                        output_measure=query._output_measure,
                        lazyframe=lf_plan,
                        global_scale=s,
                    )
                )
            ).resolve()

        def release(self) -> OnceFrame:
            """Release the query. The query must be part of a context."""
            query = object.__getattribute__(self, "_query")
            resolve = object.__getattribute__(self, "resolve")
            return query._context(resolve())  # type: ignore[misc]

except ImportError:
    ERR_MSG = "LazyFrameQuery depends on Polars: `pip install 'opendp[polars]'`."

    class LazyFrameQuery(object):  # type: ignore[no-redef]
        """LazyFrameQuery mimics a Polars LazyFrame, but makes a few additions and changes as documented below."""

        def resolve(self) -> Measurement:
            """Resolve the query into a measurement."""
            raise ImportError(ERR_MSG)

        def release(self) -> OnceFrame:
            """Release the query. The query must be part of a context."""
            raise ImportError(ERR_MSG)


@dataclass
class Margin(object):
    public_info: Literal["keys"] | Literal["lengths"] | None = None
    """Identifies properties of grouped data that are considered public information.
    
    * "keys" designates that keys are not protected
    * "lengths" designates that both keys and partition lengths are not protected
    """

    max_partition_length: int | None = None
    """An upper bound on the number of records in any one partition.

    If you don't know how many records are in the data, you can specify a very loose upper bound.

    This is used to resolve issues raised in [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    """

    max_num_partitions: int | None = None
    """An upper bound on the number of distinct partitions."""

    max_partition_contributions: int | None = None
    """The greatest number of records an individual may contribute to any one partition.
    
    This can significantly reduce the sensitivity of grouped queries under zero-Concentrated DP.
    """

    max_influenced_partitions: int | None = None
    """The greatest number of partitions any one individual can contribute to."""
