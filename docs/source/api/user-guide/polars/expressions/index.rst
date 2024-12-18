Expression Index
=========================

Polars uses expressions to describe arbitrary computations that emit a single column.
OpenDP parses these expressions into stable transformations.

The OpenDP Library operates based on an allowlist of recognized expressions, 
and custom expressions for differential privacy made accessible under :py:class`DPExpr`.

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
