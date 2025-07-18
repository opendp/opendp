.. _synthetic-data:

Synthetic Data
==============

Synthetic data mechanisms in OpenDP attempt to model the relationships between columns in your data via a contingency table,
and then generate synthetic data with the same relationships.

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

.. code:: python

    >>> import opendp.prelude as dp
    >>> import polars as pl

    >>> dp.enable_features("contrib")

    >>> context = dp.Context.compositor(
    ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
    ...     privacy_unit=dp.unit_of(contributions=36),
    ...     privacy_loss=dp.loss_of(rho=0.2, delta=2e-7),
    ... )


The :ref:`contingency table <contingency-tables>` documentation is recommended reading 
and gives an introduction to the problem,
but differs in that the query workload is fixed (using the :py:class:`opendp.extras.mbi.Fixed` algorithm).
All algorithms for differentially private contingency table estimation inherit from :py:class:`opendp.extras.mbi.Algorithm`:
these being :py:class:`opendp.extras.mbi.Fixed`, :py:class:`opendp.extras.mbi.AIM`, :py:class:`opendp.extras.mbi.MST` and :py:class:`opendp.extras.mbi.Sequential`.

The algorithms for synthetic data differ from the fixed algorithm
in that they first estimate all unknown first-order marginals, not just the marginals with missing keys.
This behavior can be controlled by switching between ``.oneway="all"`` and ``.oneway="unkeyed"``
in the algorithm of choice.

Algorithms for synthetic data have the convenience of automatically choosing which marginals to release,
but this comes with the tradeoff of spending a portion of the overall privacy budget on marginal selection.
Since OpenDP supports adaptively estimating the contingency table,
you might consider first specifying a fixed workload for columns of interest via the :py:class:`opendp.extras.mbi.Fixed` algorithm (`tutorial <contingency-tables>`_),
and then spending the rest of your privacy budget fine-tuning with a synthetic data algorithm.

Minimum Spanning Tree (MST)
---------------------------
`MST <https://arxiv.org/abs/2108.04978>`_ greedily chooses 
pairs of columns that are most poorly represented by the DP contingency table
in a way that guarantees all columns become connected by a minimum spanning tree.
MST then releases all of the selected marginals.

.. code:: python

    >>> table_mst = (
    ...     context.query(rho=0.1, delta=1e-7)
    ...     # transformations/truncation may be applied here
    ...     .select("SEX", "AGE", "HWUSUAL", "ILOSTAT")
    ...     .contingency_table(
    ...         keys={"SEX": [1, 2]}, 
    ...         cuts={"AGE": [20, 40, 60], "HWUSUAL": [1, 20, 40]},
    ...         algorithm=dp.mbi.MST()
    ...     )
    ...     .release()
    ... )

See the documentation of :py:class:`opendp.extras.mbi.MST` for more information on how to customize the settings of the algorithm.

Adaptive Iterative Mechanism (AIM)
----------------------------------
`AIM <https://arxiv.org/abs/2201.12677>`_ iteratively chooses 
and releases a marginal over a clique (set of columns) that is most poorly represented by the DP contingency table.
The stronger the correlation amongst a clique of columns, 
the more likely AIM is to select the clique.
The algorithm starts with a small per-step privacy budget, 
and in each step increases the budget if the last measured marginal doesn't sufficiently improve the model.

.. code:: python

    >>> table_aim = (
    ...     context.query(rho=0.1, delta=1e-7)
    ...     # transformations/truncation may be applied here
    ...     .select("SEX", "AGE", "HWUSUAL", "ILOSTAT")
    ...     .contingency_table(
    ...         keys={"SEX": [1, 2]}, 
    ...         cuts={"AGE": [20, 40, 60], "HWUSUAL": [1, 20, 40]},
    ...         algorithm=dp.mbi.AIM()
    ...     )
    ...     .release()
    ... )

In the interest of making these examples stand-alone, ``table_aim`` from the prior section is not re-used. 
However, building on the table from a prior release is valid and is the generally preferable approach.

See the documentation of :py:class:`opendp.extras.mbi.AIM` for more information on how to customize the settings of the algorithm.

Generating Synthetic Data
-------------------------
Generation of synthetic data from a DP contingency table is considered postprocessing, 
and thus does not affect the privacy budget.

.. code:: python

    >>> table_aim.synthesize() # doctest: +SKIP
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