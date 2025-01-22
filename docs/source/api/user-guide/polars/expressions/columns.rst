Columns
=======

[`Polars
Documentation <https://docs.pola.rs/api/python/stable/reference/expressions/columns.html>`__]

``pl.col("A")`` or ``pl.col.A`` starts an expression by selecting a
column named “A”. While the Polars Library allows for multiple columns
to be selected simultaneously (via ``pl.col("*")``,
``pl.col("A", "B")``, ``pl.col(pl.String)``, ``pl.exclude``, and so on),
the OpenDP Library currently only supports selection of one column at a
time. The column name may be changed via ``.alias``.

Take for example the work hours dataset, where there are a collection of
columns labeled ``METHODX``, where ``X`` is an increasing alphabetic
sequence.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import polars as pl
            >>> import opendp.prelude as dp
            
            >>> dp.enable_features("contrib")
            
            >>> # not recommended, OpenDP will reject this joint expression over multiple columns
            >>> single_expr = pl.col([f"METHOD{l}" for l in "ABCDE"]).fill_null(0).dp.sum((0, 9))
            
            >>> # build individual expressions for each query
            >>> split_exprs = [pl.col(f"METHOD{l}").fill_null(0).dp.sum((0, 9)) for l in "ABCDE"]
            

Demonstration of use:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=1,
            ...     margins={(): dp.polars.Margin(max_partition_length=60_000_000 * 36)},
            ... )
            
            >>> context.query().select(split_exprs).release().collect()
            shape: (1, 5)
            ┌─────────┬─────────┬─────────┬─────────┬─────────┐
            │ METHODA ┆ METHODB ┆ METHODC ┆ METHODD ┆ METHODE │
            │ ---     ┆ ---     ┆ ---     ┆ ---     ┆ ---     │
            │ i64     ┆ i64     ┆ i64     ┆ i64     ┆ i64     │
            ╞═════════╪═════════╪═════════╪═════════╪═════════╡
            │ ...     ┆ ...     ┆ ...     ┆ ...     ┆ ...     │
            └─────────┴─────────┴─────────┴─────────┴─────────┘
