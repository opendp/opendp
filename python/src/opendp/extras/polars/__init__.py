"""
This module requires extra installs: ``pip install 'opendp[polars]'``

The ``opendp.extras.polars`` module adds differential privacy to the
`Polars DataFrame library <https://docs.pola.rs>`_.

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.polars``.
"""

from __future__ import annotations
from dataclasses import asdict, dataclass, field, replace
import os
from typing import Any, Literal, Mapping, Optional, Sequence, Union, cast
from opendp._lib import lib_path, import_optional_dependency
from opendp.extras.mbi import ContingencyTable, make_contingency_table, AIM, Algorithm
from opendp.mod import (
    ChangeOneIdDistance,
    Domain,
    Measurement,
    FrameDistance,
    Metric,
    OpenDPException,
    SymmetricIdDistance,
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
    enum_domain,
    array_domain,
)
from opendp.measurements import make_private_lazyframe
from deprecated import deprecated
from opendp.transformations import make_stable_lazyframe
from typing import TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    from opendp.context import Query
    from opendp.extras.polars.contingency_table import ContingencyTableQuery

def _get_opendp_polars_lib_path():
    return os.environ.get("OPENDP_POLARS_LIB_PATH", lib_path)

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
        self.expr = expr

    def noise(
        self,
        scale: float | None = None,
    ):
        """Add noise to the expression.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.
        The noise distribution is chosen according to the privacy definition:

        * Pure-DP: Laplace noise, where ``scale == standard_deviation / sqrt(2)``
        * zCDP: Gaussian noise, where ``scale == standard_devation``

        :param scale: Scale parameter for the noise distribution.

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

        return register_plugin_function(
            plugin_path=_get_opendp_polars_lib_path(),
            function_name="noise",
            args=(self.expr, scale),
            is_elementwise=True,
        )

    @deprecated(version="0.14.1", reason="Use .noise instead. This will now apply gaussian noise if your privacy definition is zCDP.")
    def laplace(self, scale: float | None = None):
        """Add Laplace noise to the expression.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: Noise scale parameter for the Laplace distribution. ``scale == standard_deviation / sqrt(2)``
        """
        return self.noise(scale=scale)

    @deprecated(version="0.14.1", reason="Use .noise instead. This will now apply laplace noise if your privacy definition is pure-DP.")
    def gaussian(self, scale: float | None = None):
        """Add Gaussian noise to the expression.

        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param scale: Noise scale parameter for the Gaussian distribution. ``scale == standard_deviation``
        """
        return self.noise(scale=scale)

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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=_get_opendp_polars_lib_path(),
            function_name="dp_len",
            args=(self.expr, scale),
            returns_scalar=True,
        )

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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=_get_opendp_polars_lib_path(),
            function_name="dp_count",
            args=(self.expr, scale),
            returns_scalar=True,
        )

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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=_get_opendp_polars_lib_path(),
            function_name="dp_null_count",
            args=(self.expr, scale),
            returns_scalar=True,
        )

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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=_get_opendp_polars_lib_path(),
            function_name="dp_n_unique",
            args=(self.expr, scale),
            returns_scalar=True,
        )

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
        ...     margins=[dp.polars.Margin(max_length=5)]
        ... )
        >>> query = context.query().select(pl.col("visits").dp.sum((0, 1)))
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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        from polars import lit  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=_get_opendp_polars_lib_path(),
            function_name="dp_sum",
            args=(self.expr, lit(bounds[0]), lit(bounds[1]), scale),
            returns_scalar=True,
            changes_length=True,
        )

    def mean(
        self,
        bounds: tuple[float, float],
        scale: float | None = None,
    ):
        """Compute the differentially private mean.

        The amount of noise to be added to the sum is determined by the scale.
        If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

        :param bounds: clip the input data to these lower and upper bounds
        :param scale: relative parameter for the scale of the noise distributions

        :example:

        >>> import polars as pl
        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({"visits": [1, 2, None]}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.),
        ...     split_evenly_over=1,
        ...     margins=[dp.polars.Margin(max_length=5)]
        ... )
        >>> query = context.query().select(pl.col("visits").dp.mean((0, 1)))
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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        from polars import lit  # type: ignore[import-not-found]

        if isinstance(scale, tuple):  # pragma: no cover
            raise ValueError("OpenDP 0.14.1 adjusts the scale to only consist of a single float. "
                             "Individually estimate sum and len to tune budget distribution.")

        return register_plugin_function(
            plugin_path=_get_opendp_polars_lib_path(),
            function_name="dp_mean",
            args=(self.expr, lit(bounds[0]), lit(bounds[1]), scale),
            returns_scalar=True,
            changes_length=True,
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
        ...     margins=[dp.polars.Margin(max_length=100)]
        ... )
        >>> candidates = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90]
        >>> query = context.query().select(pl.col("age").cast(int).dp.quantile(0.25, candidates))
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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        from polars import lit, Series # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=_get_opendp_polars_lib_path(),
            function_name="dp_quantile",
            args=(self.expr, lit(alpha), lit(Series(candidates)), scale),
            returns_scalar=True,
            changes_length=True,
        )

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
        ...     margins=[dp.polars.Margin(max_length=100)]
        ... )
        >>> candidates = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90]
        >>> query = context.query().select(pl.col("age").cast(int).dp.quantile(0.5, candidates))
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
        from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
        from polars import lit, Series  # type: ignore[import-not-found]

        return register_plugin_function(
            plugin_path=_get_opendp_polars_lib_path(),
            function_name="dp_median",
            args=(self.expr, lit(Series(candidates)), scale),
            returns_scalar=True,
            changes_length=True,
        )


pl = import_optional_dependency("polars", raise_error=False)
if pl is not None:
    pl.api.register_expr_namespace("dp")(DPExpr)


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
    from polars.plugins import register_plugin_function  # type: ignore[import-not-found]

    return register_plugin_function(
        plugin_path=_get_opendp_polars_lib_path(),
        function_name="dp_frame_len",
        args=(scale,),
        returns_scalar=True,
    )


class OnceFrame(object):
    """OnceFrame is a Polars LazyFrame that may only be collected into a DataFrame once.

    The APIs on this class mimic those that can be found in Polars.

    Differentially private guarantees on a given LazyFrame require the LazyFrame to be evaluated at most once.
    The purpose of this class is to protect against repeatedly evaluating the LazyFrame.
    """

    def __init__(self, queryable):
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
    name, dtype = field
    return series_domain(name, option_domain(_domain_from_dtype(dtype)))


def _domain_from_dtype(dtype) -> Domain:
    """Builds the broadest possible Domain that matches a given dtype."""
    import polars as pl

    if dtype == pl.Categorical:
        return categorical_domain()
    if dtype == pl.Enum:
        return enum_domain(dtype.categories)
    if dtype == pl.Datetime:
        return datetime_domain(dtype.time_unit, dtype.time_zone)
    if dtype == pl.Array:
        return array_domain(_domain_from_dtype(dtype.inner), dtype.size)

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

    return atom_domain(T=T)


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


@dataclass
class SortBy:
    """Configuration for ``keep`` in :py:meth:`LazyFrameQuery.truncate_per_group`.

    Follows the arguments in Polars' ``sort_by`` method.
    """

    by: Any
    """Column(s) to sort by. Accepts expression input. Strings are parsed as column names."""

    descending: bool | Sequence[bool] = False
    """Sort in descending order. When sorting by multiple columns, can be specified per column by passing a sequence of booleans."""

    nulls_last: bool | Sequence[bool] = False
    """Place null values last; can specify a single boolean applying to all columns or a sequence of booleans for per-column control."""

    multithreaded: bool = True
    """Sort using multiple threads."""

    maintain_order: bool = False
    """Whether the order should be maintained if elements are equal."""


class LazyFrameQuery:
    """
    A ``LazyFrameQuery`` may be returned by :py:func:`opendp.context.Context.query`.
    It mimics a `Polars LazyFrame <https://docs.pola.rs/api/python/stable/reference/lazyframe/index.html>`_,
    but makes a few additions and changes as documented below."""

    # Keep this docstring in sync with the docstring below for the dummy class.

    def __init__(self, polars_plan, query):
        self.polars_plan = polars_plan
        self._query = query
        # do not initialize super() because inheritance is only used to mimic the API surface

    def __getattribute__(self, name):
        # Re-route all possible attribute access to self.polars_plan.
        # __getattribute__ is necessary because __getattr__ cannot intercept calls to inherited methods

        # We keep the query plan void of data anyways,
        # so running the computation doesn't affect privacy.
        # This doesn't have to cover all possible APIs that may execute the query,
        #    but it does give a simple sanity check for the obvious cases.
        if name in _LAZY_EXECUTION_METHODS:
            raise ValueError("You must call `.release()` before executing a query.")

        polars_plan = object.__getattribute__(self, "polars_plan")
        query = object.__getattribute__(self, "_query")
        attr = getattr(polars_plan, name, None)

        # If not a valid attribute on self.polars_plan, then don't re-route
        if attr is None:
            return object.__getattribute__(self, name)

        # any callable attributes (like .with_columns or .select) will now also wrap their outputs in a LazyFrameQuery
        if callable(attr):

            def _wrap(*args, **kwargs):
                out = attr(*args, **kwargs)

                if pl is not None:
                    # re-wrap any lazy outputs to keep the conveniences afforded by this class
                    if isinstance(out, pl.lazyframe.frame.LazyFrame):
                        return LazyFrameQuery(out, query)

                    if isinstance(out, pl.lazyframe.group_by.LazyGroupBy):
                        return LazyGroupByQuery(out, query)

                return out

            return _wrap
        return attr

    # These definitions are primarily for mypy:
    # Without them, a "# type: ignore[union-attr]" is needed on every line where these methods are used.
    # The docstrings are not seen by Sphinx, but aren't doing any harm either.

    def sort(  # type: ignore[empty-body]
        self,
        by,
        *more_by,
        descending: bool | Sequence[bool] = False,
        nulls_last: bool | Sequence[bool] = False,
        maintain_order: bool = False,
        multithreaded: bool = True,
    ) -> LazyFrameQuery:
        """Sort the ``LazyFrame`` by the given columns."""
        ...

    def filter(  # type: ignore[empty-body]
        self,
        *predicates,
        **constraints: Any,
    ) -> LazyFrameQuery:
        """
        Filter the rows in the ``LazyFrame`` based on a predicate expression.

        OpenDP discards relevant margin descriptors in the domain when filtering.
        """
        ...

    def select(  # type: ignore[empty-body]
        self, *exprs, **named_exprs
    ) -> LazyFrameQuery:
        """
        Select columns from this ``LazyFrame``.

        OpenDP expects expressions in select statements that don't aggregate to be row-by-row.
        """
        ...

    def select_seq(  # type: ignore[empty-body]
        self, *exprs, **named_exprs
    ) -> LazyFrameQuery:
        """
        Select columns from this ``LazyFrame``.

        OpenDP allows expressions in select statements that aggregate to not be row-by-row.
        """
        ...

    def group_by(  # type: ignore[empty-body]
        self,
        *by,
        maintain_order: bool = False,
        **named_by,
    ) -> LazyGroupByQuery:
        """
        Start a group by operation.

        OpenDP currently requires that grouping keys be simple column expressions.
        """
        ...

    def with_columns(  # type: ignore[empty-body]
        self,
        *exprs,
        **named_exprs,
    ) -> LazyFrameQuery:
        """
        Add columns to this ``LazyFrame``.

        OpenDP requires that expressions in with_columns are row-by-row:
        expressions may not change the number or order of records
        """
        ...

    def with_columns_seq(  # type: ignore[empty-body]
        self,
        *exprs,
        **named_exprs,
    ) -> LazyFrameQuery:
        """
        Add columns to this ``LazyFrame``.

        OpenDP requires that expressions in with_columns are row-by-row:
        expressions may not change the number or order of records
        """
        ...

    def join(  # type: ignore[empty-body]
        self,
        other,
        on=None,
        how="inner",
        *,
        left_on=None,
        right_on=None,
        suffix: str = "_right",
        validate="m:m",
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
        keys,
        on: list[str] | None = None,
    ) -> LazyFrameQuery:
        """
        Shorthand to join with an explicit key-set.

        :param keys: lazyframe containing a key-set whose columns correspond to the grouping keys
        :param on: optional, the names of columns to join on. Useful if the key dataframe contains extra columns
        """
        # Motivation for adding this new API:
        # 1. Writing a left join is more difficult in the context API:
        #   see the complexity of this implementation, where you have to go under the hood.
        #   This gives an easier shorthand to write a left join.
        # 2. Left joins are more likely to be supported by database backends.
        # 3. Easier to use; with the Polars API the key set needs to be lazy, user must specify they want a right join and the join keys.

        if pl is not None:
            if isinstance(keys, pl.dataframe.frame.DataFrame):
                keys = keys.lazy()

        if on is None:
            on = keys.collect_schema().names()

        return LazyFrameQuery(
            keys.join(self.polars_plan, how="left", on=on, nulls_equal=True),
            self._query,
        )

    def truncate_per_group(
        self,
        k: int,
        by: list[Any] | None = None,
        keep: Literal["sample", "first", "last"] | SortBy = "sample",
    ) -> LazyFrameQuery:
        """
        Limit the number of contributed rows per group.

        :param k: the number of rows to keep for each identifier and group
        :param by: optional, additional columns to group by
        :param keep: which rows to keep for each identifier in each group
        """
        input_metric = self._query._chain[1]

        if isinstance(by, str):
            raise ValueError(
                "by must be a list of strings or expressions"
            )  # pragma: no cover

        if isinstance(input_metric, FrameDistance):
            input_metric = input_metric.inner_metric
        if not isinstance(input_metric, (SymmetricIdDistance, ChangeOneIdDistance)):
            raise ValueError("truncation is only valid when identifier is defined")

        if keep == "sample":
            indexes = pl.int_range(pl.len()).shuffle()
        elif keep == "first":
            indexes = pl.int_range(pl.len())
        elif keep == "last":
            indexes = pl.int_range(pl.len()).reverse()
        elif isinstance(keep, SortBy):
            indexes = pl.int_range(pl.len()).sort_by(**asdict(keep))
        else:
            raise ValueError(
                "keep must be 'sample', 'first', 'last' or SortBy"
            )  # pragma: no cover

        return self.filter(indexes.over(input_metric.identifier, *by or []) < k)

    def truncate_num_groups(
        self,
        k: int,
        by: list[Any],
        keep: Literal["sample", "first", "last"] = "sample",
    ) -> LazyFrameQuery:
        """
        Limit the number of groups an individual may influence.

        :param k: the number of groups to keep for each identifier
        :param by: when grouped by these grouping columns
        :param keep: which groups to keep for each identifier
        """
        input_metric = self._query._chain[1]

        if isinstance(by, str):
            raise ValueError(
                "by must be a list of strings or expressions"
            )  # pragma: no cover

        if isinstance(input_metric, FrameDistance):
            input_metric = input_metric.inner_metric
        if not isinstance(input_metric, (SymmetricIdDistance, ChangeOneIdDistance)):
            raise ValueError("truncation is only valid when identifier is defined")

        struct = pl.struct(*by)
        if keep == "sample":
            ranks = pl.struct(struct.hash(), struct).rank("dense")
        elif keep == "first":
            ranks = struct.rank("dense", descending=False)
        elif keep == "last":
            ranks = struct.rank("dense", descending=True)
        else:
            raise ValueError(
                "keep must be 'sample', 'first' or 'last'"
            )  # pragma: no cover

        return self.filter(ranks.over(input_metric.identifier) < k)

    def resolve(self) -> Measurement:
        """Resolve the query into a measurement."""

        # access attributes of self without getting intercepted by Self.__getattribute__
        polars_plan = object.__getattribute__(self, "polars_plan")
        query = object.__getattribute__(self, "_query")
        input_domain, input_metric = query._chain
        d_in, d_out = query._d_in, query._d_out

        def _make(scale, threshold=None):
            return make_private_lazyframe(
                input_domain=input_domain,
                input_metric=input_metric,
                output_measure=query._output_measure,
                lazyframe=polars_plan,
                global_scale=scale,
                threshold=threshold,
            )

        # when the query has sensitivity zero or is behind an invariant
        try:
            m_zero = _make(0.0, threshold=None)
            if m_zero.check(d_in, d_out):
                # if the zero scale measurement is already private, return it
                return m_zero
        except OpenDPException:
            pass

        # when the output measure is δ-approximate, then there are two free parameters to tune
        if getattr(query._output_measure.type, "origin", None) == "Approximate":
            # search for a scale parameter. Solve for epsilon first,
            # setting threshold to u32::MAX so as not to interfere with the search for a suitable scale parameter
            scale = binary_search(
                lambda s: _make(s, threshold=2**32 - 1).map(d_in)[0] < d_out[0],  # type: ignore[index]
                T=float,
            )

            # attempt to return without setting a threshold
            try:
                return _make(scale, threshold=None)
            except OpenDPException:
                pass

            # now that scale has been solved, find a suitable threshold
            threshold = binary_search(
                lambda t: _make(scale, t).map(d_in)[1] < d_out[1],  # type: ignore[index]
                T=int,
            )

            # return a measurement with the discovered scale and threshold
            return _make(scale, threshold)

        # when no delta parameter is involved,
        # finding a suitable measurement just comes down to finding scale
        return binary_search_chain(_make, d_in, d_out, T=float)

    def release(self) -> OnceFrame:
        """Release the query. The query must be part of a context."""
        query = object.__getattribute__(self, "_query")
        resolve = object.__getattribute__(self, "resolve")
        return query._context(resolve())  # type: ignore[misc]

    def summarize(self, alpha: float | None = None):
        """Summarize the statistics released by this query.

        If ``alpha`` is passed, the resulting data frame includes an ``accuracy`` column.

        If a threshold is configured for censoring small/sensitive groups,
        a threshold column will be included,
        containing the cutoff for the respective count query being thresholded.

        :param alpha: optional. A value in [0, 1] denoting the statistical significance. For the corresponding confidence level, subtract from from 1: for 95% confidence, use 0.05 for alpha.

        :example:

        .. code:: pycon

            >>> import polars as pl
            >>> data = pl.LazyFrame([pl.Series("convicted", [0, 1, 1, 0, 1] * 50, dtype=pl.Int32)])

            >>> context = dp.Context.compositor(
            ...     data=data,
            ...     privacy_unit=dp.unit_of(contributions=1),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=1,
            ...     margins=[dp.polars.Margin(by=(), max_length=1000)],
            ... )

            >>> query = context.query().select(
            ...     dp.len(),
            ...     pl.col("convicted").dp.sum((0, 1))
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

        .. code:: pycon

            >>> def interpret_accuracy(distribution, scale, accuracy, alpha):
            ...     return (
            ...         f"When the {distribution} scale is {scale}, "
            ...         f"the DP estimate differs from the true value by no more than {accuracy} "
            ...         f"at a statistical significance level alpha of {alpha}, "
            ...         f"or with (1 - {alpha})100% = {(1 - alpha) * 100}% confidence."
            ...     )

            >>> interpret_accuracy("Integer Laplace", 2.0, 6.429605, alpha=.05) # doctest:+SKIP
        """
        from opendp.accuracy import summarize_polars_measurement

        return summarize_polars_measurement(self.resolve(), alpha)

        
    def contingency_table(
        self,
        *,
        keys: Optional[Mapping[str, Sequence]] = None,
        cuts: Optional[Mapping[str, Sequence[float]]] = None,
        table: Optional[ContingencyTable] = None,
        algorithm: Union[Algorithm] = AIM(),
    ) -> "ContingencyTableQuery":
        """Release an approximation to a contingency table across all columns.

        :param keys: dictionary of column names and unique categories
        :param cuts: dictionary of column names and bin edges for numerical columns
        :param table: ContingencyTable from prior release
        :param algorithm: configuration for internal estimation algorithm
        """
        from .contingency_table import ContingencyTableQuery

        query: Query = object.__getattribute__(self, "_query")
        input_domain, input_metric = cast(tuple[Domain, Metric], query._chain)
        d_in, d_out = query._d_in, query._d_out

        t_plan = make_stable_lazyframe(
            input_domain,
            input_metric,
            lazyframe=object.__getattribute__(self, "polars_plan"),
        )

        m_table, oneway_scale, oneway_threshold = make_contingency_table(
            input_domain=t_plan.output_domain,
            input_metric=t_plan.output_metric,
            output_measure=query._output_measure,
            d_in=t_plan.map(d_in),
            d_out=d_out,  # type: ignore[arg-type]
            keys=keys,
            cuts=cuts,
            table=table,
            algorithm=algorithm,
        )

        return ContingencyTableQuery(
            chain=t_plan >> m_table,
            output_measure=query._output_measure,
            context=query._context,
            oneway_scale=oneway_scale,
            oneway_threshold=oneway_threshold
        )


class LazyGroupByQuery:
    """
    A ``LazyGroupByQuery`` is returned by :py:func:`opendp.extras.polars.LazyFrameQuery.group_by`.
    It mimics a `Polars LazyGroupBy <https://docs.pola.rs/api/python/stable/reference/lazyframe/group_by.html>`_,
    but only supports APIs documented below."""

    def __init__(self, lgb_plan, query):
        self._lgb_plan = lgb_plan
        self._query = query

    def agg(
        self,
        *aggs,
        **named_aggs,
    ) -> LazyFrameQuery:
        """
        Compute aggregations for each group of a group by operation.

        :param aggs: expressions to apply in the aggregation context
        :param named_aggs: named/aliased expressions to apply in the aggregation context
        """
        polars_plan = self._lgb_plan.agg(*aggs, **named_aggs)
        return LazyFrameQuery(polars_plan, self._query)


@dataclass
class Margin:
    """
    The ``Margin`` class is used to describe what information is known publicly about a grouped dataset:
    like the values you might expect to find in the margins of a table.

    Be aware that aspects of your data marked as "public information" are not subject to privacy protections,
    so it is important that public descriptors about the margin should be set conservatively, or not set at all.

    Instances of this class are used by :py:func:`opendp.context.Context.compositor`.
    """

    by: Sequence = field(default_factory=list)
    """Polars expressions describing the grouping columns."""

    max_length: int | None = None
    """An upper bound on the number of records in any one group.

    Some operations (for instance, for float sums) will error if `max_length` is not provided.
    This is used to resolve issues raised in the paper
    `Widespread Underestimation of Sensitivity in Differentially Private Libraries and How to Fix It <https://arxiv.org/pdf/2207.10635.pdf>`_.
    
    If you don't know how many records are in the data, you can specify a very loose upper bound,
    for example, the size of the total population you are sampling from.
    """

    max_groups: int | None = None
    """An upper bound on the number of distinct groups."""

    invariant: Literal["keys"] | Literal["lengths"] | None = None
    """Identifies properties of grouped data that are considered invariant.
    
    * ``"keys"`` designates that keys are not protected
    * ``"lengths"`` designates that both keys and group lengths are not protected

    By the analysis of invariants conducted in
    `Formal Privacy Guarantees with Invariant Statistics <https://arxiv.org/abs/2410.17468>`_,
    when invariants are set, the effective privacy guarantees of the library are weaker than advertised.
    """

    @property
    @deprecated(version="0.13.0", reason="Use max_length instead.")
    def max_partition_length(self):
        return self.max_length  # pragma: no cover

    @max_partition_length.setter
    @deprecated(reason="Use max_length instead.")
    def max_partition_length(self, value):
        self.max_length = value  # pragma: no cover

    @property
    @deprecated(
        version="0.13.0",
        reason="Use max_groups instead. This was renamed to be consistent with Polars terminology."
    )
    def max_num_partitions(self):
        return self.max_groups  # pragma: no cover

    @max_num_partitions.setter
    @deprecated(
        version="0.13.0",
        reason="Use max_groups instead. This was renamed to be consistent with Polars terminology."
    )
    def max_num_partitions(self, value):
        self.max_groups = value  # pragma: no cover

    @property
    @deprecated(
        version="0.13.0",
        reason='Use invariant instead. This was renamed because invariants are not "public information". Invariants are "unprotected information".'
    )
    def public_info(self):
        return self.invariant  # pragma: no cover

    @public_info.setter
    @deprecated(
        version="0.13.0",
        reason='Use invariant instead. This was renamed because invariants are not "public information". Invariants are "unprotected information".'
    )
    def public_info(self, value):
        self.invariant = value  # pragma: no cover

    @property
    def max_partition_contributions(self):
        raise NotImplementedError(
            "max_partition_contributions has been moved. Use `dp.unit_of(contributions=[dp.polars.Bound(per_group=...)])` instead."
        )  # pragma: no cover

    @max_partition_contributions.setter
    def max_partition_contributions(self, value):
        raise NotImplementedError(
            "max_partition_contributions has been moved. Use `dp.unit_of(contributions=[dp.polars.Bound(per_group=...)])` instead."
        )  # pragma: no cover

    @property
    def max_influenced_partitions(self):
        raise NotImplementedError(
            "max_influenced_partitions has been moved. Use `dp.unit_of(contributions=[dp.polars.Bound(num_groups=...)])` instead."
        )  # pragma: no cover

    @max_influenced_partitions.setter
    def max_influenced_partitions(self, value):
        raise NotImplementedError(
            "max_influenced_partitions has been moved. Use `dp.unit_of(contributions=[dp.polars.Bound(num_groups=...)])` instead."
        )  # pragma: no cover

    def __eq__(self, other) -> bool:
        if not isinstance(other, Margin):
            return False

        # special logic for by, which is considered a set (order and dupes don't matter)
        # and may contain expressions that do not have a boolean equality operator
        def serialize(by):
            if isinstance(by, str):
                by = pl.col(by)
            return by.meta.serialize()

        self_by = {serialize(col) for col in self.by}
        other_by = {serialize(col) for col in other.by}
        if self_by != other_by:
            return False

        return asdict(replace(self, by=[])) == asdict(replace(other, by=[]))


@dataclass
class Bound(object):
    """
    The ``Bound`` class is used to describe bounds on the number of
    contributed rows per-group and the number of contributed groups.
    """

    by: Sequence = field(default_factory=list)
    """Polars expressions describing the grouping columns."""

    per_group: int | None = None
    """The greatest number of records an individual may contribute to any one group.
    
    This can significantly reduce the sensitivity of grouped queries under zero-Concentrated DP.
    """

    num_groups: int | None = None
    """The greatest number of groups any one individual can contribute."""

    def __eq__(self, other) -> bool:
        if not isinstance(other, Bound):
            return False

        # special logic for by, which is considered a set (order and dupes don't matter)
        # and may contain expressions that do not have a boolean equality operator
        def serialize(by):
            if isinstance(by, str):
                by = pl.col(by)
            return by.meta.serialize()

        self_by = {serialize(col) for col in self.by}
        other_by = {serialize(col) for col in other.by}
        if self_by != other_by:
            return False

        return asdict(replace(self, by=[])) == asdict(replace(other, by=[]))  # type: ignore[arg-type]
