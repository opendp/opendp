.. _polars-sql-user-guide:

SQL
===

When your data is managed through the Polars-backed :ref:`Context API <context-user-guide>`,
you can build queries with SQL instead of chaining Polars methods.

The Context API entry point is :py:meth:`opendp.extras.polars.LazyFrameQuery.sql`,
which you access from ``context.query()``.
It registers the sensitive table under the name ``data`` by default.

.. code:: pycon

    >>> import polars as pl
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")
    >>> context = dp.Context.compositor(
    ...     data=pl.LazyFrame({
    ...         "a": [1.0] * 50,
    ...         "b": [1, 2, 3, 4, 5] * 10,
    ...     }),
    ...     privacy_unit=dp.unit_of(contributions=1),
    ...     privacy_loss=dp.loss_of(epsilon=1.0),
    ...     split_evenly_over=1,
    ...     margins=[dp.polars.Margin(by=["b"], max_length=50, max_groups=10, invariant="keys")],
    ... )
    >>> query = context.query().sql(
    ...     "SELECT b, dp_sum(a, 1.0, 2.0) AS total FROM data GROUP BY b"
    ... )
    >>> query.release().collect().sort("b")
    shape: (5, 2)
    ┌─────┬───────┐
    │ b   ┆ total │
    │ --- ┆ ---   │
    │ i64 ┆ f64   │
    ╞═════╪═══════╡
    │ 1   ┆ ...   │
    │ 2   ┆ ...   │
    │ 3   ┆ ...   │
    │ 4   ┆ ...   │
    │ 5   ┆ ...   │
    └─────┴───────┘

For lower-level use outside the Context API, call :py:func:`opendp.measurements.sql_to_plan`
to translate SQL into a Polars plan, then pass the resulting plan into
:py:func:`opendp.measurements.make_private_lazyframe`.
