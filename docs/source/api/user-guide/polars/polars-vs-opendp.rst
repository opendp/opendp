
Polars vs. OpenDP
=================

(For the examples which follow create a :py:class:`opendp.context.Context` named :code:`context`.)

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> import polars as pl
        >>> df = pl.LazyFrame({"age": [1, 2, 3]})

        >>> context = dp.Context.compositor(
        ...     data=df,
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=10,
        ... )

OpenDP Polars differs from typical Polars in these ways:

1. **How you specify the data.**
   Instead of directly manipulating the data (a ``LazyFrame``),
   you now manipulate an :py:class:`opendp.extras.polars.LazyFrameQuery`
   returned by :code:`context.query()`
   You can think of :code:`context.query()` as a mock for the real data
   (although in reality, a ``LazyFrameQuery`` is an empty ``LazyFrame`` with some extra methods).

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> #                                 /‾‾‾‾‾‾‾‾‾‾‾‾‾\
        >>> query: dp.polars.LazyFrameQuery = context.query().select(
        ...     dp.len()
        ... )

2. **How you construct the query.**
   OpenDP extends the Polars API to include differentially private methods and statistics.
   ``LazyFrame`` (now ``LazyFrameQuery``) has additional methods, like :code:`.summarize` and :code:`.release`.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> #    /‾‾‾‾‾‾‾‾‾‾\
        >>> query.summarize()
        shape: (1, 4)
        ┌────────┬──────────────┬─────────────────┬───────┐
        │ column ┆ aggregate    ┆ distribution    ┆ scale │
        │ ---    ┆ ---          ┆ ---             ┆ ---   │
        │ str    ┆ str          ┆ str             ┆ f64   │
        ╞════════╪══════════════╪═════════════════╪═══════╡
        │ len    ┆ Frame Length ┆ Integer Laplace ┆ 10.0  │
        └────────┴──────────────┴─────────────────┴───────┘

Expressions also have an additional namespace :code:`.dp` with methods from :py:class:`opendp.extras.polars.DPExpr`.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> candidates = list(range(0, 100, 10))
        >>>
        >>> _ = context.query().select(
        ...     pl.col("income")
        ...     .fill_null(0)
        ...     .dp.median(candidates)  # "dp" namespace
        ... )

3. **How you run the query.**
   When used from OpenDP, you must first call :code:`.release()` before executing the computation with :code:`.collect()`.
   :code:`.release()` accounts for the privacy loss of releasing the query, and updates the privacy budget.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> #    /‾‾‾‾‾‾‾‾\
        >>> query.release().collect()
        shape: (1, 1)
        ┌─────┐
        │ len │
        │ --- │
        │ u32 │
        ╞═════╡
        │ ... │
        └─────┘

4. **What queries are allowed.**
   OpenDP only makes guarantees about query plans and expressions it knows about.
   Therefore OpenDP is somewhat like an allow-list on valid query plans.

   To satisfy differential privacy, there are also cases where OpenDP must change the arguments to a Polars expression.
   Most commonly this is to ensure that failures don't raise data-dependent errors.
   OpenDP may also make arguments mandatory (for example, `format strings in temporal parsing <expressions/string.html#Strptime,-To-Date,-To-Datetime,-To-Time>`_),
   or disallow the use of expressions on certain data types (for example, `imputation on categorical data <data-types.html#Categorical>`_).

   These changes in behavior, and the reasoning behind them, are discussed in :ref:`expression-index`.