Manipulation
============

[`Polars
Documentation <https://docs.pola.rs/api/python/dev/reference/lazyframe/modify_select.html>`__]

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
            

Cast
----

When a Polars LazyFrame is passed to the Context API, the data schema is
read off of the dataframe. This means that in common usage, the OpenDP
Library considers the data schema to be public information, and that the
columns are already correctly typed.

While the OpenDP Library supports cast expressions, a drawback to their
usage is that cast expressions on grouping columns will void any margin
descriptors for those columns.

One setting where you may find cast expressions useful is when computing
a float sum on a large dataset. OpenDP accounts for inexact
floating-point arithmetic when computing the float sum, and on data with
large bounds and hundreds of thousands of records, this term can
dominate the sensitivity.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> (
            ...     context.query()
            ...     .select(pl.col.HWUSUAL.fill_null(0.0).fill_nan(0.0).dp.sum((0, 100)))
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌─────────┬───────────┬───────────────┬───────────────┐
            │ column  ┆ aggregate ┆ distribution  ┆ scale         │
            │ ---     ┆ ---       ┆ ---           ┆ ---           │
            │ str     ┆ str       ┆ str           ┆ f64           │
            ╞═════════╪═══════════╪═══════════════╪═══════════════╡
            │ HWUSUAL ┆ Sum       ┆ Float Laplace ┆ 843177.046991 │
            └─────────┴───────────┴───────────────┴───────────────┘


Casting to integers avoids this term, resulting in a much smaller noise
scale to satisfy the same level of privacy.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> context.query().select(pl.col.HWUSUAL.cast(int).fill_null(0).dp.sum((0, 100))).summarize()
            shape: (1, 4)
            ┌─────────┬───────────┬─────────────────┬─────────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale   │
            │ ---     ┆ ---       ┆ ---             ┆ ---     │
            │ str     ┆ str       ┆ str             ┆ f64     │
            ╞═════════╪═══════════╪═════════════════╪═════════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 14400.0 │
            └─────────┴───────────┴─────────────────┴─────────┘


The OpenDP Library forces that failed casts do not throw a
(data-dependent) exception, instead returning a null. Therefore using
this cast operation updates the output domain to indicate that there may
potentially be nulls. You’ll probably need to apply ``.fill_null``
before computing statistics with casted data.

Clip
----

Computing the sum and mean privately requires input data to be
restricted between some lower and upper bound. DP expressions like
``.dp.sum`` and ``.dp.mean`` automatically insert a ``.clip`` expression
based on given data bounds. However, a ``.clip`` transformation may be
used anywhere, and it will establish a domain descriptor for the column
being clipped. When an aggregation is conducted, the library will check
for the presence of this descriptor if it is necessary to bound the
sensitivity of the query.

This is demonstrated in the following query, where the preprocessing is
broken apart into different data processing phases.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> (
            ...     context.query()
            ...     .with_columns(pl.col.HWUSUAL.cast(int).fill_null(0).clip(0, 100))
            ...     .select(pl.col.HWUSUAL.sum().dp.noise())
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌─────────┬───────────┬─────────────────┬─────────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale   │
            │ ---     ┆ ---       ┆ ---             ┆ ---     │
            │ str     ┆ str       ┆ str             ┆ f64     │
            ╞═════════╪═══════════╪═════════════════╪═════════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 14400.0 │
            └─────────┴───────────┴─────────────────┴─────────┘


Cut
---

Cut is a transformation that bins numerical data according to a list of
breaks. The following example releases counts of the number of
individuals working each hour range.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> breaks = [0, 20, 40, 60, 80, 98]
            
            >>> query = (
            ...     context.query()
            ...     .with_columns(pl.col.HWUSUAL.cut(breaks=breaks))
            ...     .group_by("HWUSUAL")
            ...     .agg(dp.len())
            ... )
            >>> query.release().collect().sort("HWUSUAL")
            shape: (4, 2)
            ┌───────────┬────────┐
            │ HWUSUAL   ┆ len    │
            │ ---       ┆ ---    │
            │ cat       ┆ u32    │
            ╞═══════════╪════════╡
            │ ...       ┆ ...    │
            └───────────┴────────┘

        .. Different sets of buckets are returns on successive runs.


In this setting it is not necessary to spend an additional
:math:`\delta` parameter to privately release the keys. Instead we can
construct an explicit key set based on the bin labels from grouping:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> def cut_labels(breaks, left_closed=False):
            ...     edges = ["-inf", *breaks, "inf"]
            ...     bl, br = ("[", ")") if left_closed else ("(", "]")
            ...     return [f"{bl}{l}, {r}{br}" for l, r in zip(edges[:-1], edges[1:])]
            
            >>> labels = pl.Series("HWUSUAL", cut_labels(breaks), dtype=pl.Categorical)
            
            >>> query = (
            ...     context.query()
            ...     .with_columns(pl.col.HWUSUAL.cut(breaks=breaks))
            ...     .group_by("HWUSUAL")
            ...     .agg(dp.len())
            ...     .with_keys(pl.LazyFrame([labels]))
            ... )
            >>> query.summarize()
            shape: (1, 4)
            ┌────────┬──────────────┬─────────────────┬───────┐
            │ column ┆ aggregate    ┆ distribution    ┆ scale │
            │ ---    ┆ ---          ┆ ---             ┆ ---   │
            │ str    ┆ str          ┆ str             ┆ f64   │
            ╞════════╪══════════════╪═════════════════╪═══════╡
            │ len    ┆ Frame Length ┆ Integer Laplace ┆ 144.0 │
            └────────┴──────────────┴─────────────────┴───────┘

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query.release().collect().sort("HWUSUAL")
                shape: (7, 2)
                ┌───────────┬────────┐
                │ HWUSUAL   ┆ len    │
                │ ---       ┆ ---    │
                │ cat       ┆ u32    │
                ╞═══════════╪════════╡
                │ (-inf, 0] ┆ ...    │
                │ (0, 20]   ┆ ...    │
                │ (20, 40]  ┆ ...    │
                │ (40, 60]  ┆ ...    │
                │ (60, 80]  ┆ ...    │
                │ (80, 98]  ┆ ...    │
                │ (98, inf] ┆ ...    │
                └───────────┴────────┘


The output type is categorical, but with a data-independent encoding,
meaning OpenDP allows grouping by these keys.

Fill NaN
--------

``.fill_nan`` replaces NaN float values. Not to be confused with
``.fill_null``. The output data is only considered non-nan if the fill
expression is both non-null and non-nan.

In common use throughout the documentation, the fill value has been
simply a single scalar, but more complicated expressions are valid:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> (
            ...     context.query()
            ...     # prepare actual work hours as a valid fill column
            ...     .with_columns(pl.col.HWACTUAL.fill_nan(0.0).fill_null(0.0))
            ...     # prepare usual work hours with actual work hours as a fill
            ...     .with_columns(pl.col.HWUSUAL.fill_nan(pl.col.HWACTUAL).fill_null(pl.col.HWACTUAL))
            ...     # compute the dp sum
            ...     .select(pl.col.HWUSUAL.dp.sum((0, 100)))
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌─────────┬───────────┬───────────────┬───────────────┐
            │ column  ┆ aggregate ┆ distribution  ┆ scale         │
            │ ---     ┆ ---       ┆ ---           ┆ ---           │
            │ str     ┆ str       ┆ str           ┆ f64           │
            ╞═════════╪═══════════╪═══════════════╪═══════════════╡
            │ HWUSUAL ┆ Sum       ┆ Float Laplace ┆ 843177.046991 │
            └─────────┴───────────┴───────────────┴───────────────┘


At this time ``.fill_nan`` always drops data bounds, so make sure your
data is non-nan before running ``.clip``.

Even if you are in an aggregation context like ``.select`` or ``.agg``,
OpenDP enforces that inputs to ``.fill_nan`` are row-by-row. This is to
ensure that the left and right arguments of binary operators have
meaningful row alignment, and that inputs share the same number of
records, to avoid data-dependent errors that would violate the privacy
guarantee.

Fill Null
---------

``.fill_null`` replaces null values. Not to be confused with
``.fill_nan``. All data types in Polars may be null. The output data is
only considered non-null if the fill expression is non-null.

In common use throughout the documentation, the fill value has been
simply a single scalar, but more complicated expressions are valid:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> (
            ...     context.query()
            ...     # prepare actual work hours as a valid fill column
            ...     .with_columns(pl.col.HWACTUAL.cast(int).fill_null(0.0))
            ...     # prepare usual work hours with actual work hours as a fill
            ...     .with_columns(pl.col.HWUSUAL.cast(int).fill_null(pl.col.HWACTUAL))
            ...     # compute the dp sum
            ...     .select(pl.col.HWUSUAL.dp.sum((0, 100)))
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌─────────┬───────────┬─────────────────┬─────────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale   │
            │ ---     ┆ ---       ┆ ---             ┆ ---     │
            │ str     ┆ str       ┆ str             ┆ f64     │
            ╞═════════╪═══════════╪═════════════════╪═════════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 14400.0 │
            └─────────┴───────────┴─────────────────┴─────────┘
            

At this time ``.fill_null`` always drops data bounds, so make sure your
data is non-null before running ``.clip``.

Just like ``.fill_nan``, even if you are in an aggregation context like
``.select`` or ``.agg``, OpenDP enforces that inputs to ``.fill_nan``
are row-by-row.

To Physical
-----------

``.to_physical`` returns the underlying data representation categorical
(``pl.Categorical``) or temporal (``pl.Date``, ``pl.Time``,
``pl.Datetime``) data types. For example, you can use the
``.to_physical`` expression to retrieve the bin indices of the ``.cut``
expression.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> breaks = [0, 20, 40, 60, 80, 98]
            >>> labels = pl.Series("HWUSUAL", list(range(len(breaks) + 1)), dtype=pl.UInt32)
            
            >>> query = (
            ...     context.query()
            ...     .with_columns(pl.col.HWUSUAL.cut(breaks=breaks).to_physical())
            ...     .group_by("HWUSUAL")
            ...     .agg(dp.len())
            ...     .with_keys(pl.LazyFrame([labels]))
            ... )
            >>> query.release().collect().sort("HWUSUAL")
            shape: (7, 2)
            ┌─────────┬────────┐
            │ HWUSUAL ┆ len    │
            │ ---     ┆ ---    │
            │ u32     ┆ u32    │
            ╞═════════╪════════╡
            │ 0       ┆ ...    │
            │ 1       ┆ ...    │
            │ 2       ┆ ...    │
            │ 3       ┆ ...    │
            │ 4       ┆ ...    │
            │ 5       ┆ ...    │
            │ 6       ┆ ...    │
            └─────────┴────────┘


In the case of categorical data types, OpenDP only allows this
expression if the encoding is data-independent. More information can be
found in `Data Types <../data-types.ipynb>`__.
