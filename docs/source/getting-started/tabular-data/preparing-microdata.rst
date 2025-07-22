Preparing Microdata
===================

Data is seldom already in the form you need it in. We use Polars
*expressions* to describe how to build new columns and Polars *contexts*
to describe how those expressions are applied to your data. More
information can be found in the `Polars User
Guide <https://docs.pola.rs/user-guide/concepts/expressions-and-contexts/#group_by-and-aggregations>`__.

This section explains OpenDP’s supported contexts for preparing
microdata (column addition and filtering).

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import polars as pl
            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")
            
            >>> context = dp.Context.compositor(
            ...     # Many columns contain mixtures of strings and numbers and cannot be parsed as floats,
            ...     # so we'll set `ignore_errors` to true to avoid conversion errors.
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
            ...     split_evenly_over=4,
            ...     margins=[dp.polars.Margin(max_length=150_000 * 36)]
            ... )
            

Previous documentation sections cover the ``.select`` context for
aggregation and the ``.agg`` context for aggregation. OpenDP allows
expressions used in the ``.select`` context and ``.agg`` context to
change the number and order of rows, whereas expressions used in the
``.with_columns`` context, ``.filter`` context and ``.group_by`` context
must be row-by-row.

With Columns
------------

[`Polars
Documentation <https://docs.pola.rs/user-guide/concepts/expressions-and-contexts/#with_columns>`__]

``.with_columns`` resolves each passed expression to a column and then
adds those columns to the data.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_hwusual_binned = (
            ...     context.query()
            ...     # shadows the usual work hours "HWUSUAL" column with binned data
            ...     .with_columns(pl.col.HWUSUAL.cut(breaks=[0, 20, 40, 60, 80, 98], left_closed=True))
            ...     .group_by(pl.col.HWUSUAL)
            ...     .agg(dp.len())
            ... )
            >>> query_hwusual_binned.release().collect().sort("HWUSUAL") # doctest: +FUZZY_DF
            shape: (7, 2)
            ┌───────────┬─────────┐
            │ HWUSUAL   ┆ len     │
            │ ---       ┆ ---     │
            │ cat       ┆ u32     │
            ╞═══════════╪═════════╡
            │ null      ┆ ...     │
            │ [0, 20)   ┆ ...     │
            │ [20, 40)  ┆ ...     │
            │ [40, 60)  ┆ ...     │
            │ [60, 80)  ┆ ...     │
            │ [80, 98)  ┆ ...     │
            │ [98, inf) ┆ ...     │
            └───────────┴─────────┘
            

To ensure that the privacy unit remains meaningful, expressions passed
into ``.with_columns`` must be row-by-row, meaning that the expression
could be represented as a function applied to each row in the data. The
row-by-row property implies that the expression doesn’t break the
alignment between individual contributions in the data and their
individual contributions in the new constructed columns.

Another consideration is that any new columns added by ``.with_columns``
do not (currently) have margin descriptors. For instance, in the above
query, any margin descriptors related to ``HWUSUAL`` would no longer
apply to the new, shadowing, ``HWUSUAL`` column after ``.with_columns``.

Select
------

[`Polars
Documentation <https://docs.pola.rs/user-guide/concepts/expressions-and-contexts/#select>`__]

``.select`` resolves each passed expression to a column and then returns
those columns. The behavior is the same as ``.with_columns``, but only
the columns specified in expressions will remain.

Filter
------

[`Polars
Documentation <https://docs.pola.rs/user-guide/concepts/expressions-and-contexts/#filter>`__]

``.filter`` uses row-by-row expressions of booleans to mask rows.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_total_hours_worked = (
            ...     context.query()
            ...     .with_columns(pl.col.HWUSUAL.cast(int).fill_null(0))
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     .select(pl.col.HWUSUAL.dp.sum((0, 80)))
            ... )
            >>> print('sum:', query_total_hours_worked.release().collect().item())
            sum: ...


Filtering discards *all* invariants about the group keys and group
sizes. Margin descriptors are considered applicable for the input
dataset, so a data-dependent filtering renders these invariants invalid.

Otherwise, filtering preserves all other margin descriptors, because
filtering only ever removes rows.

Group By (Private)
------------------

`Polars
Documentation <https://docs.pola.rs/user-guide/concepts/expressions-and-contexts/#group_by-and-aggregations>`__

``.group_by`` also resolves each passed expression to a column, and then
groups on those columns. Just like ``.select`` and ``.with_columns``,
these expressions must be row-by-row.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_hwusual_binned = (
            ...     context.query()
            ...     .group_by(pl.col.HWUSUAL.cut([0, 20, 40, 60, 80, 98], left_closed=True))
            ...     .agg(dp.len())
            ... )
            >>> query_hwusual_binned.release().collect().sort("HWUSUAL") # doctest: +FUZZY_DF
            shape: (7, 2)
            ┌───────────┬─────────┐
            │ HWUSUAL   ┆ len     │
            │ ---       ┆ ---     │
            │ cat       ┆ u32     │
            ╞═══════════╪═════════╡
            │ null      ┆ ...     │
            │ [0, 20)   ┆ ...     │
            │ [20, 40)  ┆ ...     │
            │ [40, 60)  ┆ ...     │
            │ [60, 80)  ┆ ...     │
            │ [80, 98)  ┆ ...     │
            │ [98, inf) ┆ ...     │
            └───────────┴─────────┘


This is the same query as shown above, but with the binning moved into
the group by context.

Group By / Agg (Stable)
-----------------------

``group_by/agg`` can also be used earlier in the data pipeline, before
the private ``group_by/agg`` or ``select`` aggregation. This is a
generalization of the *sample and aggregate* framework.

The approach is appealing because arbitrary expressions can be used in
the ``agg`` argument, but it comes with the drawback that a large amount
of data is needed to get reasonable utility.

The following query demonstrates how you can use the approach to compute
arbitrary statistics, by first computing a statistic of interest (the
min) on each of roughly 1000 groups, and then releasing a differentially
private mean.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_hwusual_binned = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     # group 1000 ways
            ...     .group_by(pl.col.PIDENT % 1000)
            ...     .agg(pl.col.HWUSUAL.min())
            ...     # up to 1000 records left to work with to compute a DP mean
            ...     .select(pl.col.HWUSUAL.cast(int).fill_null(0).dp.mean((0, 30)))
            ... )
            >>> query_hwusual_binned.summarize()
            shape: (2, 4)
            ┌─────────┬───────────┬─────────────────┬─────────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale   │
            │ ---     ┆ ---       ┆ ---             ┆ ---     │
            │ str     ┆ str       ┆ str             ┆ f64     │
            ╞═════════╪═══════════╪═════════════════╪═════════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 17280.0 │
            │ HWUSUAL ┆ Length    ┆ Integer Laplace ┆ 576.0   │
            └─────────┴───────────┴─────────────────┴─────────┘


The noise scale is also relatively large. The current configuration of
the context doesn’t know that all records from a user share the same
``PIDENT``. This information can be added when building the context:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> context_pident = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=dp.unit_of(contributions=[
            ...         dp.polars.Bound(per_group=36),
            ...         # a user can only be in one group at a time when grouped this way
            ...         dp.polars.Bound(by=[pl.col.PIDENT % 1000], num_groups=1),
            ...     ]),
            ...     privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
            ...     split_evenly_over=4,
            ...     margins=[dp.polars.Margin(max_length=150_000 * 36)]
            ... )
            >>> query_hwusual_binned = (
            ...     context_pident.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     # group 1000 ways
            ...     .group_by(pl.col.PIDENT % 1000)
            ...     .agg(pl.col.HWUSUAL.min())
            ...     # up to 1000 records left to work with to compute a DP mean
            ...     .select(pl.col.HWUSUAL.cast(int).fill_null(0).dp.mean((0, 30)))
            ... )
            >>> query_hwusual_binned.summarize()
            shape: (2, 4)
            ┌─────────┬───────────┬─────────────────┬───────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale │
            │ ---     ┆ ---       ┆ ---             ┆ ---   │
            │ str     ┆ str       ┆ str             ┆ f64   │
            ╞═════════╪═══════════╪═════════════════╪═══════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 480.0 │
            │ HWUSUAL ┆ Length    ┆ Integer Laplace ┆ 16.0  │
            └─────────┴───────────┴─────────────────┴───────┘


Adding this ``Bound`` reduced the noise scale by a factor of 36, because
in the resulting dataset, only at most one record is changed, instead of
36. Nevertheless, the ``group_by/agg`` doubles the amount of noise
necessary, because contributing one record results in a change of the
aggregated record.
