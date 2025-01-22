Aggregation
===========

[`Polars
Documentation <https://docs.pola.rs/api/python/stable/reference/expressions/aggregation.html>`__]

The most common aggregators like length, sum, mean, median and quantile
are covered in `essential
statistics <../../../../getting-started/tabular-data/essential-statistics.ipynb>`__.

In addition to these aggregators, OpenDP also supports other variations
of counting queries. A counting query tells you how many rows in a
dataset meet a given condition.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import polars as pl 
            >>> import opendp.prelude as dp
            
            >>> dp.enable_features("contrib")
            

To get started, we’ll recreate the Context from the `tabular data
introduction <../index.rst>`__.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=5,
            ... )
            

Frame Length vs Expression Length
---------------------------------

`Frame
length <https://docs.pola.rs/api/python/stable/reference/expressions/api/polars.len.html>`__
is not the same as `expression
length <https://docs.pola.rs/api/python/stable/reference/expressions/api/polars.Expr.len.html>`__.
These quantities can differ if the expression changes the number of
rows.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_len_variations = (
            ...     context.query()
            ...     .group_by("SEX")
            ...     .agg([
            ...         # total number of rows in the frame, including nulls
            ...         dp.len(),
            ...         # total number of rows in the HWUSUAL column (including nulls)
            ...         pl.col.HWUSUAL.dp.len(),
            ...     ])
            ...     # explicitly specifying keys makes the query satisfy pure-DP
            ...     .with_keys(pl.LazyFrame({"SEX": [1, 2]}))
            ... )
            >>> query_len_variations.summarize()
            shape: (2, 4)
            ┌─────────┬──────────────┬─────────────────┬───────┐
            │ column  ┆ aggregate    ┆ distribution    ┆ scale │
            │ ---     ┆ ---          ┆ ---             ┆ ---   │
            │ str     ┆ str          ┆ str             ┆ f64   │
            ╞═════════╪══════════════╪═════════════════╪═══════╡
            │ len     ┆ Frame Length ┆ Integer Laplace ┆ 360.0 │
            │ HWUSUAL ┆ Length       ┆ Integer Laplace ┆ 360.0 │
            └─────────┴──────────────┴─────────────────┴───────┘


These two statistics are equivalent, but the frame length (the first)
can be used to release stable grouping keys, while the column length
(the second) can be preprocessed with filtering.

The OpenDP Library will still use margin descriptors that may reduce the
sensitivity of the column length if it detects that the column has not
been transformed in a way that changes the number of rows.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_len_variations.release().collect()
            shape: (2, 3)
            ┌─────┬────────┬─────────┐
            │ SEX ┆ len    ┆ HWUSUAL │
            │ --- ┆ ---    ┆ ---     │
            │ i64 ┆ u32    ┆ u32     │
            ╞═════╪════════╪═════════╡
            ...
            └─────┴────────┴─────────┘


Unique Counts
-------------

A count of the number of unique values in a column is as sensitive as
the frame or column length when protecting user contributions. However,
unlike the frame length, the sensitivity does not reduce to zero when
protecting changed records, as a change in an individual’s answer may
result in one more, or one less, unique value.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_n_unique = context.query().select([
            ...     # total number of unique elements in the HWUSUAL column (including null)
            ...     pl.col.HWUSUAL.dp.n_unique(),
            ... ])
            >>> query_n_unique.summarize()
            shape: (1, 4)
            ┌─────────┬───────────┬─────────────────┬───────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale │
            │ ---     ┆ ---       ┆ ---             ┆ ---   │
            │ str     ┆ str       ┆ str             ┆ f64   │
            ╞═════════╪═══════════╪═════════════════╪═══════╡
            │ HWUSUAL ┆ N Unique  ┆ Integer Laplace ┆ 180.0 │
            └─────────┴───────────┴─────────────────┴───────┘

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_n_unique.release().collect()
            shape: (1, 1)
            ┌─────────┐
            │ HWUSUAL │
            │ ---     │
            │ u32     │
            ╞═════════╡
            │ ...     │
            └─────────┘


Noise added to a count can make the count go negative, but since the
output data type is an unsigned integer, the library may return zero.
This is more likely to happen with the true value is small.

This release tells us that the number of null values is relatively
small.

Null and Non-Null Counts
------------------------

You can release a count of the number of null or non-null records,
respectively, as follows:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_counts = context.query().select([
            ...     # total number of non-null elements in the HWUSUAL column
            ...     pl.col.HWUSUAL.dp.count(),
            ...     # total number of null elements in the HWUSUAL column
            ...     pl.col.HWUSUAL.dp.null_count(),
            ... ])
            >>> query_counts.summarize()
            shape: (2, 4)
            ┌─────────┬────────────┬─────────────────┬───────┐
            │ column  ┆ aggregate  ┆ distribution    ┆ scale │
            │ ---     ┆ ---        ┆ ---             ┆ ---   │
            │ str     ┆ str        ┆ str             ┆ f64   │
            ╞═════════╪════════════╪═════════════════╪═══════╡
            │ HWUSUAL ┆ Count      ┆ Integer Laplace ┆ 360.0 │
            │ HWUSUAL ┆ Null Count ┆ Integer Laplace ┆ 360.0 │
            └─────────┴────────────┴─────────────────┴───────┘


Notice that the ``count`` and ``null_count`` are complementary: you
could instead release ``len`` for ``HWUSUAL`` grouped by whether the
value is null.

You can take advantage of this to estimate both statistics with the same
privacy loss, but with half as much noise.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_counts_via_grouping = (
            ...     context.query()
            ...     .with_columns(pl.col("HWUSUAL").is_null().alias("HWUSUAL_is_null"))
            ...     .group_by("HWUSUAL_is_null")
            ...     .agg(dp.len())
            ...     # we're grouping on a bool column, so the groups are:
            ...     .with_keys(pl.LazyFrame({"HWUSUAL_is_null": [True, False]}))
            ... )
            >>> query_counts_via_grouping.summarize()
            shape: (1, 4)
            ┌────────┬──────────────┬─────────────────┬───────┐
            │ column ┆ aggregate    ┆ distribution    ┆ scale │
            │ ---    ┆ ---          ┆ ---             ┆ ---   │
            │ str    ┆ str          ┆ str             ┆ f64   │
            ╞════════╪══════════════╪═════════════════╪═══════╡
            │ len    ┆ Frame Length ┆ Integer Laplace ┆ 180.0 │
            └────────┴──────────────┴─────────────────┴───────┘

The noise scale dropped from 360 to 180…

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_counts_via_grouping.release().collect()
            shape: (2, 2)
            ┌─────────────────┬────────┐
            │ HWUSUAL_is_null ┆ len    │
            │ ---             ┆ ---    │
            │ bool            ┆ u32    │
            ╞═════════════════╪════════╡
            │ false           ┆ ...
            │ true            ┆ ...
            └─────────────────┴────────┘

…but we still get answers to all of the same queries!
