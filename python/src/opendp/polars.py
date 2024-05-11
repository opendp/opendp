from __future__ import annotations
from dataclasses import dataclass
from typing import Literal, Tuple
from opendp._lib import import_optional_dependency, lib_path
from opendp.mod import assert_features
from opendp.domains import series_domain, lazyframe_domain, option_domain, atom_domain
from opendp.measurements import make_private_lazyframe


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


def lazyframe_domain_from_schema(schema):
    return lazyframe_domain(
        [series_domain_from_field(field) for field in schema.items()]
    )


def series_domain_from_field(field):
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

if pl is not None:
    from polars import LazyFrame
    from polars.lazyframe.group_by import LazyGroupBy

    class LazyQuery(LazyFrame):
        """This class mimics the LazyFrame API, but adds additional methods `release` and `resolve`."""
        def __init__(self, lazy: LazyFrame | LazyGroupBy, query):
            self._lazy = lazy
            self._query = query

        def __getattribute__(self, name):

            lf = object.__getattribute__(self, "_lazy")
            query = object.__getattribute__(self, "_query")
            attr = getattr(lf, name, None)

            if attr is None:
                return object.__getattribute__(self, name)

            if callable(attr):

                def wrap(*args, **kwargs):
                    out = attr(*args, **kwargs)
                    if isinstance(out, (LazyGroupBy, LazyFrame)):
                        return LazyQuery(out, query)
                    return out

                return wrap

        def resolve(self):
            from opendp.context import PartialChain

            lf = object.__getattribute__(self, "_lazy")
            query = object.__getattribute__(self, "_query")
            input_domain, input_metric = query._chain
            query._chain = PartialChain(
                lambda s: make_private_lazyframe(
                    input_domain=input_domain,
                    input_metric=input_metric,
                    output_measure=query._output_measure,
                    lazyframe=lf,
                    global_scale=s,
                )
            )
            return query.resolve()

        def release(self):
            query = object.__getattribute__(self, "_query")
            resolve = object.__getattribute__(self, "resolve")
            return query._context(resolve())  # type: ignore[misc]

else:
    class LazyQuery(object): # type: ignore[no-redef]
        pass

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

    This is used to resolve issues raised in "Widespread Underestimation of Sensitivity in Differentially Private Libraries and How to Fix It" [CSVW22]
    """

    max_num_partitions: int | None = None
    """An upper bound on the number of distinct partitions."""

    max_partition_contributions: int | None = None
    """The greatest number of records an individual may contribute to any one partition.
    
    This can significantly reduce the sensitivity of grouped queries under zero-Concentrated DP.
    """

    max_influenced_partitions: int | None = None
    """The greatest number of partitions any one individual can contribute to."""

    def with_public_keys(self) -> "Margin":
        self.public_info = "keys"
        return self

    def with_public_lengths(self) -> "Margin":
        self.public_info = "lengths"
        return self
