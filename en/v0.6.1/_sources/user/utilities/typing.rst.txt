Typing
======

OpenDP computations are always strict about the types being used. 
Integers and floats are never treated interchangeably and there is never implicit casting between types.
In fact, OpenDP is particular about the bit-depth of data types, as it can impact the choice of constants in the privacy analysis.

.. _RuntimeTypeDescriptor:

Type Argument
-------------

You can explicitly set the types that a transformation or measurement work with via type arguments in the constructor.
Each constructor has its own set of permissible types, based on the type of computation it is performing.
For instance, the clamp constructor accepts any numerical type for its type argument `TA`:

.. testsetup::

    from opendp.mod import enable_features
    enable_features('contrib')

.. doctest::

    >>> from opendp.transformations import make_clamp
    >>> from opendp.typing import *
    ...
    >>> clamper = make_clamp((0, 1), TA=i32)

Many of the API docs indicate that parameters like `TIA` or `D` are type arguments.
When you want to describe the type of a domain, metric, measure, or other elements, you can do so via a type descriptor.
The canonical form is a literal string, like ``"i32"`` to denote that the type should be a 32-bit integer,
or ``"SymmetricDistance"`` to denote the type of a metric.
These arguments accept any of the following, depending on the context:

* string
* Python type (like `float` or `bool`)
* Python typing module annotation (like `List[str]`)
* :py:mod:`opendp.typing` (mimics Python type annotations for OpenDP types)

In addition, there is a common pattern to the naming of type arguments.

* `D` for Domain
* `M` for Metric or Measure
* `T` for the type of members of a domain
* `Q` for the type of distances in a metric or measure

There are additional modifiers:

* `I` for Input
* `O` for Output
* `A` for Atomic, the smallest type. `i32` is the atomic type of `Vec<i32>`

Some examples being:

* `TA` for the atomic type. `float` could be the TA for a float :py:func:`clamp transformation <opendp.transformations.make_clamp>`.
* `TIA` for Atomic Input Type. `str` could be the TIA for a :py:func:`count transformation <opendp.transformations.make_count>`.
* `MO` for Output Metric. `AbsoluteDistance[int]` could be the MO for a :py:func:`histogram transformation <opendp.transformations.make_count_by_categories>`.
* `QO` for Output distance. `float` could be the QO for a :py:func:`discrete laplace measurement <opendp.measurements.make_base_discrete_laplace>`.

The API docs should also include explanations in most contexts.

Supported Types
---------------

OpenDP supports the following atomic data types:

* i8 (8-bit signed integer)
* i16 (16-bit signed integer)
* i32 
* i64
* i128
* u8 (8-bit unsigned integer)
* u16 (16-bit unsigned integer)
* u32 
* u64
* u128
* f32 (32-bit single-precision float)
* f64 (64-bit double-precision float)
* String
* bool

Some types are parameterized by another type, like:

* AllDomain
* VectorDomain
* AbsoluteDistance
* L1Distance
* L2Distance

Examples of these types include:

* AllDomain[bool]
* VectorDomain[AllDomain[f32]]
* L1Distance[u128]

The docstrings on the constructor APIs should typically guide you as to what types are permissible.
If you aren't familiar with these concepts, it may help to review :ref:`domains` and :ref:`metrics`.


Type Aliases
------------

It can be more convenient to denote types in terms of Python types, so we've added some aliases for Python types.


.. list-table::
   :header-rows: 1

   * - Python Type Alias
     - Default Rust Type
   * - ``float``
     - ``f64``
   * - ``int``
     - ``i32``
   * - ``str``
     - ``String``
   * - ``bool``
     - ``bool``

You can change the default type for floats and ints via :py:func:`opendp.typing.set_default_float_type` and :py:func:`opendp.typing.set_default_int_type`, respectively.
These functions make it easy to set the default bit depth throughout your code, all at once.

This can be particularly useful when working with NumPy arrays which default to `i64`, or when working with deep learning libraries that default to single-precision floats. 
