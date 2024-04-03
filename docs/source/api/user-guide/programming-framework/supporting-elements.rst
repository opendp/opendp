Supporting Elements
===================

This section builds on the :ref:`core-user-guide` documentation to expand on the constituent pieces of Measurements and Transformations.


.. _functions-user-guide:

Function
--------

As one would expect, all data processing is handled via a function.
The function member stored in a Transformation or Measurement struct is a straightforward representation of an idealized mathematical function.

To use the function, the Transformation or Measurement can be called directly:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> import opendp.prelude as dp
      >>> dp.enable_features("contrib")
      >>> input_domain = dp.vector_domain(dp.atom_domain(T=float))
      >>> input_metric = dp.symmetric_distance()
      >>> clamp = dp.t.make_clamp(input_domain, input_metric, bounds=(0., 5.))
      >>> type(clamp)
      <class 'opendp.mod.Transformation'>
      >>> clamp([10.0])
      [5.0]

Or ``invoke`` can be used equivalently:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> type(clamp.invoke)
      <class 'method'>
      >>> clamp.invoke([10.0])
      [5.0]

A mathematical function associates each value in some input set with some value in the output set (or a distribution over such values, in the case of a randomized function).
In OpenDP, as discussed in the next section, we capture these sets with domains.

.. _domains-user-guide:

Domain
------

(See also :py:mod:`opendp.domains` in the API reference.)

A domain describes the set of all possible input values of a function, or all possible output values of a function.
Transformations have both an ``input_domain`` and ``output_domain``, while measurements only have an ``input_domain``.

A commonly-used domain is ``atom_domain(T)``, which describes the set of all possible non-null values of type ``T``.
The following example creates a domain consisting of all possible non-null 64-bit floats, 
and checks that 1.0 is a member of the domain, but NaN is not.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> f64_atom_domain = dp.atom_domain(T=float)  # float defaults to f64, a double-precision 64-bit float
      >>> assert f64_atom_domain.member(1.0)
      >>> assert not f64_atom_domain.member(float('nan'))

Other domains may be described in a similar way. For example:

* ``atom_domain(T=u8)`` consists of all possible non-null unsigned 8-bit integers: ``{0, 1, 2, 3, ..., 127}``
* ``atom_domain(bounds=(-2, 2))`` describes all possible 32-bit signed integers bounded between -2 and 2: ``{-2, -1, 0, 1, 2}``.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> i32_bounded_domain = dp.atom_domain(bounds=(-2, 2))  # int defaults to i32, a 32-bit signed integer
      >>> assert i32_bounded_domain.member(-2)
      >>> assert not i32_bounded_domain.member(3)

In addition, domains may also be used to construct higher-level domains. For instance:

* ``vector_domain(atom_domain(T=bool))`` describes the set of all boolean vectors: ``{[], [True], [False], [True, True], [True, False], ...}``.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> bool_vector_domain = dp.vector_domain(dp.atom_domain(T=bool))
      >>> assert bool_vector_domain.member([])
      >>> assert bool_vector_domain.member([True, False])

In addition, a ``size`` parameter may be used. For example:

* ``vector_domain(atom_domain(T=bool), size=2)`` describes the set of boolean vectors of size 2: ``{[True, True], [True, False], [False, True], [False, False]}``.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> bool_vector_2_domain = dp.vector_domain(dp.atom_domain(T=bool), size=2)
      >>> assert bool_vector_2_domain.member([True, True])
      >>> assert not bool_vector_2_domain.member([True, True, True])

Let's look at the Transformation returned from :py:func:`make_sum() <opendp.transformations.make_sum>`.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> dp.enable_features('contrib')
      >>> bounded_sum = dp.t.make_sum(
      ...     input_domain=dp.vector_domain(dp.atom_domain(bounds=(0, 1))), 
      ...     input_metric=dp.symmetric_distance(),
      ... )
      >>> bounded_sum.input_domain
      VectorDomain(AtomDomain(bounds=[0, 1], T=i32))

We see that the input domain is the same as we passed in: 
"the set of all vectors of 32-bit signed integers bounded between 0 and 1."

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> bounded_sum.output_domain
      AtomDomain(T=i32)

The output domain is "the set of all 32-bit signed integers."

These domains serve two purposes:

#. The stability map or privacy map depends on the input domain in its proof to restrict the set of neighboring datasets or distributions.
   An example is the relation for :py:func:`opendp.transformations.make_sum`,
   which may make use of a size descriptor in the vector domain to more tightly bound the sensitivity.
#. Combinators also use domains to ensure that the output is well-defined.
   For instance, chainer constructors check that intermediate domains are equivalent
   to guarantee that the output of the first function is always a valid input to the second function.


.. _metrics-user-guide:

Metric
------

(See also :py:mod:`opendp.metrics` in the API reference.)

A metric is a function that computes the distance between two elements of a domain.
Transformations have both an ``input_metric`` and ``output_metric``, while measurements only have an ``input_metric``.

.. _symmetric-distance:

A concrete example of a metric in opendp is ``SymmetricDistance``, or "the symmetric distance metric ``|A △ B| = |(A\B) ∪ (B\A)|``."
This is used to count the fewest number of additions or removals to convert one dataset ``A`` into another dataset ``B``.

.. _absolute-distance:

Each metric is bundled together with a domain, and ``A`` and ``B`` are members of that domain.
Since the symmetric distance metric is often paired with a ``VectorDomain<D>``, ``A`` and ``B`` are often vectors.
If we had a dataset where each user can influence at most k records, we would say that the symmetric distance is bounded by `k`, so ``d_in=k`` 
(where ``d_in`` denotes an upper bound on the distance between adjacent inputs).

Another example metric is ``AbsoluteDistance<f64>``.
This can be read as "the absolute distance metric ``|A - B|``, where distances are expressed in 64-bit floats."
This metric is used to represent global sensitivities
(an upper bound on how much an aggregated value can change if you were to perturb an individual in the original dataset).
In practice, you may not have a need to provide global sensitivities to stability/privacy maps,
because they are a midway distance bound encountered while relating dataset distances and privacy distances.
However, there are situations where constructors accept a metric for specifying the metric for sensitivities.

.. _measures-user-guide:

Measure
-------

(See also :py:mod:`opendp.measures` in the API reference.)

In OpenDP, a measure is a function for measuring the distance between probability distributions.
Transformations don't make use of a measure, but measurements do have an ``output_measure``.

.. _max-divergence:

A concrete example is ``MaxDivergence<f64>``,
read as "the max divergence metric where numbers are expressed in terms of 64-bit floats."
The max divergence measure has distances that correspond to ``epsilon`` in the definition of pure differential privacy.


.. _smoothed-max-divergence:

Another example is ``SmoothedMaxDivergence<f64>``.
The smoothed max divergence measure corresponds to approximate differential privacy,
where distances are ``(epsilon, delta)`` tuples.

Every Measurement (:ref:`see listing <measurement-constructors>`) contains an output_measure, and compositors are always typed by a Measure.


.. _maps:

Stability/Privacy Map
---------------------
A map is a function that takes some ``d_in`` and returns a ``d_out`` that is (``d_in``, ``d_out``)-close.

``d_in`` is a distance in terms of the input metric, and ``d_out`` is a distance in terms of the output metric or measure.
Refer to :ref:`distances` below for more details on what ``d_in`` and ``d_out`` are.

If a measurement is (``d_in``, ``d_out``)-close,
then the output is ``d_out``-DP when the input may change by at most ``d_in``.
If a transformation is (``d_in``, ``d_out``)-close,
then the output can change by at most ``d_out`` when the input may change by at most ``d_in``.

The ``d_out`` returned is not necessarily the smallest value that is still "close",
but every effort is made to make it as small as provably possible.

Maps are a useful tool to find stability or privacy properties directly.

Putting this to practice, the following example invokes the stability map on a clamp transformation.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> clamper = dp.t.make_clamp(dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance(), bounds=(1, 10))
        ...
        >>> # The maximum number of records that any one individual may influence in your dataset
        >>> in_symmetric_distance = 3
        >>> # clamp is a 1-stable transformation, so this should pass for any symmetric_distance >= 3
        >>> clamper.map(d_in=in_symmetric_distance)
        3

There is also a relation check predicate function that simply compares the output of the map with ``d_out`` as follows: ``d_out >= map(d_in)``.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> # reusing the prior clamp transformation
        >>> assert clamper.check(d_in=3, d_out=3)

This should be sufficient to make use of the library, but a more mathematical treatment may help give a more thorough understanding.
Consider ``d_X`` the input metric, ``d_Y`` the output metric or measure,
and ``f`` the function in the Transformation or Measurement.

If the relation check passes, then it tells you that, for all ``x``, ``x'`` in the input domain:

* if ``d_X(x, x') <= d_in`` (if neighboring datasets are at most ``d_in``-close)
* then ``d_Y(f(x), f(x')) <= d_out`` (then the distance between function outputs is no greater than ``d_out``)

Notice that if the relation passes at ``d_out``, it will pass for any value greater than ``d_out`` 
(so long as the relation doesn't throw an error due to numerical overflow).
The usefulness of this property is shown in the :ref:`parameter-search` section.


.. _distances:

Distance
--------

You can determine what units ``d_in`` and ``d_out`` are expressed in based on the ``input_metric``, and ``output_metric`` or ``output_measure``.
Follow the links into the example metrics and measures to get more detail on what the distances mean for that kind of metric or measure.

On Transformations, the ``input_metric`` will typically be a dataset metric like :ref:`SymmetricDistance <symmetric-distance>`.
The ``output_metric`` will typically be either some dataset metric (on dataset transformations)
or some kind of global sensitivity metric like :ref:`AbsoluteDistance <absolute-distance>` (on aggregations).

The ``input_metric`` of Measurements is initially only some kind of global sensitivity metric.
However, once you chain the Measurement with a Transformation, the resulting Measurement will have whatever ``input_metric`` was on the Transformation.
The ``output_measure`` of Measurements is some kind of privacy measure like :ref:`MaxDivergence <max-divergence>` or :ref:`SmoothedMaxDivergence <smoothed-max-divergence>`.

In some cases, distances may not form a total order. 
For example, in :math:`(\epsilon, \delta)`-DP, :math:`(\epsilon_1, \delta_1) = (1.5, 1e-6)` is incomparable to :math:`(\epsilon_2, \delta_2) = (1.0, 1e-7)`, 
so neither :math:`(\epsilon_1, \delta_1) \ge (\epsilon_2, \delta_2)` nor :math:`(\epsilon_2, \delta_2) \ge (\epsilon_1, \delta_1)` holds.
However, :math:`(1.5, 1e-6) \ge (1.0, 1e-6)` would still hold, as both elements compare greater than or equal.

It is critical that you choose the correct ``d_in`` for the relation,
whereas you can use :ref:`binary search utilities <parameter-search>` to find the tightest ``d_out``.
Practically speaking, the smaller the ``d_out``, the tighter your analysis will be.

You might find it surprising that metrics and measures are never actually evaluated!
The framework does not evaluate these because it only needs to relate a user-provided input distance to another user-provided output distance.
Even the user should not directly compute input and output distances:
they are :ref:`solved-for <accuracy-user-guide>`, :ref:`bisected <parameter-search>`, or provided by the :ref:`Context API <context-user-guide>`.

Be careful: even a dataset query to determine the greatest number of contributions made by any one individual can itself be private information.
