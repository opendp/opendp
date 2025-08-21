Identifier Truncation
=====================

OpenDP can be used to query datasets where each individual may
contribute an unbounded number of records, but where all records
contributed by an individual share the same identifier.

The user identifier is a part of the privacy unit.
(Remember that the privacy unit quantifies the influence an individual may have on the data.)

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import opendp.prelude as dp
            >>> import polars as pl
            
            >>> # The PIDENT column contains individual identifiers.
            >>> # An individual may contribute data under at most 1 PIDENT identifier.
            >>> privacy_unit = dp.unit_of(contributions=1, identifier=pl.col("PIDENT"))
            

This ``privacy_unit`` consists of all records associated with any one
unique identifier in the ``PIDENT`` column. OpenDP allows identifiers to
be arbitrary Polars expressions. The identifier expression must be
row-by-row to be well-defined.

We'll use this new ``privacy_unit`` to create a context as we have previously:

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

If an identifier has been used in creating a context,
an additional identifier truncation step is necessary in the query,
where only a limited number of records corresponding to each identifier are kept.

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

The ``.truncate_per_group(10)`` is equivalent to ``.filter(pl.int_range(pl.len()).over("PIDENT") < 10)``
and returns the same scale parameters,
but `truncate_per_group` is easier to read and write.

Previous examples with this dataset assumed the worst-case of 36
contributed records per individual (one contribution per quarter for
nine years) which resulted in a 36-fold increase in the amount of noise.
By truncating to at most ten records, there is only a 10-fold increase
in the amount of noise. This statistical estimator is introducing some
bias by dropping records from individuals who contributed more than ten
records, but on the other hand there is much lower variance.


Truncating Contributed Groups
-----------------------------

To release queries that involve identifier columns and grouping, it is also necessary to bound
the number of groups an individual may contribute to, and not just the
number of contributions per-group.

The following query demonstrates a second truncation that also limits the
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


OpenDP allows queries to contain multiple truncations, so long as they
are together in the data pipeline. OpenDP does, however, enforce that
group-by truncations are the last truncations in the data pipeline.

See :ref:`Bounds <bounds-user-guide>` in the API user guide, and
:py:func:`truncate_per_group <opendp.extras.polars.LazyFrameQuery.truncate_per_group>`
and :py:func:`truncate_num_groups <opendp.extras.polars.LazyFrameQuery.truncate_num_groups>`
in the API documentation for more on configuring truncation.