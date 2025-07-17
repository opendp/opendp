Identifier Truncation
=====================

OpenDP can be used to query datasets where each individual may
contribute an unbounded number of records, but where all records
contributed by an individual share the same identifier.

The user identifier is a part of the privacy unit (the privacy unit
quantifies the influence an individual may have on the data).

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import opendp.prelude as dp
            >>> import polars as pl
            
            >>> # the PIDENT column contains individual identifiers
            >>> # an individual may contribute data under at most 1 PIDENT identifier
            >>> privacy_unit = dp.unit_of(contributions=1, identifier=pl.col("PIDENT"))
            

This ``privacy_unit`` consists of all records associated with any one
unique identifier in the ``PIDENT`` column. OpenDP allows identifiers to
be arbitrary Polars expressions. The identifier expression must be
row-by-row to be well-defined.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> dp.enable_features("contrib")
            
            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=privacy_unit,
            ...     privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-8),
            ...     split_evenly_over=4,
            ...     margins=[dp.polars.Margin(max_length=150_000 * 36)],
            ... )
            

Truncating Per-Group Contributions
----------------------------------

In order to make differentially private releases on this data, an
additional identifier truncation step is necessary, where only a limited
number of records corresponding to each identifier are kept.

Under the assumption that it unlikely that an individual is chosen for
the survey more than ten times, the following query limits the number of
contributions to ten.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     .truncate_per_group(10)
            ...     # ...is equivalent to:
            ...     # .filter(pl.int_range(pl.len()).over("PIDENT") < 10)
            ...     .select(pl.col.HWUSUAL.cast(int).fill_null(0).dp.mean((0, 80)))
            ... )
            >>> query.summarize()
            shape: (2, 4)
            ┌─────────┬───────────┬─────────────────┬────────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale  │
            │ ---     ┆ ---       ┆ ---             ┆ ---    │
            │ str     ┆ str       ┆ str             ┆ f64    │
            ╞═════════╪═══════════╪═════════════════╪════════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 6400.0 │
            │ HWUSUAL ┆ Length    ┆ Integer Laplace ┆ 80.0   │
            └─────────┴───────────┴─────────────────┴────────┘


Previous examples with this dataset assumed the worst-case of 36
contributed records per individual (one contribution per quarter for
nine years) which resulted in a 36-time increase in the amount of noise.
By truncating to at most ten records, there is only a 10-fold increase
in the amount of noise. This statistical estimator is introducing some
bias by dropping records from individuals who contributed more than ten
records, at the benefit of attaining a much lower variance.

By default, ``truncate_per_group`` takes a random sample of records
per-identifier, per-group. To choose which records you’d like to keep,
you can also set ``keep`` to ``first``, ``last``, or an instance of
``SortBy``. ``first`` is the most computationally efficient, but may
bias your estimates if natural order is significant. The following
demonstrates the sort, which prefers records with lower ``ILOSTAT``
status, when the individual worked for pay or profit.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     .truncate_per_group(10, keep=dp.polars.SortBy(pl.col("ILOSTAT")))
            ...     # ...is equivalent to:
            ...     # .filter(pl.int_range(pl.len()).sort_by(pl.col.ILOSTAT).over("PIDENT") < 10)
            ...     .select(pl.col.HWUSUAL.cast(int).fill_null(0).dp.mean((0, 80)))
            ... )
            >>> query.summarize()
            shape: (2, 4)
            ┌─────────┬───────────┬─────────────────┬────────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale  │
            │ ---     ┆ ---       ┆ ---             ┆ ---    │
            │ str     ┆ str       ┆ str             ┆ f64    │
            ╞═════════╪═══════════╪═════════════════╪════════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 6400.0 │
            │ HWUSUAL ┆ Length    ┆ Integer Laplace ┆ 80.0   │
            └─────────┴───────────┴─────────────────┴────────┘


See the API documentation for
```truncate_per_group`` <../../api/python/opendp.extras.polars.html#opendp.extras.polars.LazyFrameQuery.truncate_per_group>`__
for more informaton on configuring sorting.

In this case, when computing the mean, an even better approach is to
group by the identifier and aggregate down to one row, before computing
the statistics of interest.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     .group_by(pl.col.PIDENT) # truncation begins here
            ...     .agg(pl.col.HWUSUAL.mean()) # arbitrary expressions can be used here
            ...     .select(pl.col.HWUSUAL.cast(int).fill_null(0).dp.mean((0, 80)))
            ... )
            >>> query.summarize()
            shape: (2, 4)
            ┌─────────┬───────────┬─────────────────┬───────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale │
            │ ---     ┆ ---       ┆ ---             ┆ ---   │
            │ str     ┆ str       ┆ str             ┆ f64   │
            ╞═════════╪═══════════╪═════════════════╪═══════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 640.0 │
            │ HWUSUAL ┆ Length    ┆ Integer Laplace ┆ 8.0   │
            └─────────┴───────────┴─────────────────┴───────┘


This reduces the sensitivity even further, resulting in no increase to
the noise scale, despite a potentially unlimited number of user
contributions.

Truncating Contributed Groups
-----------------------------

To release queries that involve grouping, it is also necessary to bound
the number of groups an individual may contribute to, not just the
number of contributions per-group.

The following query introduces a second truncation that also limits the
number of records per quarter.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> quarterly = [pl.col.QUARTER, pl.col.YEAR]
            >>> query = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     .truncate_per_group(1, by=quarterly)
            ...     # ...is equivalent to:
            ...     # .filter(pl.int_range(pl.len()).over("PIDENT", *quarterly) < 1)
            ...     .truncate_num_groups(10, by=quarterly)
            ...     # ...is roughly equivalent to:
            ...     # .filter(pl.struct(*quarterly).rank("dense").over("PIDENT") < 10)
            ...     .group_by(quarterly)
            ...     .agg(dp.len(), pl.col.HWUSUAL.cast(int).fill_null(0).dp.sum((0, 80)))
            ... )
            >>> query.summarize()
            shape: (2, 5)
            ┌─────────┬──────────────┬─────────────────┬────────┬───────────┐
            │ column  ┆ aggregate    ┆ distribution    ┆ scale  ┆ threshold │
            │ ---     ┆ ---          ┆ ---             ┆ ---    ┆ ---       │
            │ str     ┆ str          ┆ str             ┆ f64    ┆ u32       │
            ╞═════════╪══════════════╪═════════════════╪════════╪═══════════╡
            │ len     ┆ Frame Length ┆ Integer Laplace ┆ 80.0   ┆ 1714      │
            │ HWUSUAL ┆ Sum          ┆ Integer Laplace ┆ 6400.0 ┆ null      │
            └─────────┴──────────────┴─────────────────┴────────┴───────────┘


By default, ``truncate_num_groups`` takes a random sample of groups
per-identifier. To choose which groups you’d like to keep, you can also
set ``keep`` to ``first`` or ``last``. ``first`` and ``last`` should be
more computationally efficient, but may bias your estimates if natural
order is significant.

OpenDP allows queries to contain multiple truncations, so long as they
are together in the data pipeline. OpenDP does, however, enforce that
group by truncations are the last truncation in the data pipeline.

Privacy Unit with Multiple Bounds
---------------------------------

It is also possible to set more fine-grained bounds on user identifier
contributions across different levels of grouping.

Take, for example, the case where your data comes from two different
sources, spanning different years. This means individuals could
contribute data under two user identifiers, which would double the
amount of noise. However, due to the structure of the data, you know
that each individual only ever contributes data under one identifier
each quarter.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=dp.unit_of(contributions=[
            ...         # an individual may contribute data under up to 2 identifiers
            ...         dp.polars.Bound(per_group=2),
            ...         # ...but only under 1 identifier each quarter
            ...         dp.polars.Bound(by=quarterly, per_group=1),
            ...     ], identifier="PIDENT"),
            ...     privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-8),
            ...     split_evenly_over=4,
            ...     margins=[dp.polars.Margin(max_length=150_000 * 36)],
            ... )
            
            >>> query = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     .truncate_per_group(1, by=quarterly)
            ...     .truncate_num_groups(5, by=quarterly) # each identifier may affect up to 5 groups
            ...     .group_by(quarterly)
            ...     .agg(dp.len(), pl.col.HWUSUAL.cast(int).fill_null(0).dp.sum((0, 80)))
            ... )
            >>> query.summarize()
            shape: (2, 5)
            ┌─────────┬──────────────┬─────────────────┬────────┬───────────┐
            │ column  ┆ aggregate    ┆ distribution    ┆ scale  ┆ threshold │
            │ ---     ┆ ---          ┆ ---             ┆ ---    ┆ ---       │
            │ str     ┆ str          ┆ str             ┆ f64    ┆ u32       │
            ╞═════════╪══════════════╪═════════════════╪════════╪═══════════╡
            │ len     ┆ Frame Length ┆ Integer Laplace ┆ 80.0   ┆ 1714      │
            │ HWUSUAL ┆ Sum          ┆ Integer Laplace ┆ 6400.0 ┆ null      │
            └─────────┴──────────────┴─────────────────┴────────┴───────────┘


This ensures the privacy unit is still accurately modeled, while
preserving the expected utility.

It is also possible to set an upper bound on the number of groups a user
may influence in the same way.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> bound = dp.polars.Bound(by=quarterly, num_groups=10)
            

However, the general recommendation and best practice is to truncate—
and not set distance bounds in the context, unless you need to. This is
because, if there is an individual with greater influence than expected,
their privacy loss can exceed the privacy guarantee. The same logic
applies for other preprocessing, like clipping, where it is best
practice to clip the data, and not set bounds on the data in the input
domain.
