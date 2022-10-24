.. testsetup::

    from opendp.mod import enable_features
    enable_features('contrib', 'floating-point', 'honest-but-curious')

.. _combinator-constructors:

Combinators
===========

Combinator constructors use Transformations or Measurements to produce a new Transformation or Measurement.

.. _chaining:

Chaining
--------

Two of the most essential constructors are the "chainers" that chain transformations with transformations, and transformations with measurements.
Chainers are used to incrementally piece Transformations or Measurements together that represent longer computational pipelines.

The :py:func:`opendp.combinators.make_chain_tt` constructor creates a new Transformation by combining an inner and an outer Transformation.
The resulting Transformation contains a function that sequentially executes the function of the constituent Transformations.
It also contains a privacy map that takes an input distance bound on the inner Transformation and emits an output distance bound on the outer transformation.

The :py:func:`opendp.combinators.make_chain_mt` constructor similarly creates a new Measurement by combining an inner Transformation with an outer Measurement.
Notice that `there is no` ``make_chain_mm`` for chaining measurements together!
Any computation beyond a measurement is postprocessing and need not be governed by relations.

Postprocessing functionality is provided by the :py:func:`opendp.combinators.make_chain_tm` constructor that allows transformations to be chained onto a Measurement.
Since the outer Transformation is postprocessing, the metrics and stability map of the outer Transformation are ignored.
In this case, it is only necessary for the domains to conform.

In the following example we chain :py:func:`opendp.measurements.make_base_discrete_laplace` with :py:func:`opendp.transformations.make_bounded_sum`.

.. doctest::

    >>> from opendp.transformations import make_bounded_sum
    >>> from opendp.measurements import make_base_discrete_laplace
    >>> from opendp.combinators import make_chain_mt
    ...
    >>> # call a constructor to produce a transformation
    >>> bounded_sum = make_bounded_sum(bounds=(0, 1))
    >>> # call a constructor to produce a measurement
    >>> base_dl = make_base_discrete_laplace(scale=1.0)
    >>> noisy_sum = make_chain_mt(base_dl, bounded_sum)
    ...
    >>> # investigate the privacy relation
    >>> symmetric_distance = 1
    >>> epsilon = 1.0
    >>> assert noisy_sum.check(d_in=symmetric_distance, d_out=epsilon)
    ...
    >>> # invoke the chained measurement's function
    >>> dataset = [0, 0, 1, 1, 0, 1, 1, 1]
    >>> release = noisy_sum(dataset)

In practice, these chainers are used so frequently that we've written a shorthand (``>>``).
The syntax automatically chooses between :func:`make_chain_mt <opendp.combinators.make_chain_mt>`, 
:func:`make_chain_tt <opendp.combinators.make_chain_tt>`, 
and :func:`make_chain_tm <opendp.combinators.make_chain_tm>`.

.. doctest::

    >>> noisy_sum = bounded_sum >> base_dl

.. _chaining-mismatch:

In this example the chaining was successful because:

* bounded_sum's output domain is equivalent to base_dl's input domain
* bounded_sum's output metric is equivalent to base_dl's input metric

Chaining fails if we were to adjust the domains such that they won't match.
In the below example, the adjustment is subtle, but the bounds were adjusted to floats.
``make_bounded_sum`` is equally capable of summing floats,
but the chaining fails because the sum emits floats and the discrete laplace mechanism expects integers.

.. doctest::

    >>> from opendp.mod import OpenDPException
    >>> try:
    ...     make_bounded_sum(bounds=(0., 1.)) >> base_dl
    ... except OpenDPException as err:
    ...     print(err.message[:-1])
    Intermediate domains don't match. See https://github.com/opendp/opendp/discussions/297
        output_domain: AllDomain(f64)
        input_domain:  AllDomain(i32)

Note that ``noisy_sum``'s input domain and input metric come from ``bounded_sum``'s input domain and input metric.
This is intended to enable further chaining with preprocessors like :py:func:`make_cast <opendp.transformations.make_cast>`, :py:func:`make_impute_constant <opendp.transformations.make_impute_constant>`, :py:func:`make_clamp <opendp.transformations.make_clamp>` and :py:func:`make_bounded_resize <opendp.transformations.make_bounded_resize>`.
See the section on :ref:`transformation-constructors` for more information on how to preprocess data in OpenDP.

Composition
-----------

OpenDP has a basic composition combinator for composing a list of measurements into a new measurement:
:func:`opendp.combinators.make_basic_composition`.

.. doctest::

    >>> from opendp.combinators import make_basic_composition
    >>> noisy_sum_pair = make_basic_composition([noisy_sum, noisy_sum])
    >>> release_1, release_2 = noisy_sum_pair(dataset)

This kind of composition primitive gives a structural guarantee that all statistics are computed together in a batch.
Thus the privacy map simply sums the constituent output distances.

.. doctest::

    >>> noisy_sum_pair.map(1)
    2.0

This combinator can compose Measurements with ``ZeroConcentratedDivergence``, ``MaxDivergence`` and ``FixedSmoothedMaxDivergence`` output measures.

.. _measure-casting:

Measure Casting
---------------
These combinators are used to cast the output measure of a Measurement.

.. list-table::
   :header-rows: 1

   * - Input Measure
     - Output Measure
     - Constructor
   * - ``MaxDivergence<Q>``
     - ``FixedSmoothedMaxDivergence<Q>``
     - :func:`opendp.combinators.make_pureDP_to_fixed_approxDP`
   * - ``MaxDivergence<Q>``
     - ``ZeroConcentratedDivergence<Q>``
     - :func:`opendp.combinators.make_pureDP_to_zCDP`
   * - ``ZeroConcentratedDivergence<Q>``
     - ``SmoothedMaxDivergence<Q>``
     - :func:`opendp.combinators.make_zCDP_to_approxDP`
   * - ``SmoothedMaxDivergence<Q>``
     - ``FixedSmoothedMaxDivergence<Q>``
     - :func:`opendp.combinators.make_fix_delta`

:func:`opendp.combinators.make_pureDP_to_fixed_approxDP` is used for casting an output measure from ``MaxDivergence`` to ``FixedSmoothedMaxDivergence``.
This is useful if you want to compose pure-DP measurements with approximate-DP measurements.

.. doctest::

    >>> from opendp.measurements import make_base_laplace
    >>> from opendp.combinators import make_pureDP_to_fixed_approxDP
    >>> meas_pureDP = make_base_laplace(scale=10.)
    >>> # convert the output measure to `FixedSmoothedMaxDivergence`
    >>> meas_fixed_approxDP = make_pureDP_to_fixed_approxDP(meas_pureDP)
    ...
    >>> # FixedSmoothedMaxDivergence distances are (ε, δ) tuples
    >>> meas_fixed_approxDP.map(d_in=1.)
    (0.1, 0.0)

Similarly, :func:`opendp.combinators.make_pureDP_to_zCDP` is used for casting an output measure from ``MaxDivergence`` to ``ZeroConcentratedDivergence``.


:func:`opendp.combinators.make_zCDP_to_approxDP` is used for casting an output measure from ``ZeroConcentratedDivergence`` to ``SmoothedMaxDivergence``.

.. doctest::

    >>> from opendp.measurements import make_base_gaussian
    >>> from opendp.combinators import make_zCDP_to_approxDP
    >>> meas_zCDP = make_base_gaussian(scale=0.5)
    >>> # convert the output measure to `SmoothedMaxDivergence`
    >>> meas_approxDP = make_zCDP_to_approxDP(meas_zCDP)
    ...
    >>> # SmoothedMaxDivergence distances are ε(δ) curves
    >>> curve = meas_approxDP.map(d_in=1.)
    >>> curve.epsilon(delta=1e-6)
    11.688596249354896

:func:`opendp.combinators.make_fix_delta` changes the output measure from ``SmoothedMaxDivergence`` to ``FixedSmoothedMaxDivergence``.
It fixes the delta parameter in the curve, so that the resulting measurement can be composed with other ``FixedSmoothedMaxDivergence`` measurements.

.. doctest::

    >>> from opendp.combinators import make_fix_delta
    >>> # convert the output measure to `FixedSmoothedMaxDivergence`
    >>> meas_fixed_approxDP = make_fix_delta(meas_approxDP, delta=1e-8)
    ...
    >>> # FixedSmoothedMaxDivergence distances are (ε, δ) tuples
    >>> meas_fixed_approxDP.map(d_in=1.)
    (13.3861046488579, 1e-08)

These last two combinators allow you to convert output distances in terms of ρ-zCDP to ε(δ)-approxDP, and then to (ε, δ)-approxDP.


Amplification
-------------

If your dataset is a simple sample from a larger population,
you can make the privacy relation more permissive by wrapping your measurement with a privacy amplification combinator:
:func:`opendp.combinators.make_population_amplification`.

The amplifier requires a looser trust model, as the population size can be set arbitrarily.

.. doctest::

    enable_features("honest-but-curious")


In order to demonstrate this API, we'll first create a measurement with a sized input domain.
The resulting measurement expects the size of the input dataset to be 10.

.. doctest::

    >>> from opendp.transformations import make_sized_bounded_mean
    >>> from opendp.measurements import make_base_laplace
    >>> meas = make_sized_bounded_mean(size=10, bounds=(0., 10.)) >> make_base_laplace(scale=0.5)
    >>> print("standard mean:", amplified([1.] * 10)) # -> 1.03 # doctest: +SKIP

We can now use the amplification combinator to construct an amplified measurement.
The function on the amplified measurement is identical to the standard measurement.

.. doctest::

    >>> from opendp.combinators import make_population_amplification
    >>> amplified = make_population_amplification(meas, population_size=100)
    >>> print("amplified mean:", amplified([1.] * 10)) # -> .97 # doctest: +SKIP

The privacy relation on the amplified measurement takes into account that the input dataset of size 10
is a simple sample of individuals from a theoretical larger dataset that captures the entire population, with 100 rows.

.. doctest::

    >>> # Where we once had a privacy utilization of ~2 epsilon...
    >>> assert meas.check(2, 2. + 1e-6)
    ...
    >>> # ...we now have a privacy utilization of ~.4941 epsilon.
    >>> assert amplified.check(2, .4941)

The efficacy of this combinator improves as n gets larger.
