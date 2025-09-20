.. _contingency-tables:

Contingency Tables
==================

A *contingency table* counts the number of records in each group, when grouping by the columns in a dataset.
Contingency tables capture the relationships between columns in your data.

As the number of columns in the contingency table grows, 
the number of possible combinations of keys (AKA groups) increases exponentially quickly.
This results in the contingency table becoming too large to hold in memory or measure with any utility.
For this reason, it is recommended to instead estimate specific relationships between smaller sets of two or maybe three columns
by releasing *marginals* (counts when data is grouped by some subset of the columns).

OpenDP uses `Private-PGM <https://private-pgm.readthedocs.io/en/latest/introduction.html>`_ 
to build a contingency table that is consistent with a set of marginal queries via a process called model-based inference (``mbi``).
This functionality is not enabled by default.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        Add ``[mbi]`` to the package installation to enable model-based-inference functionality.

        .. prompt:: bash

            pip install 'opendp[mbi]'

        This installs the specific version of the Private-PGM package that is compatible with the OpenDP Library.

    .. tab-item:: R
        :sync: r

        ``mbi`` (Private-PGM) is only available in Python. 

    .. tab-item:: Rust
        :sync: rust

        ``mbi`` (Private-PGM) is only available in Python. 

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
    ...     privacy_loss=dp.loss_of(rho=0.2, delta=1e-7),
    ... )

``.contingency_table`` releases an approximation to a contingency table over all columns
by estimating lower order marginals.
To improve the utility of the mechanism,
specify the key-sets for categorical data, and cut-sets for numerical data.
The mechanism will release stable keys for any columns that don't have key- or cut-sets.


.. code:: pycon

    >>> query = (
    ...     context.query(rho=0.1, delta=1e-7)
    ...     # transformations/truncation may be applied here
    ...     .select(
    ...         "SEX", "AGE", "HWUSUAL", "ILOSTAT"
    ...     ).contingency_table(
    ...         keys={"SEX": [1, 2]},
    ...         cuts={"AGE": [20, 40, 60], "HWUSUAL": [1, 20, 40]},
    ...         algorithm=dp.mbi.Fixed(
    ...             queries=[
    ...                 dp.mbi.Count(("AGE", "HWUSUAL")),
    ...                 dp.mbi.Count(("HWUSUAL", "ILOSTAT")),
    ...             ],
    ...             # proportion of budget to spend estimating one-way marginal key-sets
    ...             #   (defaults to num_unknown / num_columns / 2)
    ...             oneway_split=1 / 8,
    ...         ),
    ...     )
    ... )

By default, up to half of the privacy budget is used to estimate one-way marginals and their keys.
The privacy budget used depends on the proportion of columns with unknown keys or cuts.
For example, in the prior query, keys are specified for all columns except ``ILOSTAT``,
so an eighth of the budget is used to estimate a one-way marginal on ``ILOSTAT``.
You can configure the privacy budget split for one-way marginals and key release
via the ``oneway_split`` attribute on the algorithm.

Before releasing, you can view the noise scale and threshold to be used when estimating one-way key-sets.

.. code:: pycon

    >>> query.oneway_scale
    227.68399153212334

    >>> query.oneway_threshold
    1364

In this setting, the scale and threshold are reasonably small.
To make the threshold smaller, consider adding more key-sets 
or bounding the number of groups a user may contribute.

When you release, all marginals are estimated and stored inside a :py:class:`opendp.extras.mbi.ContingencyTable`.
The contingency table supports counting queries over arbitrary sets of grouping columns.

.. code:: pycon

    >>> table: dp.mbi.ContingencyTable = query.release()

Any keys in the data that are not present in the key-set,
either because they are missing from the key-set (like a third category for ``SEX``)
or because they had frequencies too low to meet the threshold (like the uncommon ``ILOSTAT`` category ``5``),
are replaced with ``null`` when estimating higher-order marginals.

.. code:: pycon

    >>> table.keys["ILOSTAT"]  # doctest: +NORMALIZE_WHITESPACE
    shape: (5,)
    Series: 'ILOSTAT' [i64]
    [
        1
        2
        3
        9
        null
    ]

In this example, infrequent ``ILOSTAT`` categories of ``4`` and ``5`` were absorbed into the ``null`` key.
Preserve true nulls in your data by preprocessing them into another key via ``.select`` or ``.with_columns``.

To see counts of the number of records corresponding to each key,
project the contingency table down to just ``ILOSTAT``.

.. code:: pycon

    >>> table.project(("ILOSTAT",)).astype(int)  # doctest: +SKIP
    Array([1515729,  155912, 1450748,  689452,     109], dtype=int64)

The same projection can be viewed in a melted dataframe form:

.. code:: pycon

    >>> table.project_melted(("ILOSTAT",))  # doctest: +SKIP
    shape: (5, 2)
    ┌─────────┬──────────────┐
    │ ILOSTAT ┆ len          │
    │ ---     ┆ ---          │
    │ i64     ┆ f64          │
    ╞═════════╪══════════════╡
    │ 1       ┆ 1.5157e6     │
    │ 2       ┆ 156011.15206 │
    │ 3       ┆ 1.4509e6     │
    │ 9       ┆ 689551.39267 │
    │ null    ┆ 119.022842   │
    └─────────┴──────────────┘

Since ``("ILOSTAT",)`` is covered by the query workload,
an estimate of the standard deviation can be derived.
Since all noise added is gaussian-distributed, 
the resulting noise distribution remains approximately gaussian-distributed,
so it is possible to construct a confidence interval for each scalar in the projection.

.. code:: pycon

    >>> scale = table.std(("ILOSTAT",))
    >>> dp.gaussian_scale_to_accuracy(scale, alpha=0.05)
    446.2524232592843

That is, the true count differs from the estimate by no more than the accuracy estimate,
with ``(1 - alpha)100%`` confidence.

Adaptive Estimation
-------------------

Now consider the ``SEX`` column, which contains three keys: 
the two specified in ``keys``, as well as ``null`` for any records not in the key set.

.. code:: pycon

    >>> table.keys["SEX"]  # doctest: +NORMALIZE_WHITESPACE
    shape: (3,)
    Series: 'SEX' [i64]
    [
        1
        2
        null
    ]

``SEX`` is not well-supported by the contingency table,
because it is not included in any of the released marginals.
As a result, the best estimate for the total number of records is distributed uniformly over the key set,
and the standard deviation is unknown.

.. code:: pycon

    >>> table.project(("SEX",)).astype(int)  # doctest: +SKIP
    Array([1270561, 1270561, 1270561], dtype=int64)

    >>> table.std(("SEX",))  # doctest: +IGNORE_EXCEPTION_DETAIL
    Traceback (most recent call last):
    ...
    ValueError: attrs (('SEX',)) are not covered by the query set

To improve the utility of this projection, you could release an updated contingency table with more marginals.

.. code:: pycon

    >>> table2 = (
    ...     context.query(rho=0.05, delta=0.0)
    ...     .select("SEX")
    ...     .contingency_table(
    ...         table=table,
    ...         algorithm=dp.mbi.Fixed(
    ...             queries=[dp.mbi.Count(("SEX",))]
    ...         ),
    ...     )
    ...     .release()
    ... )


The updated table now much more accurately reflects the distribution of counts over ``SEX``.

.. code:: pycon

    >>> table2.project(("SEX",)).astype(int)  # doctest: +SKIP
    Array([1828251, 1982295,    1228], dtype=int64)

    >>> table2.std(("SEX",))
    113.84199576606166

The contingency table mechanism can satisfy either pure-DP (epsilon) or zCDP (rho). 
If approximate-DP (delta) is not enabled, then the key-set and cut-set must be exhaustive.
Since microdata is never materialized outside of Polars, 
datasets can be as large in size as can be handled by the Polars library.

The sensitivity of marginals released by the mechanism are derived in the same fashion as any other OpenDP Polars marginal query,
meaning they benefit from bounds on user contribution defined in the privacy unit or acquired through identifier truncation.
