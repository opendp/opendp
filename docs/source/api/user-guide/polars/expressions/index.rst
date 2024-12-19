.. _Expression Index:

Expression Index
================

Polars uses expressions to describe arbitrary computations that emit a single column.
OpenDP parses these expressions into stable transformations.

OpenDP supports many, but not all, Polars expressions, 
and adds extensions under the :code:`.dp` namespace, which returns a :py:class:`opendp.extras.polars.DPExpr`.

The following sections align one-to-one with the Polars API documentation structure,
but contain additional information specific to their usage in OpenDP for differential privacy.

.. toctree::

  aggregation
  boolean
  columns
  manipulation
  operators
  string
  temporal
