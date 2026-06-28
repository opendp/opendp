Boolean
=======

[`Polars
Documentation <https://docs.pola.rs/api/python/stable/reference/expressions/boolean.html>`__]

OpenDP supports a subset of the Polars boolean functions.

Is Property
-----------

Since these expressions are row-by-row, they may be used in row-by-row
or aggregation contexts.

- ``is_null``
- ``is_not_null``
- ``is_finite``
- ``is_not_finite``
- ``is_nan``
- ``is_not_nan``

The latter four only apply to float types that can be NaN or infinite.

The output domain of ``is_null`` and ``is_not_null`` does not include
null values, even if the input does. These expressions can be useful as
part of a predicate function for filtering, or as part of a group-by.

Not
---

Commonly used for negating predicates.

Less commonly, the input data to ``.not_`` may be non-boolean, resulting
in a bitwise negation. In this setting, all domain descriptors about
elements in the column are dropped.
