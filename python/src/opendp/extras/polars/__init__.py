"""
This module requires extra installs: ``pip install 'opendp[polars]'``

The ``opendp.extras.polars`` module adds differential privacy to the
`Polars DataFrame library <https://docs.pola.rs>`_.

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.polars``.
"""

from __future__ import annotations
from dataclasses import dataclass
import os
from typing import Any, Iterable, Literal, Sequence
from opendp._lib import lib_path
from opendp.mod import (
    Domain,
    Measurement,
    OpenDPException,
    binary_search,
    binary_search_chain,
)
from opendp.domains import (
    series_domain,
    lazyframe_domain,
    option_domain,
    atom_domain,
    categorical_domain,
    datetime_domain,
)
from opendp.measurements import make_private_lazyframe


class DPExpr(object):
    """
    If both ``opendp`` and ``polars`` have been imported,
    the methods of :py:class:`DPExpr` are registered under the ``dp`` namespace in
    `Polars expressions <https://docs.pola.rs/py-polars/html/reference/expressions/index.html>`_.
    An expression can be used as a plan in :py:func:`opendp.measurements.make_private_lazyframe`;
    See the full example there for more information.

    In addition to the DP-specific methods here, many Polars ``Expr`` methods are also supported,
    and are documented in the :ref:`API User Guide <expression-index>`.    

    This class is typically not used directly by users:
    Instead its methods are registered under the ``dp`` namespace of Polars expressions.

    >>> import polars as pl
    >>> pl.len().dp
    <opendp.extras.polars.DPExpr object at ...>
    """

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

        * Pure-DP: Laplace noise, where ``scale == standard_deviation / sqrt(2)``
        * zCDP: Gaussian noise, where ``scale == standard_devation``

        :param scale: Scale parameter for the noise distribution.
        :param distribution: Either Laplace, Gaussian or None.

        :example:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"A": list(range(100))}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ... )
        >>> query = context.query().select(dp.len())
        >>> query.release().collect()
        shape: (1, 1)
        ┌─────┐
        │ len │
        │ --- │
        │ u32 │
        ╞═════╡
        │ ... │
        └─────┘
        """
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        from polars import lit  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=os.environ.get("OPENDP_POLARS_LIB_PATH", lib_path),
            function_name="noise",
            args=(self.expr, lit(distribution), scale),
            is_elementwise=True,
        )

    def laplace(self, scale: float | None = None):
        """Add Laplace noise to the expression.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: Noise scale parameter for the Laplace distribution. ``scale == standard_deviation / sqrt(2)``

        :example:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"A": list(range(100))}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ... )
        >>> query = context.query().select(pl.len().dp.laplace())
        >>> query.release().collect()
        shape: (1, 1)
        ┌─────┐
        │ len │
        │ --- │
        │ u32 │
        ╞═════╡
        │ ... │
        └─────┘
        """
        return self.noise(scale=scale, distribution="Laplace")

    def gaussian(self, scale: float | None = None):
        """Add Gaussian noise to the expression.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: Noise scale parameter for the Gaussian distribution. ``scale == standard_deviation``

        :example:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"A": list(range(100))}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(rho=0.5),
        ...     split_evenly_over=1,
        ... )
        >>> query = context.query().select(pl.len().dp.gaussian())
        >>> query.release().collect()
        shape: (1, 1)
        ┌─────┐
        │ len │
        │ --- │
        │ u32 │
        ╞═════╡
        │ ... │
        └─────┘
        """
        return self.noise(scale=scale, distribution="Gaussian")

    def len(self, scale: float | None = None):
        """Compute a differentially private estimate of the number of elements in `self`, including null values.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: parameter for the noise distribution.

        :example:

        This function is a shortcut for the exact Polars ``len`` and then noise addition:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"visits": [1, 2, None]}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ... )
        >>> query = context.query().select(pl.col("visits").dp.len())
        >>> query.release().collect()
        shape: (1, 1)
        ┌────────┐
        │ visits │
        │ ---    │
        │ u32    │
        ╞════════╡
        │ ...    │
        └────────┘

        Output is noise added to three.

        It can differ from frame length (``.select(dp.len())``) if the expression uses transformations that change the number of rows,
        like filtering.
        """
        return self.expr.len().dp.noise(scale)

    def count(self, scale: float | None = None):
        """Compute a differentially private estimate of the number of elements in `self`, not including null values.

        This function is a shortcut for the exact Polars ``count`` and then noise addition.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: parameter for the noise distribution.

        :example:

        Count the number of records with known (non-null) visits:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"visits": [1, 2, None]}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ... )
        >>> query = context.query().select(pl.col("visits").dp.count())
        >>> query.release().collect()
        shape: (1, 1)
        ┌────────┐
        │ visits │
        │ ---    │
        │ u32    │
        ╞════════╡
        │ ...    │
        └────────┘

        Output is noise added to three.
        """
        return self.expr.count().dp.noise(scale)

    def null_count(self, scale: float | None = None):
        """Compute a differentially private estimate of the number of null elements in `self`.

        This function is a shortcut for the exact Polars ``null_count`` and then noise addition.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: parameter for the noise distribution.

        :example:

        Count the number of records with unknown (null) visits:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"visits": [1, 2, None]}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ... )
        >>> query = context.query().select(pl.col("visits").dp.null_count())
        >>> query.release().collect()
        shape: (1, 1)
        ┌────────┐
        │ visits │
        │ ---    │
        │ u32    │
        ╞════════╡
        │ ...    │
        └────────┘

        Output is noise added to one.

        Note that if you want to count the number of null *and* non-null records,
        consider combining the queries by constructing a boolean nullity column to group on,
        grouping by this column, and then using ``dp.len()``.
        """
        return self.expr.null_count().dp.noise(scale)

    def n_unique(self, scale: float | None = None):
        """Compute a differentially private estimate of the number of unique elements in `self`.

        This function is a shortcut for the exact Polars ``n_unique`` and then noise addition.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: parameter for the noise distribution.

        :example:

        Count the number of unique addresses:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"visits": [1, 2, None]}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ... )
        >>> query = context.query().select(pl.col("visits").dp.n_unique())
        >>> query.release().collect()
        shape: (1, 1)
        ┌────────┐
        │ visits │
        │ ---    │
        │ u32    │
        ╞════════╡
        │ ...    │
        └────────┘

        Output is noise added to three.
        """
        return self.expr.n_unique().dp.noise(scale)

    def sum(self, bounds: tuple[float, float], scale: float | None = None):
        """Compute the differentially private sum.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param bounds: clip the input data to these lower and upper bounds
        :param scale: parameter for the noise distribution

        :example:

        This function is a shortcut which actually implies several operations:

        * Clipping the values
        * Summing them
        * Applying noise to the sum

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"visits": [1, 2, None]}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ...     margins={(): dp.polars.Margin(max_partition_length=5)}
        ... )
        >>> query = context.query().select(pl.col("visits").fill_null(0).dp.sum((0, 1)))
        >>> query.release().collect()
        shape: (1, 1)
        ┌────────┐
        │ visits │
        │ ---    │
        │ i64    │
        ╞════════╡
        │ ...    │
        └────────┘

        Output is noise added to two due to each value being clipped to (0, 1).
        """
        return self.expr.clip(*bounds).sum().dp.noise(scale)

    def mean(
        self,
        bounds: tuple[float, float],
        scale: tuple[float | None, float | None] = (None, None),
    ):
        """Compute the differentially private mean.

        The amount of noise to be added to the sum is determined by the scale.
        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param bounds: clip the input data to these lower and upper bounds
        :param scale: parameters for the noise distributions of the numerator and denominator

        :example:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"visits": [1, 2, None]}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ...     margins={(): dp.polars.Margin(max_partition_length=5)}
        ... )
        >>> query = context.query().select(pl.col("visits").fill_null(0).dp.mean((0, 1)))
        >>> with pl.Config(float_precision=0): # just to prevent doctest from failing
        ...     query.release().collect()
        shape: (1, 1)
        ┌────────┐
        │ visits │
        │ ---    │
        │ f64    │
        ╞════════╡
        │ ...... │
        └────────┘

        Privately estimates the numerator and denominator separately, and then returns their ratio.
        """
        numer, denom = scale
        return self.sum(bounds, numer) / self.len(denom)

    def _discrete_quantile_score(self, alpha: float, candidates: list[float]):
        """Score the utility of each candidate for representing the true quantile.

        Candidates closer to the true quantile are assigned scores closer to zero.
        Lower scores are better.

        :param alpha: a value in [0, 1]. Choose 0.5 for median
        :param candidates: Set of possible quantiles to evaluate the utility of.
        """
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        from polars import Series  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=os.environ.get("OPENDP_POLARS_LIB_PATH", lib_path),
            function_name="discrete_quantile_score",
            args=[self.expr, alpha, Series(candidates)],
            returns_scalar=True,
        )

    def _report_noisy_max(
        self, optimize: Literal["min", "max"], scale: float | None = None
    ):
        """Report the argmax or argmin after adding Gumbel noise.

        The scale calibrates the level of entropy when selecting an index.
        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param optimize: Distinguish between argmax and argmin.
        :param scale: Noise scale parameter for the Gumbel distribution.
        """
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        from polars import lit  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=os.environ.get("OPENDP_POLARS_LIB_PATH", lib_path),
            function_name="report_noisy_max",
            args=[self.expr, lit(optimize), scale],
            is_elementwise=True,
        )

    def _index_candidates(self, candidates: list[float]):
        """Index into a candidate set.

        Typically used after :py:func:`_report_noisy_max` to map selected indices to candidates.

        :param candidates: The values that each selected index corresponds to.
        """
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        from polars import Series  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=os.environ.get("OPENDP_POLARS_LIB_PATH", lib_path),
            function_name="index_candidates",
            args=[self.expr, Series(candidates)],
            is_elementwise=True,
        )

    def quantile(
        self, alpha: float, candidates: list[float], scale: float | None = None
    ):
        """Compute a differentially private quantile.

        The scale calibrates the level of entropy when selecting a candidate.

        :param alpha: a value in [0, 1]. Choose 0.5 for median.
        :param candidates: Potential quantiles to select from.
        :param scale: How much noise to add to the scores of candidate.

        :example:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"age": list(range(100))}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ...     margins={(): dp.polars.Margin(max_partition_length=100)}
        ... )
        >>> candidates = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90]
        >>> query = context.query().select(pl.col("age").fill_null(0).dp.quantile(0.25, candidates))
        >>> query.release().collect()
        shape: (1, 1)
        ┌─────┐
        │ age │
        │ --- │
        │ i64 │
        ╞═════╡
        │ ... │
        └─────┘

        Output will be one of the candidates,
        with greater likelihood of being selected the closer the candidate is to the first quartile.
        """
        dq_score = self.expr.dp._discrete_quantile_score(alpha, candidates)
        noisy_idx = dq_score.dp._report_noisy_max("min", scale)
        return noisy_idx.dp._index_candidates(candidates)

    def median(self, candidates: list[float], scale: float | None = None):
        """Compute a differentially private median.

        The scale calibrates the level of entropy when selecting a candidate.
        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param candidates: Potential quantiles to select from.
        :param scale: How much noise to add to the scores of candidate.

        :example:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"age": list(range(100))}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ...     margins={(): dp.polars.Margin(max_partition_length=100)}
        ... )
        >>> candidates = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90]
        >>> query = context.query().select(pl.col("age").fill_null(0).dp.quantile(0.5, candidates))
        >>> query.release().collect()
        shape: (1, 1)
        ┌─────┐
        │ age │
        │ --- │
        │ i64 │
        ╞═════╡
        │ ... │
        └─────┘

        Output will be one of the candidates,
        with greater likelihood of being selected the closer the candidate is to the median.
        """
        return self.expr.dp.quantile(0.5, candidates, scale)


try:
    from polars.api import register_expr_namespace  # type: ignore[import-not-found]

    register_expr_namespace("dp")(DPExpr)
except ImportError:  # pragma: no cover
    pass


def dp_len(scale: float | None = None):
    """Compute a differentially private estimate of the number of rows.

    If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

    :param scale: parameter for the noise distribution.

    :example:

    This function is a shortcut for the exact Polars ``len`` and then noise addition:

    >>> import polars as pl
    >>> context = dp.Context.compositor(
    ...     data=pl.LazyFrame({"A": list(range(100))}),
    ...     privacy_unit=dp.unit_of(contributions=1),
    ...     privacy_loss=dp.loss_of(epsilon=1.),
    ...     split_evenly_over=1,
    ... )
    >>> query = context.query().select(dp.len())
    >>> query.release().collect()
    shape: (1, 1)
    ┌─────┐
    │ len │
    │ --- │
    │ u32 │
    ╞═════╡
    │ ... │
    └─────┘
    """
    from polars.functions import len  # type: ignore[import-not-found]

    return DPExpr(len()).noise(scale=scale)


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
        """Extracts a ``LazyFrame`` from a ``OnceFrame``,
        circumventing protections against multiple evaluations.

        Each collection consumes the entire allocated privacy budget.
        To remain DP at the advertised privacy level, only collect the ``LazyFrame`` once.

        Requires "honest-but-curious" because the privacy guarantees only apply if:
        1. The LazyFrame (compute plan) is only ever executed once.
        2. The analyst does not observe ordering of rows in the output. 
        
        To ensure that row ordering is not observed:
        1. Do not extend the compute plan with order-sensitive computations.
        2. Shuffle the output once collected `(in Polars sample all, with shuffle enabled) <https://docs.pola.rs/api/python/stable/reference/dataframe/api/polars.DataFrame.sample.html>`_.
        """
        from opendp._data import onceframe_lazy

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
    if dtype == pl.Categorical:
        return series_domain(name, option_domain(categorical_domain()))
    if dtype == pl.Datetime:
        dt_domain = datetime_domain(dtype.time_unit, dtype.time_zone)
        return series_domain(name, option_domain(dt_domain))

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
        pl.Time: "NaiveTime",
        pl.Date: "NaiveDate",
    }.get(dtype)

    if T is None:
        raise ValueError(f"unrecognized dtype: {dtype}")  # pragma: no cover

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
    from polars.dataframe.frame import DataFrame as _DataFrame  # type: ignore[import-not-found]
    from polars.expr.expr import Expr as _Expr  # type: ignore[import-not-found]
    from polars.lazyframe.group_by import LazyGroupBy as _LazyGroupBy  # type: ignore[import-not-found]
    from polars._typing import IntoExpr, IntoExprColumn, JoinStrategy, JoinValidation  # type: ignore[import-not-found]
    import numpy as np  # type: ignore[import-not-found]

    class LazyFrameQuery(_LazyFrame):
        """
        A ``LazyFrameQuery`` may be returned by :py:func:`opendp.context.Context.query`.
        It mimics a `Polars LazyFrame <https://docs.pola.rs/api/python/stable/reference/lazyframe/index.html>`_,
        but makes a few additions and changes as documented below."""

        # Keep this docstring in sync with the docstring below for the dummy class.

        def __init__(self, lf_plan: _LazyFrame, query):
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
                    if isinstance(out, _LazyFrame):
                        return LazyFrameQuery(out, query)

                    if isinstance(out, _LazyGroupBy):
                        return LazyGroupByQuery(out, query)

                    return out

                return wrap
            return attr

        # These definitions are primarily for mypy:
        # Without them, a "# type: ignore[union-attr]" is needed on every line where these methods are used.
        # The docstrings are not seen by Sphinx, but aren't doing any harm either.

        def sort(  # type: ignore[empty-body]
            self,
            by: IntoExpr | Iterable[IntoExpr],
            *more_by: IntoExpr,
            descending: bool | Sequence[bool] = False,
            nulls_last: bool | Sequence[bool] = False,
            maintain_order: bool = False,
            multithreaded: bool = True,
        ) -> LazyFrameQuery:
            """Sort the ``LazyFrame`` by the given columns."""
            ...

        def filter(  # type: ignore[empty-body]
            self,
            *predicates: (
                IntoExprColumn
                | Iterable[IntoExprColumn]
                | bool
                | list[bool]
                | np.ndarray[Any, Any]
            ),
            **constraints: Any,
        ) -> LazyFrameQuery:
            """
            Filter the rows in the ``LazyFrame`` based on a predicate expression.

            OpenDP discards relevant margin descriptors in the domain when filtering.
            """
            ...

        def select(  # type: ignore[empty-body]
            self, *exprs: IntoExpr | Iterable[IntoExpr], **named_exprs: IntoExpr
        ) -> LazyFrameQuery:
            """
            Select columns from this ``LazyFrame``.

            OpenDP allows expressions in select statements that aggregate to not be row-by-row.
            """
            ...

        def select_seq(  # type: ignore[empty-body]
            self, *exprs: IntoExpr | Iterable[IntoExpr], **named_exprs: IntoExpr
        ) -> LazyFrameQuery:
            """
            Select columns from this ``LazyFrame``.

            OpenDP allows expressions in select statements that aggregate to not be row-by-row.
            """
            ...

        def group_by(  # type: ignore[empty-body]
            self,
            *by: IntoExpr | Iterable[IntoExpr],
            maintain_order: bool = False,
            **named_by: IntoExpr,
        ) -> LazyGroupByQuery:
            """
            Start a group by operation.

            OpenDP currently requires that grouping keys be simple column expressions.
            """
            ...

        def with_columns(  # type: ignore[empty-body]
            self,
            *exprs: IntoExpr | Iterable[IntoExpr],
            **named_exprs: IntoExpr,
        ) -> LazyFrameQuery:
            """
            Add columns to this ``LazyFrame``.

            OpenDP requires that expressions in with_columns are row-by-row:
            expressions may not change the number or order of records
            """
            ...

        def with_columns_seq(  # type: ignore[empty-body]
            self,
            *exprs: IntoExpr | Iterable[IntoExpr],
            **named_exprs: IntoExpr,
        ) -> LazyFrameQuery:
            """
            Add columns to this ``LazyFrame``.

            OpenDP requires that expressions in with_columns are row-by-row:
            expressions may not change the number or order of records
            """
            ...

        def join(  # type: ignore[empty-body]
            self,
            other: _LazyFrame,
            on: str | _Expr | Sequence[str | _Expr] | None = None,
            how: JoinStrategy = "inner",
            *,
            left_on: str | _Expr | Sequence[str | _Expr] | None = None,
            right_on: str | _Expr | Sequence[str | _Expr] | None = None,
            suffix: str = "_right",
            validate: JoinValidation = "m:m",
            join_nulls: bool = False,
            coalesce: bool | None = None,
            allow_parallel: bool = True,
            force_parallel: bool = False,
        ) -> LazyFrameQuery:
            """
            Add a join operation to the Logical Plan.
            """
            ...

        def with_keys(
            self,
            keys: _LazyFrame,
            on: list[str] | None = None,
        ) -> LazyFrameQuery:
            """
            Shorthand to join with an explicit key-set.
            """
            # Motivation for adding this new API:
            # 1. Writing a left join is more difficult in the context API: 
            #   see the complexity of this implementation, where you have to go under the hood. 
            #   This gives an easier shorthand to write a left join.
            # 2. Left joins are more likely to be supported by database backends.
            # 3. Easier to use; with the Polars API the key set needs to be lazy, user must specify they want a right join and the join keys.

            if isinstance(keys, _DataFrame):
                keys = keys.lazy()
            
            if on is None:
                on = keys.collect_schema().names()

            return LazyFrameQuery(
                keys.join(self._lf_plan, how="left", on=on),
                self._query,
            )

        def resolve(self) -> Measurement:
            """Resolve the query into a measurement."""

            # access attributes of self without getting intercepted by Self.__getattribute__
            lf_plan = object.__getattribute__(self, "_lf_plan")
            query = object.__getattribute__(self, "_query")
            input_domain, input_metric = query._chain
            d_in, d_out = query._d_in, query._d_out

            def make(scale, threshold=None):
                return make_private_lazyframe(
                    input_domain=input_domain,
                    input_metric=input_metric,
                    output_measure=query._output_measure,
                    lazyframe=lf_plan,
                    global_scale=scale,
                    threshold=threshold,
                )

            # when the output measure is δ-approximate, then there are two free parameters to tune
            if getattr(query._output_measure.type, "origin", None) == "Approximate":

                # search for a scale parameter. Solve for epsilon first,
                # setting threshold to u32::MAX so as not to interfere with the search for a suitable scale parameter
                scale = binary_search(
                    lambda s: make(s, threshold=2**32 - 1).map(d_in)[0] < d_out[0],  # type: ignore[index]
                    T=float,
                )

                # attempt to return without setting a threshold
                try:
                    return make(scale, threshold=None)
                except OpenDPException:
                    pass

                # now that scale has been solved, find a suitable threshold
                threshold = binary_search(
                    lambda t: make(scale, t).map(d_in)[1] < d_out[1],  # type: ignore[index]
                    T=int,
                )

                # return a measurement with the discovered scale and threshold
                return make(scale, threshold)

            # when no delta parameter is involved,
            # finding a suitable measurement just comes down to finding scale
            return binary_search_chain(make, d_in, d_out, T=float)

        def release(self) -> OnceFrame:
            """Release the query. The query must be part of a context."""
            query = object.__getattribute__(self, "_query")
            resolve = object.__getattribute__(self, "resolve")
            return query._context(resolve())  # type: ignore[misc]

        def summarize(self, alpha: float | None = None):
            """Summarize the statistics released by this query.

            If ``alpha`` is passed, the resulting data frame includes an ``accuracy`` column.

            If a threshold is configured for censoring small/sensitive partitions,
            a threshold column will be included,
            containing the cutoff for the respective count query being thresholded.

            :example:

            >>> import polars as pl
            >>> data = pl.LazyFrame([pl.Series("convicted", [0, 1, 1, 0, 1] * 50, dtype=pl.Int32)])

            >>> context = dp.Context.compositor(
            ...     data=data,
            ...     privacy_unit=dp.unit_of(contributions=1),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=1,
            ...     margins={(): dp.polars.Margin(max_partition_length=1000)},
            ... )

            >>> query = context.query().select(
            ...     dp.len(),
            ...     pl.col("convicted").fill_null(0).dp.sum((0, 1))
            ... )

            >>> query.summarize(alpha=.05)  # type: ignore[union-attr]
            shape: (2, 5)
            ┌───────────┬──────────────┬─────────────────┬───────┬──────────┐
            │ column    ┆ aggregate    ┆ distribution    ┆ scale ┆ accuracy │
            │ ---       ┆ ---          ┆ ---             ┆ ---   ┆ ---      │
            │ str       ┆ str          ┆ str             ┆ f64   ┆ f64      │
            ╞═══════════╪══════════════╪═════════════════╪═══════╪══════════╡
            │ len       ┆ Frame Length ┆ Integer Laplace ┆ 2.0   ┆ 6.429605 │
            │ convicted ┆ Sum          ┆ Integer Laplace ┆ 2.0   ┆ 6.429605 │
            └───────────┴──────────────┴─────────────────┴───────┴──────────┘

            The accuracy in any given row can be interpreted with:

            >>> def interpret_accuracy(distribution, scale, accuracy, alpha):
            ...     return (
            ...         f"When the {distribution} scale is {scale}, "
            ...         f"the DP estimate differs from the true value by no more than {accuracy} "
            ...         f"at a statistical significance level alpha of {alpha}, "
            ...         f"or with (1 - {alpha})100% = {(1 - alpha) * 100}% confidence."
            ...     )
            ...
            >>> interpret_accuracy("Integer Laplace", 2.0, 6.429605, alpha=.05) # doctest:+SKIP

            :param alpha: optional. A value in [0, 1] denoting the statistical significance.
            """
            from opendp.accuracy import summarize_polars_measurement

            return summarize_polars_measurement(self.resolve(), alpha)

    class LazyGroupByQuery(_LazyGroupBy):
        """
        A ``LazyGroupByQuery`` is returned by :py:func:`opendp.extras.polars.LazyFrameQuery.group_by`.
        It mimics a `Polars LazyGroupBy <https://docs.pola.rs/api/python/stable/reference/lazyframe/group_by.html>`_,
        but only supports APIs documented below."""

        def __init__(self, lgb_plan: _LazyGroupBy, query):
            self._lgb_plan = lgb_plan
            self._query = query

        def agg(
            self,
            *aggs: IntoExpr | Iterable[IntoExpr],
            **named_aggs: IntoExpr,
        ) -> LazyFrameQuery:
            """
            Compute aggregations for each group of a group by operation.
            """
            lf_plan = self._lgb_plan.agg(*aggs, **named_aggs)
            return LazyFrameQuery(lf_plan, self._query)

except ImportError:  # pragma: no cover
    ERR_MSG = "LazyFrameQuery depends on Polars: `pip install 'opendp[polars]'`."

    class LazyFrameQuery(object):  # type: ignore[no-redef]
        """
        A ``LazyFrameQuery`` may be returned by :py:func:`opendp.context.Context.query`.
        It mimics a `Polars LazyFrame <https://docs.pola.rs/api/python/stable/reference/lazyframe/index.html>`_,
        but makes a few additions as documented below."""

        # Keep this docstring in sync with the docstring above for the real class.

        def resolve(self) -> Measurement:
            """Resolve the query into a measurement."""
            raise ImportError(ERR_MSG)

        def release(self) -> OnceFrame:
            """Release the query. The query must be part of a context."""
            raise ImportError(ERR_MSG)

        def summarize(self, alpha: float | None = None):
            """Summarize the statistics released by this query."""
            raise ImportError(ERR_MSG)


@dataclass
class Margin(object):
    """
    The ``Margin`` class is used to describe what information is known publicly about a grouped dataset:
    like the values you might expect to find in the margins of a table.

    Be aware that aspects of your data marked as "public information" are not subject to privacy protections,
    so it is important that public descriptors about the margin should be set conservatively, or not set at all.

    Instances of this class are used by :py:func:`opendp.context.Context.compositor`.
    """

    public_info: Literal["keys"] | Literal["lengths"] | None = None
    """Identifies properties of grouped data that are considered public information.
    
    * ``"keys"`` designates that keys are not protected
    * ``"lengths"`` designates that both keys and partition lengths are not protected
    """

    max_partition_length: int | None = None
    """An upper bound on the number of records in any one partition.

    If you don't know how many records are in the data, you can specify a very loose upper bound,
    for example, the size of the total population you are sampling from.

    This is used to resolve issues raised in the paper
    `Widespread Underestimation of Sensitivity in Differentially Private Libraries and How to Fix It <https://arxiv.org/pdf/2207.10635.pdf>`_.
    """

    max_num_partitions: int | None = None
    """An upper bound on the number of distinct partitions."""

    max_partition_contributions: int | None = None
    """The greatest number of records an individual may contribute to any one partition.
    
    This can significantly reduce the sensitivity of grouped queries under zero-Concentrated DP.
    """

    max_influenced_partitions: int | None = None
    """The greatest number of partitions any one individual can contribute to."""
