String
======

[`Polars
Documentation <https://docs.pola.rs/api/python/stable/reference/expressions/string.html>`__]

In the string module, OpenDP currently only supports parsing to temporal
data types.

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
            ...     split_evenly_over=2,
            ... )
            

Strptime, To Date, To Datetime, To Time
---------------------------------------

Dates can be parsed from strings via ``.str.strptime``, and its variants
``.str.to_date``, ``.str.to_datetime``, and ``.str.to_time``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query = (
            ...     context.query()
            ...     .with_columns(pl.col.YEAR.cast(str).str.to_date(format=r"%Y"))
            ...     .group_by("YEAR")
            ...     .agg(dp.len())
            ... )
            >>> query.release().collect().sort("YEAR")
            shape: (10, 2)
            ┌────────────┬───────┐
            │ YEAR       ┆ len   │
            │ ---        ┆ ---   │
            │ date       ┆ u32   │
            ╞════════════╪═══════╡
            │ 2004-01-01 ┆ ...   │
            │ 2005-01-01 ┆ ...   │
            │ 2006-01-01 ┆ ...   │
            │ 2007-01-01 ┆ ...   │
            │ 2008-01-01 ┆ ...   │
            │ 2009-01-01 ┆ ...   │
            │ 2010-01-01 ┆ ...   │
            │ 2011-01-01 ┆ ...   │
            │ 2012-01-01 ┆ ...   │
            │ 2013-01-01 ┆ ...   │
            └────────────┴───────┘

While Polars supports automatic inference of the datetime format from
reading the data, doing so can lead to situations where the
data-dependent inferred format changes or cannot be inferred upon the
addition or removal of a single individual, resulting in an unstable
computation. For this reason, the ``format`` argument is required.

OpenDP also does not allow parsing strings into nanosecond datetimes,
because the underlying implementation throws data-dependent errors (not
private) `for certain
inputs <https://github.com/pola-rs/polars/issues/19928>`__.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query = (
            ...     context.query()
            ...     .with_columns(pl.col.YEAR.cast(str).str.to_datetime(format=r"%Y", time_unit="ns"))
            ...     .group_by("YEAR")
            ...     .agg(dp.len())
            ... )
            >>> query.release()
            Traceback (most recent call last):
            ...
            opendp.mod.OpenDPException:
              MakeMeasurement("Nanoseconds are not currently supported due to potential panics when parsing inputs. Please open an issue on the OpenDP repository if you would find this functionality useful. Otherwise, consider parsing into micro- or millisecond datetimes instead.")
            Predicate in binary search always raises an exception. This exception is raised when the predicate is evaluated at 0.0.


Parsed data can then be manipulated with `temporal
expressions <temporal.ipynb>`__.
