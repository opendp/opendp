Manipulation
============

[`Polars
Documentation <https://docs.pola.rs/api/python/dev/reference/lazyframe/modify_select.html>`__]

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

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

        .. code:: pycon

            >>> (
            ...     context.query()
            ...     .select(pl.col.HWUSUAL.dp.sum((0, 100)))
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌─────────┬───────────┬───────────────┬──────────────┐
            │ column  ┆ aggregate ┆ distribution  ┆ scale        │
            │ ---     ┆ ---       ┆ ---           ┆ ---          │
            │ str     ┆ str       ┆ str           ┆ f64          │
            ╞═════════╪═══════════╪═══════════════╪══════════════╡
            │ HWUSUAL ┆ Sum       ┆ Float Laplace ┆ 14405.179857 │
            └─────────┴───────────┴───────────────┴──────────────┘


Casting to integers avoids this term, resulting in a much smaller noise
scale to satisfy the same level of privacy.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> context.query().select(pl.col.HWUSUAL.cast(int).dp.sum((0, 100))).summarize()
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

Computing the differentially private sum and mean requires input data to
be restricted between some lower and upper bound. DP expressions like
``.dp.sum`` and ``.dp.mean`` automatically insert imputation when
necessary and ``.clip`` based on given data bounds. However, a ``.clip``
transformation may be used anywhere, and it will establish a domain
descriptor for the column being clipped. When an aggregation is
conducted, the library will check for the presence of this descriptor if
it is necessary to bound the sensitivity of the query.

This is demonstrated in the following query, where the preprocessing is
broken apart into different data processing phases.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

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

        .. code:: pycon

            >>> breaks = [0, 20, 40, 60, 80, 98]
            
            >>> query = (
            ...     context.query()
            ...     .with_columns(pl.col.HWUSUAL.cut(breaks=breaks))
            ...     .group_by("HWUSUAL")
            ...     .agg(dp.len())
            ... )
            >>> query.release().collect().sort("HWUSUAL")
            shape: (7, 2)
            ┌───────────┬─────────┐
            │ HWUSUAL   ┆ len     │
            │ ---       ┆ ---     │
            │ cat       ┆ u32     │
            ╞═══════════╪═════════╡
            │ null      ┆ ... │
            │ (-inf, 0] ┆ ... │
            │ (0, 20]   ┆ ... │
            │ (20, 40]  ┆ ... │
            │ (40, 60]  ┆ ... │
            │ (60, 80]  ┆ ... │
            │ (98, inf] ┆ ... │
            └───────────┴─────────┘


In this setting it is not necessary to spend an additional
:math:`\delta` parameter to release the keys with differential privacy.
Instead we can construct an explicit key set based on the bin labels
from grouping:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

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

        .. code:: pycon

            >>> query.release().collect().sort("HWUSUAL")
            shape: (7, 2)
            ┌───────────┬─────────┐
            │ HWUSUAL   ┆ len     │
            │ ---       ┆ ---     │
            │ cat       ┆ u32     │
            ╞═══════════╪═════════╡
            │ (-inf, 0] ┆ ... │
            │ (0, 20]   ┆ ... │
            │ (20, 40]  ┆ ... │
            │ (40, 60]  ┆ ... │
            │ (60, 80]  ┆ ... │
            │ (80, 98]  ┆ ... │
            │ (98, inf] ┆ ... │
            └───────────┴─────────┘


The output type is categorical, but with a data-independent encoding,
meaning OpenDP allows grouping by these keys.

Drop NaNs
---------

``.drop_nans`` removes NaN float values from the data. Not to be
confused with ``.drop_nulls``. While this guarantees that the data does
not have NaNs, it also drops data invariants on dataset size. This
expression may not be used in a row-by-row context.

See ``.drop_nulls`` for a usage example.

Drop Nulls
----------

``.drop_nulls`` removes null values from the data. Not to be confused
with ``.drop_nans`` for float data. While this guarantees that the data
does not have nulls, it also drops data invariants on dataset size. This
expression may not be used in a row-by-row context.

In the following example, all nans and nulls are dropped from the
``HWUSUAL`` column.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> (
            ...     context.query()
            ...     .select(pl.col.HWUSUAL.drop_nans().drop_nulls().dp.sum((0, 100)))
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌─────────┬───────────┬───────────────┬──────────────┐
            │ column  ┆ aggregate ┆ distribution  ┆ scale        │
            │ ---     ┆ ---       ┆ ---           ┆ ---          │
            │ str     ┆ str       ┆ str           ┆ f64          │
            ╞═════════╪═══════════╪═══════════════╪══════════════╡
            │ HWUSUAL ┆ Sum       ┆ Float Laplace ┆ 14405.179857 │
            └─────────┴───────────┴───────────────┴──────────────┘


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

        .. code:: pycon

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
            ┌─────────┬───────────┬───────────────┬──────────────┐
            │ column  ┆ aggregate ┆ distribution  ┆ scale        │
            │ ---     ┆ ---       ┆ ---           ┆ ---          │
            │ str     ┆ str       ┆ str           ┆ f64          │
            ╞═════════╪═══════════╪═══════════════╪══════════════╡
            │ HWUSUAL ┆ Sum       ┆ Float Laplace ┆ 14405.179857 │
            └─────────┴───────────┴───────────────┴──────────────┘


If the fill expression is not a scalar literal, then the input
expression and fill expression must both by row-by-row. This is to
prevent a mismatch in lengths that results in data-dependent errors.

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

        .. code:: pycon

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


If the fill expression is not a scalar literal, then the input
expression and fill expression must both by row-by-row. This is to
prevent a mismatch in lengths that results in data-dependent errors.

At this time ``.fill_null`` always drops data bounds, so make sure your
data is non-null before running ``.clip``.

Just like ``.fill_nan``, even if you are in an aggregation context like
``.select`` or ``.agg``, OpenDP enforces that inputs to ``.fill_nan``
are row-by-row.

Filter
------

In addition to the full-dataframe filter, you can also filter
expressions in contexts with aggregators. The filter is not row-by-row,
but it requires that its inputs are row-by-row; this is to ensure the
data and predicate match.

The following query counts the number of respondents over retirement
age:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> (
            ...     context.query()
            ...     .with_columns(pl.col.HWUSUAL.fill_nan(0).fill_null(0))
            ...     # compute the dp sum of individuals over 64 years old
            ...     .select(pl.col.HWUSUAL.filter(pl.col.AGE > 64).dp.sum((0, 100)))
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌─────────┬───────────┬───────────────┬──────────────┐
            │ column  ┆ aggregate ┆ distribution  ┆ scale        │
            │ ---     ┆ ---       ┆ ---           ┆ ---          │
            │ str     ┆ str       ┆ str           ┆ f64          │
            ╞═════════╪═══════════╪═══════════════╪══════════════╡
            │ HWUSUAL ┆ Sum       ┆ Float Laplace ┆ 14405.179857 │
            └─────────┴───────────┴───────────────┴──────────────┘


This is a setting where the query can be answered via a group by
instead, with the benefit of also getting a count of the number of
individuals less than 65:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> (
            ...     context.query()
            ...     .group_by(pl.col.AGE > 64)
            ...     .agg(dp.len())
            ...     .with_keys(pl.LazyFrame({"AGE": [True, False]}))
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌────────┬──────────────┬─────────────────┬───────┐
            │ column ┆ aggregate    ┆ distribution    ┆ scale │
            │ ---    ┆ ---          ┆ ---             ┆ ---   │
            │ str    ┆ str          ┆ str             ┆ f64   │
            ╞════════╪══════════════╪═════════════════╪═══════╡
            │ len    ┆ Frame Length ┆ Integer Laplace ┆ 144.0 │
            └────────┴──────────────┴─────────────────┴───────┘


OpenDP doesn’t infer descriptors from the filtering criteria, so the
filtered data is not considered non-null (or non-NaN) even if you filter
nulls (or NaNs). In the following code snip, that preprocessing is added
by ``.dp.sum``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> (
            ...     context.query()
            ...     .with_columns(pl.col.HWUSUAL.fill_nan(0))
            ...     # compute the dp sum of individuals over 64 years old
            ...     .select(pl.col.HWUSUAL.filter(pl.col.HWUSUAL.is_not_null()).dp.sum((0, 98)))
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌─────────┬───────────┬───────────────┬──────────────┐
            │ column  ┆ aggregate ┆ distribution  ┆ scale        │
            │ ---     ┆ ---       ┆ ---           ┆ ---          │
            │ str     ┆ str       ┆ str           ┆ f64          │
            ╞═════════╪═══════════╪═══════════════╪══════════════╡
            │ HWUSUAL ┆ Sum       ┆ Float Laplace ┆ 14117.076259 │
            └─────────┴───────────┴───────────────┴──────────────┘


In this query, ``.fill_null`` has a broadcast-able filler, so it doesn’t
require its input to be row-by-row. Since the input does not need to be
row-by-row, OpenDP accepts the filter.

Replace
-------

``replace`` replaces all ``old`` values with the respective ``new``
values. Useful for recoding, in cases where you want to preserve the
same data type. The following recodes ``99`` and null in the ``HWUSUAL``
column to ``0``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> (
            ...     context.query()
            ...     .select(
            ...         pl.col.ILOSTAT.cast(int)
            ...         .replace(old=[99, None], new=0) # replace 99 and None with 0
            ...         .dp.sum((0, 98))
            ...     )
            ...     .summarize()
            ... )
            shape: (1, 4)
            ┌─────────┬───────────┬─────────────────┬─────────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale   │
            │ ---     ┆ ---       ┆ ---             ┆ ---     │
            │ str     ┆ str       ┆ str             ┆ f64     │
            ╞═════════╪═══════════╪═════════════════╪═════════╡
            │ ILOSTAT ┆ Sum       ┆ Integer Laplace ┆ 14112.0 │
            └─────────┴───────────┴─────────────────┴─────────┘
            

When passed ``old`` and ``new`` arguments, ``new`` can either match the
length of ``old``, or be a single value that is then broadcast.

Alternatively, ``replace`` can be passed a dictionary, where the keys
are ``old`` and values are ``new``. An example of this is shown for
``replace_strict``, which has a very similar API.

If ``old`` contains null, and ``new`` doesn’t, then this expression
considers the output domain to not contain nulls.

Replace cannot be used on categorical data because Polars raises a
data-dependent `categorical remapping
warning <https://docs.pola.rs/api/python/stable/reference/api/polars.exceptions.CategoricalRemappingWarning.html>`__.
OpenDP is also “pickier” about the data type of the input than Polars
is: OpenDP will reject the query if fallible casting of old or new
values is necessary.

Replace Strict
--------------

Unlike ``replace``, which just replaces the specified ``old`` values,
``replace_strict`` replaces all values in a column. Since
``replace_strict`` replaces all values, the data type of the output may
differ from that of the input.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> ilostat_labels = {
            ...     1: "Working for pay or profit",
            ...     2: "Employed but not working",
            ...     3: "Unemployed",
            ...     9: "Not in labor force",
            ... }
            
            >>> (
            ...     context.query()
            ...     .group_by(pl.col.ILOSTAT.replace_strict(ilostat_labels))
            ...     .agg(dp.len())
            ...     .summarize()
            ... )
            shape: (1, 5)
            ┌────────┬──────────────┬─────────────────┬───────┬───────────┐
            │ column ┆ aggregate    ┆ distribution    ┆ scale ┆ threshold │
            │ ---    ┆ ---          ┆ ---             ┆ ---   ┆ ---       │
            │ str    ┆ str          ┆ str             ┆ f64   ┆ u32       │
            ╞════════╪══════════════╪═════════════════╪═══════╪═══════════╡
            │ len    ┆ Frame Length ┆ Integer Laplace ┆ 144.0 ┆ 2973      │
            └────────┴──────────────┴─────────────────┴───────┴───────────┘


Values that are not in ``old`` are filled with ``default``. If a value
is not in ``old`` and ``default`` is not specified, Polars raises a
data-dependent error. To protect against this, OpenDP changes
``default`` to null if ``default`` is not specified.

If the new values don’t contain null, and a non-null default value is
supplied, then OpenDP considers the output to be non-null.

To Physical
-----------

``.to_physical`` returns the underlying representation of categorical
(``pl.Categorical``, ``pl.Enum``) or temporal (``pl.Date``, ``pl.Time``,
``pl.Datetime``) data. For example, you can use the ``.to_physical``
expression to retrieve the bin indices of the ``.cut`` expression.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

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
            ┌─────────┬─────────┐
            │ HWUSUAL ┆ len     │
            │ ---     ┆ ---     │
            │ u32     ┆ u32     │
            ╞═════════╪═════════╡
            │ 0       ┆ ... │
            │ 1       ┆ ... │
            │ 2       ┆ ... │
            │ 3       ┆ ... │
            │ 4       ┆ ... │
            │ 5       ┆ ... │
            │ 6       ┆ ... │
            └─────────┴─────────┘


In the case of categorical data types, OpenDP only allows this
expression if the encoding is data-independent. More information can be
found in `Data Types <../data-types.ipynb>`__.
