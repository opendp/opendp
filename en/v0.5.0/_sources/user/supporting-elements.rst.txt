Supporting Elements
===================

This section builds on the :ref:`core-structures` documentation to expand on the constituent pieces of Measurements and Transformations.


.. _functions:

Function
--------
As one would expect, all data processing is handled via a function.
The function member stored in a Transformation or Measurement struct is straightforward representation of an idealized mathematical function.
A mathematical function is a binary relation between two sets
that associates each value in the input set with a value in the output set.
In OpenDP, we capture these sets with domains...

.. _domains:

Domains
-------
A domain describes the set of all possible input values of a function, or all possible output values of a function.
Two domains (``input_domain`` and ``output_domain``) are bundled within each Transformation or Measurement to describe all possible inputs and outputs of the function.

Some common domains are:

:AllDomain<T>: | The set of all non-null values in the type T.
  | For example, ``AllDomain<u8>`` describes the set of all possible unsigned 8-bit integers:
  | ``{0, 1, 2, 3, ..., 127}``.
:BoundedDomain<T>: | The set of all non-null values in the type T, bounded between some L and U.
  | For example, ``BoundedDomain<i32>`` between -2 and 2:
  | ``{-2, -1, 0, 1, 2}``.
:VectorDomain<D>: | The set of all vectors, where each element of the vector is a member of domain D.
  | For example, ``VectorDomain<AllDomain<bool>>`` describes the set of all boolean vectors:
  | ``{[True], [False], [True, True], [True, False], ...}``.
:SizedDomain<D>: | The set of all values in the domain D, that have a specific size.
  | For example, ``SizedDomain<VectorDomain<AllDomain<bool>>>`` of size 2 describe the set of boolean vectors:
  | ``{[True, True], [True, False], [False, True], [False, False]}``.

In many cases, you provide some qualities about the underlying domain and the rest is automatically chosen by the constructor.

Let's look at the Transformation returned from :py:func:`make_bounded_sum(bounds=(0, 1)) <opendp.trans.make_bounded_sum>`.
The input domain has type ``VectorDomain<BoundedDomain<i32>>``,
read as "the set of all vectors of 32-bit signed integers bounded between 0 and 1."
The bounds argument to the constructor provides L and U, and since TIA (atomic input type) is not passed,
TIA is inferred from the type of the public bounds.
The output domain is simply ``AllDomain<i32>``, or "the set of all 32-bit signed integers."

These domains serve two purposes:

#. The relation depends on the input and output domain in its proof to restrict the set of neighboring datasets or distributions.
   An example is the relation for :py:func:`opendp.trans.make_sized_bounded_sum`,
   which makes use of a ``SizedDomain`` domain descriptor to more tightly bound the sensitivity.
#. Combinators also use domains to ensure the output is well-defined.
   For instance, chainer constructors check that intermediate domains are equivalent
   to guarantee that the output of the interior function is always a valid input to the exterior function.


.. _metrics:

Metrics
-------
A metric is a function that computes the distance between two elements of a set.

.. _symmetric-distance:

A concrete example of a metric in opendp is ``SymmetricDistance``, or "the symmetric distance metric ``|A △ B| = |(A\B) ∪ (B\A)|``."
This is used to count the fewest number of additions or removals to convert one dataset ``A`` into another dataset ``B``.

.. _absolute-distance:

Each metric is bundled together with a domain, and ``A`` and ``B`` are members of that domain.
Since the symmetric distance metric is often paired with a ``VectorDomain<D>``, ``A`` and ``B`` are often vectors.
In practice, if we had a dataset where each user can influence at most k records, we would say that the symmetric distance is bounded by `k`, so ``d_in=k``.

Another example metric is ``AbsoluteDistance<f64>``.
This can be read as "the absolute distance metric ``|A - B|``, where distances are expressed in 64-bit floats."
This metric is used to represent global sensitivities
(an upper bound on how much an aggregated value can change if you were to perturb an individual in the original dataset).
In practice, most users will not have a need to provide global sensitivities to privacy relations,
because they are a midway distance bound encountered while relating dataset distances and privacy distances.
However, there are situations where constructors accept a metric for specifying the metric for sensitivities.

.. _measures:

Measures
--------
In OpenDP, a measure is a function for measuring the distance between probability distributions.

.. _max-divergence:

A concrete example is ``MaxDivergence<f64>``,
read as "the max divergence metric where numbers are expressed in terms of 64-bit floats."
The max divergence measure has distances that correspond to ``epsilon`` in the pure definition of differential privacy.


.. _smoothed-max-divergence:

Another example is ``SmoothedMaxDivergence<f64>``.
The smoothed max divergence measure corresponds to approximate differential privacy,
where distances are ``(epsilon, delta)`` tuples.

Every Measurement (:ref:`see listing <measurement-constructors>`) contains an output_measure, and compositors are always typed by a Measure.

.. _relations:

Relations
---------
We assert the privacy properties of a Transformation or Measurement's function via a relation.
Relations accept a ``d_in`` and a ``d_out`` and return a boolean.
There are a couple equivalent interpretations for when a relation returns True:

* All potential input perturbations do not significantly influence the output.
* The transformation or measurement is (``d_in``, ``d_out``)-close.

What does (``d_in``, ``d_out``)-close mean?
If a measurement is (``d_in``, ``d_out``)-close,
then the output is ``d_out``-DP when the input is changed by at most ``d_in``.
If a transformation is (``d_in``, ``d_out``)-close,
then the output can change by at most ``d_out`` when the input is changed by at most ``d_in``.

.. The relation tells you if the function is (``d_in``, ``d_out``)-close for any choice of ``d_in`` and ``d_out``.

What are ``d_in`` and ``d_out``?
``d_in`` and ``d_out`` are distances in terms of the input and output metric or measure.
Refer to :ref:`distances` below for more details.

This should be enough rope to work with, but let's still touch quickly on the mathematical side.
Refer to the programming framework paper itself if you want a deeper understanding.
Consider ``d_X`` the input metric, ``d_Y`` the output metric or measure,
and ``f`` the function in the Transformation or Measurement.

A slightly more mathematical way to express this is:
If the relation passes, then it tells you that, for all ``x``, ``x'`` in the input domain:

* if ``d_X(x, x') <= d_in`` (if neighboring datasets are at most ``d_in``-close)
* then ``d_Y(f(x), f(x')) <= d_out`` (then the distance between function outputs is no greater than ``d_out``)

Notice that if the relation passes at ``d_out``, it will pass for any value greater than ``d_out``.
This is an incredibly useful observation, as we will see in the :ref:`parameter-search` section.

Putting this to practice, the following example checks the stability relation on a clamp transformation.

.. testsetup::

    from opendp.mod import enable_features
    enable_features('contrib')

.. doctest::

    >>> from opendp.trans import make_clamp
    >>> clamp = make_clamp(bounds=(1, 10))
    ...
    >>> # The maximum number of records that any one individual may influence in your dataset
    >>> in_symmetric_distance = 3
    >>> # clamp is a 1-stable transformation, so this should pass for any symmetric_distance >= 3
    >>> assert clamp.check(d_in=in_symmetric_distance, d_out=4)

.. _maps:

Maps
----
A map is a function that takes some ``d_in`` and returns the smallest ``d_out`` that is (``d_in``, ``d_out``)-close.

Maps are a useful shorthand to find privacy properties directly:

.. doctest::

    >>> # reusing the prior clamp transformation
    >>> clamp.map(d_in=3)
    3

The relation check predicate function simply compares the output of the map with ``d_out`` as follows: ``d_out >= map(d_in)``.
For a more thorough understanding of maps, please read the :ref:`relations <relations>` section.

.. _distances:

Distances
---------

You can determine what units ``d_in`` and ``d_out`` are expressed in based on the ``input_metric``, and ``output_metric`` or ``output_measure``.
Follow the links into the example metrics and measures to get more detail on what the distances mean for that kind of metric or measure.

On Transformations, the ``input_metric`` will be a dataset metric like :ref:`SymmetricDistance <symmetric-distance>`.
The ``output_metric`` will either be some dataset metric (on dataset transformations)
or some kind of global sensitivity metric like :ref:`AbsoluteDistance <absolute-distance>` (on aggregations).

The ``input_metric`` of Measurements is initially only some kind of global sensitivity metric.
However, once you chain the Measurement with a Transformation, the resulting Measurement will have whatever ``input_metric`` was on the Transformation.
The ``output_measure`` of Measurements is some kind of privacy measure like :ref:`MaxDivergence <max-divergence>` or :ref:`SmoothedMaxDivergence <smoothed-max-divergence>`.

It is critical that you choose the correct ``d_in`` for the relation,
whereas you can use :ref:`binary search utilities <parameter-search>` to find the tightest ``d_out``.
Practically speaking, the smaller the ``d_out``, the tighter your analysis will be.

You might find it surprising that metrics and measures are never actually evaluated!
The framework does not evaluate these because it only needs to relate a user-provided input distance to another user-provided output distance.
Even the user should not directly compute input and output distances:
they are :ref:`solved-for <determining-accuracy>`, :ref:`bisected <parameter-search>`, or even :ref:`contextual <putting-together>`.

Be careful: even a dataset query to determine the greatest number of contributions made by any one individual can itself be private information.


.. _RuntimeTypeDescriptor:

Type Arguments
--------------

Many of the API docs reference RuntimeTypeDescriptor on type arguments like `TIA` or `D`.
When you want to describe the type of a domain, metric, measure, or other elements, you can do so via a type descriptor.
The canonical form is a literal string, like `"i32"` to denote that the type should be a 32-bit integer,
or `"SymmetricDistance"` to denote the type of a metric.
In practice, people like to denote types in ways they are already familiar, 
like the python type `int`, which is treated like `"i32"`, so we've made this as flexible as possible.
Whenever you see an argument of type `RuntimeTypeDescriptor`, you may use any of the following, depending on the context:

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

* `TA` for the atomic type. `float` could be the TA for a float :py:func:`clamp transformation <opendp.trans.make_clamp>`.
* `TIA` for Atomic Input Type. `str` could be the TIA for a :py:func:`count transformation <opendp.trans.make_count>`.
* `MO` for Output Metric. `AbsoluteDistance[int]` could be the MO for a :py:func:`histogram transformation <opendp.trans.make_count_by_categories>`.
* `QO` for Output distance. `float` could be the QO for a :py:func:`discrete laplace measurement <opendp.meas.make_base_discrete_laplace>`.

The API docs should also include explanations in most contexts.
