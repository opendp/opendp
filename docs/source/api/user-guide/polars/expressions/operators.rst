Operators
=========

[`Polars
Documentation <https://docs.pola.rs/api/python/stable/reference/expressions/operators.html>`__]

All Polars
`conjunction <https://docs.pola.rs/api/python/stable/reference/expressions/operators.html#conjunction>`__,
`comparison <https://docs.pola.rs/api/python/stable/reference/expressions/operators.html#comparison>`__,
and
`binary <https://docs.pola.rs/api/python/stable/reference/expressions/operators.html#binary>`__
operators in the linked documentation are supported and are considered
row-by-row.

Even if you are in an aggregation context like ``.select`` or ``.agg``,
OpenDP enforces that inputs to binary operators are row-by-row. This is
to ensure that the left and right arguments of binary operators have
meaningful row alignment.

These operators are particularly useful for building filtering
predicates and grouping columns.

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
            ...     split_evenly_over=1,
            ...     margins={(): dp.polars.Margin(max_partition_length=60_000_000 * 36)}
            ... )
            
            >>> query = (
            ...     context.query()
            ...     .filter((pl.col.HWUSUAL > 0) & (pl.col.HWUSUAL != 99))  # using the .gt, .and_ and .ne operators
            ...     .with_columns(OVER_40=pl.col.AGE > 40)
            ...     .group_by("SEX", "OVER_40")
            ...     .agg(dp.len())
            ... )
            >>> query.release().collect().sort("SEX", "OVER_40")
            shape: (4, 3)
            ┌─────┬─────────┬───────┐
            │ SEX ┆ OVER_40 ┆ len   │
            │ --- ┆ ---     ┆ ---   │
            │ i64 ┆ bool    ┆ u32   │
            ╞═════╪═════════╪═══════╡
            │ 1   ┆ false   ┆ ...   │
            │ 1   ┆ true    ┆ ...   │
            │ 2   ┆ false   ┆ ...   │
            │ 2   ┆ true    ┆ ...   │
            └─────┴─────────┴───────┘
