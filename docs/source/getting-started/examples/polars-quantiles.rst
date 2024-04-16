Polars Quantiles
================

The Polars API for OpenDP is under development and subject to change.
We are currently focused on supporting a particular use case,
but we plan to expand the API until it is the preferred interface
for both Python and R.

The OpenDP Polars API leverages `Polars <https://docs.pola.rs/>`_:
You'll use Polars methods like ``group_by``, ``agg``, and ``col`` to construct
most of your query, and only use ``dp`` extensions when needed.

This example demonstrates how to construct DP aggregate statistics on quantiles.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: init
            :end-before: /init

The columns of Polars dataframes have data types analogous to the types used by OpenDP domains.
While there is a function to extract an OpenDP domain from a Polars dataframe,
we discourage its use: We should be very careful about indvertantly leaking information,
so it is safer to give the public characteristics of a domain without referencing the private data.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: init-domain
            :end-before: /init-domain

Grouping on a column necessarily reveals characteristics of that column,
so we need to modify the original domain to make clear that these characteristics are public.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/polars-quantiles.rst
            :language: python
            :start-after: margin-domain
            :end-before: /margin-domain