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
            ...     margins={(): dp.polars.Margin(max_partition_length=60_000_000 * 36)}
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
            ...     .with_columns(pl.col.HWUSUAL.cut(breaks=[0, 20, 40, 60, 80, 98]))
            ...     .group_by(pl.col.HWUSUAL)
            ...     .agg(dp.len())
            ... )
            >>> query_hwusual_binned.release().collect().sort("HWUSUAL")
            shape: (4, 2)
            ┌───────────┬────────┐
            │ HWUSUAL   ┆ len    │
            │ ---       ┆ ---    │
            │ cat       ┆ u32    │
            ╞═══════════╪════════╡
            │ (0, 20]   ┆ ...    │
            │ (20, 40]  ┆ ...    │
            │ (40, 60]  ┆ ...    │
            │ (98, inf] ┆ ...    │
            └───────────┴────────┘

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (4, 2)</small><table border="1" class="dataframe"><thead><tr><th>HWUSUAL</th><th>len</th></tr><tr><td>cat</td><td>u32</td></tr></thead><tbody><tr><td>&quot;(0, 20]&quot;</td><td>6407</td></tr><tr><td>&quot;(20, 40]&quot;</td><td>54209</td></tr><tr><td>&quot;(40, 60]&quot;</td><td>15472</td></tr><tr><td>&quot;(98, inf]&quot;</td><td>119814</td></tr></tbody></table></div>



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
            ...     .filter(pl.col.HWUSUAL > 0)
            ...     .select(pl.col.HWUSUAL.dp.sum((0, 80)))
            ... )
            >>> query_total_hours_worked.release().collect()
            shape: (1, 1)
            ┌──────────┐
            │ HWUSUAL  │
            │ ---      │
            │ i64      │
            ╞══════════╡
            │ ...      │
            └──────────┘


Filtering discards *all* ``public_info`` invariants about the partition
keys and partition sizes. Margin descriptors are considered applicable
for the input dataset, so a data-dependent filtering renders these
invariants invalid.

Otherwise, filtering preserves all other margin descriptors, because
filtering only ever removes rows.


