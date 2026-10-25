.. _expression-index:

Expressions
===========

Polars uses expressions to describe arbitrary computations that emit a single column.
OpenDP parses these expressions into stable transformations.

In the following pages we describe the subset of Polars expressions supported by OpenDP.
(For more information on Polars expressions outside OpenDP see
the `Polars User Guide <https://docs.pola.rs/user-guide/expressions/>`_
and `API Reference <https://docs.pola.rs/api/python/stable/reference/expressions/index.html>`_.)

OpenDP also adds extensions for differential privacy under the :code:`.dp` namespace;
these are documented under :py:class:`opendp.extras.polars.DPExpr`.

.. toctree::
  :maxdepth: 1
  
  aggregation
  boolean
  columns
  manipulation
  operators
  string
  temporal
