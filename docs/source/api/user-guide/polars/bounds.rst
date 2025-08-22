.. _bounds-user-guide:

Identifier Truncation and Bounds
================================

It's also important to be mindful of the structure of our data
when thinking about identifier truncation and bounds.
This is another area where there are opportunities to lower the
sensitivity of the analysis to individual contributions, and hence the noise required.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> import polars as pl
            >>> dp.enable_features("contrib")

            >>> # The PIDENT column contains individual identifiers.
            >>> # An individual may contribute data under at most 1 PIDENT identifier.
            >>> privacy_unit = dp.unit_of(
            ...     contributions=1, identifier=pl.col("PIDENT")
            ... )
            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(
            ...         dp.examples.get_france_lfs_path(),
            ...         ignore_errors=True,
            ...     ),
            ...     privacy_unit=privacy_unit,
            ...     privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-8),
            ...     split_evenly_over=4,
            ...     margins=[dp.polars.Margin(max_length=150_000 * 36)],
            ... )

By default, ``truncate_per_group`` takes a random sample of records
per-identifier, per-group. To choose which records you’d like to keep,
you can also set ``keep`` to ``'first'``, ``'last'``, or an instance of
``SortBy``. ``'first'`` is the most computationally efficient, but may
bias your estimates if natural order is significant. The following
demonstrates the sort, which prefers records with lower ``ILOSTAT``
status, when the individual worked for pay or profit.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     .truncate_per_group(
            ...         10, keep=dp.polars.SortBy(pl.col("ILOSTAT"))
            ...     )
            ...     # ...is equivalent to:
            ...     # .filter(pl.int_range(pl.len()).sort_by(pl.col.ILOSTAT).over("PIDENT") < 10)
            ...     .select(
            ...         pl.col.HWUSUAL.cast(int)
            ...         .fill_null(0)
            ...         .dp.mean((0, 80))
            ...     )
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

In this case, when computing the mean, an even better approach is to
group by the identifier and aggregate down to one row, before computing
the statistics of interest.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     .group_by(pl.col.PIDENT)  # truncation begins here
            ...     .agg(
            ...         pl.col.HWUSUAL.mean()
            ...     )  # arbitrary expressions can be used here
            ...     .select(
            ...         pl.col.HWUSUAL.cast(int)
            ...         .fill_null(0)
            ...         .dp.mean((0, 80))
            ...     )
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

Privacy Unit with Multiple Bounds
---------------------------------

It is also possible to set more fine-grained bounds on user identifier
contributions across different levels of grouping.

Imagine that your data comes from two different
sources, spanning different years. This means individuals could
contribute data under two user identifiers, which would double the
amount of noise. However, if you know
that each individual only ever contributes data under one identifier
each quarter, you can take this into account in your annalysis.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> quarterly = [pl.col.QUARTER, pl.col.YEAR]
            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(
            ...         dp.examples.get_france_lfs_path(),
            ...         ignore_errors=True,
            ...     ),
            ...     privacy_unit=dp.unit_of(
            ...         contributions=[
            ...             # an individual may contribute data under up to 2 identifiers
            ...             dp.polars.Bound(per_group=2),
            ...             # ...but only under 1 identifier each quarter
            ...             dp.polars.Bound(by=quarterly, per_group=1),
            ...         ],
            ...         identifier="PIDENT",
            ...     ),
            ...     privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-8),
            ...     split_evenly_over=4,
            ...     margins=[dp.polars.Margin(max_length=150_000 * 36)],
            ... )

            >>> query = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99)
            ...     .truncate_per_group(1, by=quarterly)
            ...     .truncate_num_groups(
            ...         5, by=quarterly
            ...     )  # each identifier may affect up to 5 groups
            ...     .group_by(quarterly)
            ...     .agg(
            ...         dp.len(),
            ...         pl.col.HWUSUAL.cast(int)
            ...         .fill_null(0)
            ...         .dp.sum((0, 80)),
            ...     )
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

        .. code:: pycon

            >>> bound = dp.polars.Bound(by=quarterly, num_groups=10)


However, the general recommendation and best practice is to truncate—
and not set distance bounds in the context, unless you need to. This is
because, if there is an individual with greater influence than expected,
their privacy loss can exceed the privacy guarantee. The same logic
applies for other preprocessing, like clipping, where it is best
practice to clip the data, and not set bounds on the data in the input
domain.
