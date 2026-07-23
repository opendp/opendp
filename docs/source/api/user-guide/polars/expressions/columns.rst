.. _columns:

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

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import polars as pl
            >>> import opendp.prelude as dp

            >>> dp.enable_features("contrib")

            >>> work_hours_cols = ["HWUSUAL", "HWACTUAL"]

            >>> # not recommended, OpenDP will reject this joint expression over multiple columns
            >>> single_expr = (
            ...     pl.col(work_hours_cols).cast(int).dp.sum((0, 60))
            ... )

            >>> # build individual expressions for each query
            >>> split_exprs = [
            ...     pl.col(c).cast(int).dp.sum((0, 60))
            ...     for c in work_hours_cols
            ... ]


Demonstration of use:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(
            ...         dp.examples.get_france_lfs_path(),
            ...     ),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=1,
            ...     margins=[dp.polars.Margin(max_length=150_000 * 36)],
            ... )

            >>> context.query().select(split_exprs).summarize()
            shape: (2, 4)
            ┌──────────┬───────────┬─────────────────┬────────┐
            │ column   ┆ aggregate ┆ distribution    ┆ scale  │
            │ ---      ┆ ---       ┆ ---             ┆ ---    │
            │ str      ┆ str       ┆ str             ┆ f64    │
            ╞══════════╪═══════════╪═════════════════╪════════╡
            │ HWUSUAL  ┆ Sum       ┆ Integer Laplace ┆ 4320.0 │
            │ HWACTUAL ┆ Sum       ┆ Integer Laplace ┆ 4320.0 │
            └──────────┴───────────┴─────────────────┴────────┘
