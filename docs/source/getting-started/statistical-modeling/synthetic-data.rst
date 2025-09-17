.. _synthetic-data:

Synthetic Data
==============

Synthetic data mechanisms in OpenDP attempt to model the relationships between columns in your data via a contingency table,
and then generate synthetic data with the same relationships.

Read the :ref:`contingency table <contingency-tables>` documentation first for install instructions,
how to preprocess data, and how to specify and/or release keys.
Synthetic data works in much the same way, but the query workload is no longer fixed (using the :py:class:`opendp.extras.mbi.Fixed` algorithm).

All algorithms for differentially private contingency table estimation inherit from :py:class:`opendp.extras.mbi.Algorithm`:

* :py:class:`opendp.extras.mbi.Fixed` (static, pre-defined workload) 
* :py:class:`opendp.extras.mbi.AIM` (synthetic data: adaptive and iterative mechanism) 
* :py:class:`opendp.extras.mbi.MST` (synthetic data: minimum spanning tree) 
* :py:class:`opendp.extras.mbi.Sequential` (run a sequence of algorithms)

Let's get started by setting up the context for the Labor Force dataset.

.. code:: pycon

    >>> import opendp.prelude as dp
    >>> import polars as pl

    >>> dp.enable_features("contrib")

    >>> context = dp.Context.compositor(
    ...     data=pl.scan_csv(
    ...         dp.examples.get_france_lfs_path(),
    ...         ignore_errors=True,
    ...     ),
    ...     privacy_unit=dp.unit_of(contributions=36),
    ...     privacy_loss=dp.loss_of(rho=0.2, delta=2e-7),
    ... )

We now release a contingency table via the :py:class:`opendp.extras.mbi.AIM` algorithm.

.. code:: pycon

    >>> table_aim = (
    ...     context.query(rho=0.1, delta=1e-7)
    ...     # transformations/truncation may be applied here
    ...     .select("SEX", "AGE", "HWUSUAL", "ILOSTAT")
    ...     .contingency_table(
    ...         keys={"SEX": [1, 2]},
    ...         cuts={"AGE": [20, 40, 60], "HWUSUAL": [1, 20, 40]},
    ...         algorithm=dp.mbi.AIM(),
    ...     )
    ...     .release()
    ... )

Generation of synthetic data from a DP contingency table is considered postprocessing, 
and thus does not affect the privacy budget.

.. code:: pycon

    >>> table_aim.synthesize()  # doctest: +SKIP
    shape: (3_807_732, 4)
    ┌─────┬───────────┬───────────┬─────────┐
    │ SEX ┆ AGE       ┆ HWUSUAL   ┆ ILOSTAT │
    │ --- ┆ ---       ┆ ---       ┆ ---     │
    │ i64 ┆ f64       ┆ f64       ┆ i64     │
    ╞═════╪═══════════╪═══════════╪═════════╡
    │ 1   ┆ 55.446336 ┆ 20.776579 ┆ 1       │
    │ 1   ┆ 28.21838  ┆ 40.53348  ┆ 1       │
    │ 2   ┆ 43.291215 ┆ 34.406155 ┆ 1       │
    │ 1   ┆ 55.106615 ┆ 22.413161 ┆ 1       │
    │ 2   ┆ 42.585227 ┆ 40.11279  ┆ 3       │
    │ …   ┆ …         ┆ …         ┆ …       │
    │ 1   ┆ 58.197292 ┆ 40.139579 ┆ 1       │
    │ 1   ┆ 59.371221 ┆ 19.671153 ┆ 1       │
    │ 2   ┆ 19.862917 ┆ 40.339046 ┆ 9       │
    │ 1   ┆ 19.492355 ┆ 32.233661 ┆ 1       │
    │ 2   ┆ 60.863244 ┆ 40.737908 ┆ 3       │
    └─────┴───────────┴───────────┴─────────┘

Numerical columns with cuts are sampled uniformly from between the bin edges,
in a manner consistent with the input data types.

Handling Null Values
--------------------

The underlying marginal-based inference algorithm 
requires that every column in the data has a statically defined key-set.
The key-set may come from two sources:

* Explicit Keys: passed by the user
* Stable Keys: estimated with a portion of the privacy loss budget

In both cases, the set of keys is not necessarily exhaustive:
explicit keys (defined by the user) may not span all keys in the data, 
and stable keys omits keys with low counts.
OpenDP replaces all keys that are not present in the key-set with null.

