Polars API
==========

The Polars API for OpenDP is under development and subject to change.
We are currently focused on supporting a particular use case,
but we plan to expand the API until it is the preferred interface
for both Python and R.

The OpenDP Polars API leverages `Polars <https://docs.pola.rs/>`_:
You'll use Polars methods like ``group_by``, ``agg``, ``alias``, and ``col`` to construct
most of your query, and only use ``dp`` extensions when needed.

This example demonstrates how to construct DP aggregate statistics, including quantiles.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: init
            :end-before: /init

We need to understand the structure of our private data before we can apply differential privacy,
but we usually shouldn't or can't look at the private data as we prepare our analysis.
For this example, we'll keep the data simple:

* ``grouping-key``: the grouping key, an integer between 1 and 5
* ``noisy-key``: most of the time, equal to ``grouping-key``, but some rows are set to 1, and some are set to 5.  
* ``ones``: a constant value, the float 1.0

Our first step is to represent this as a domain:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: init-domain
            :end-before: /init-domain

Grouping on a column necessarily reveals characteristics of that column,
so we need to modify the original domain to make this margin explicitly public.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: margin-domain
            :end-before: /margin-domain

Now we'll use this same information, but instead of preparing an OpenDP domain,
we'll create an empty Polars `LazyFrame <https://docs.pola.rs/py-polars/html/reference/lazyframe/index.html>`_.
We won't be storing data in this LazyFrame:
Instead we'll use it to keep track of the steps of our analysis,
and then pass it back to OpenDP for evaluation.

Sums and means
--------------

Let's first look at the calculation of differentially private sums and means.
Note the use of ``dp`` to access differentially private extensions to Polars.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: means-plan
            :end-before: /means-plan

We can pass this ``means_plan`` to :py:func:`make_private_lazyframe <opendp.measurements.make_private_lazyframe>` to get a measurement function:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: means-measurement
            :end-before: /means-measurement

Finally, the ``means_measurement`` function is applied to the private data to create a DP release:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: means-release
            :end-before: /means-release

Note that after the ``collect`` you have a normal Polars DataFrame,
so you can use the Polars methods for post-processing.

In this case you should have a DataFrame with 5 rows, corresponding to the key values.
The first column, ``grouping-key``, will be the values 1 through 5.
After that, the values for ``sum of ones`` will be centered on 10, while ``mean of ones`` will center on 1.0.
Calculating the mean requires that ``public_info="lengths"`` be enabled in ``with_margin``;
If only sums are required, then ``public_info="keys"`` would suffice.

Medians and quantiles
---------------------

Let's now consider the calculation of medians and quantiles.
These work a little differently, because instead of supplying bounds, we provide candidate values.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: quantiles-plan-measurement-release
            :end-before: /quantiles-plan-measurement-release