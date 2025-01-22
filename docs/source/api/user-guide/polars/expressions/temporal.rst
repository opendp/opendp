.. _temporal:

Temporal
========

[`Polars
Documentation <https://docs.pola.rs/api/python/stable/reference/expressions/temporal.html>`__]

OpenDP supports some manipulation of dates and times, which can be
useful in predicates and grouping functions.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import polars as pl
            
            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")
            
            >>> lf_dates = (
            ...     pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True)
            ...     # prepare the data with some expressions that are not yet supported in OpenDP
            ...     .select(DATE=pl.concat_str("REFYEAR", pl.col.QUARTER * 3, pl.lit("01"), separator="-"))
            ... )
            
            >>> context = dp.Context.compositor(
            ...     data=lf_dates,
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
            ...     split_evenly_over=1,
            ... )
            

Date/Time Components
--------------------

-  Date expressions (can be applied to ``pl.Date`` and ``pl.Datetime``
   dtypes)

   -  ``.dt.year``
   -  ``.dt.iso_year``
   -  ``.dt.quarter``
   -  ``.dt.month``
   -  ``.dt.week``
   -  ``.dt.weekday``
   -  ``.dt.day``
   -  ``.dt.ordinal_day``

-  Time expressions (can be applied to ``pl.Time`` and ``pl.Datetime``
   dtypes)

   -  ``.dt.hour``
   -  ``.dt.minute``
   -  ``.dt.second``
   -  ``.dt.millisecond``
   -  ``.dt.microsecond``
   -  ``.dt.nanosecond``

An example of their use can be seen below, where a string column is
parsed into dates, and then year and month components are retrieved from
the dates.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query = (
            ...     context.query()
            ...     .with_columns(pl.col.DATE.str.to_date(format=r"%Y-%m-%d"))
            ...     .with_columns(YEAR=pl.col.DATE.dt.year(), MONTH=pl.col.DATE.dt.month())
            ...     .group_by("YEAR", "MONTH")
            ...     .agg(dp.len())
            ... )
            >>> query.release().collect().sort("YEAR", "MONTH")
            shape: (40, 3)
            ┌──────┬───────┬──────┐
            │ YEAR ┆ MONTH ┆ len  │
            │ ---  ┆ ---   ┆ ---  │
            │ i32  ┆ i8    ┆ u32  │
            ╞══════╪═══════╪══════╡
            │ 2004 ┆ 3     ┆ ...  │
            │ 2004 ┆ 6     ┆ ...  │
            │ 2004 ┆ 9     ┆ ...  │
            │ 2004 ┆ 12    ┆ ...  │
            │ 2005 ┆ 3     ┆ ...  │
            │ ...                 │
            └──────┴───────┴──────┘