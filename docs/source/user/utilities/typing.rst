Typing
======




.. _RuntimeTypeDescriptor:

Type Argument
-------------

Many of the API docs indicate that parameters like `TIA` or `D` are type arguments.
When you want to describe the type of a domain, metric, measure, or other elements, you can do so via a type descriptor.
The canonical form is a literal string, like `"i32"` to denote that the type should be a 32-bit integer,
or `"SymmetricDistance"` to denote the type of a metric.
In practice, people like to denote types in ways they are already familiar, 
like the python type `int`, which is treated like `"i32"`, so we've made this as flexible as possible.
These arguments accept any of the following, depending on the context:

* string
* python type (like `float` or `bool`)
* python typing module annotation (like `List[str]`)
* :py:mod:`opendp.typing` (mimics python type annotations for OpenDP types)

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
