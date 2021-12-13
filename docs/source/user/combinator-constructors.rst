.. testsetup::

    from opendp.mod import enable_features
    enable_features('contrib', 'floating-point')

.. _combinator-constructors:

Combinator Constructors
=======================

Combinator constructors use transformations or measurements to produce new transformations or measurements.
Combinators are an area of OpenDP that are still heavily in development,
but the chainers in particular are foundational and well-tested.

.. _chaining:

Chaining
--------

Two of the most essential constructors are the "chainers" that chain transformations with transformations, and transformations with measurements.
Chainers are used to incrementally piece Transformations or Measurements together that represent longer computational pipelines.

The :py:func:`opendp.comb.make_chain_tt` constructor creates a new Transformation by combining an inner and an outer Transformation.
The resulting Transformation contains a function that sequentially executes the function of the constituent Transformations.
It also contains a privacy relation that relates an input distance bound on the inner Transformation with an output distance bound on the outer transformation.

The :py:func:`opendp.comb.make_chain_mt` constructor similarly creates a new Measurement by combining an inner Transformation with an outer Measurement.
Notice that `there is no` ``make_chain_mm`` for chaining measurements together!
Any computation beyond a measurement is postprocessing and need not be governed by relations.

In the following example we chain :py:func:`opendp.meas.make_base_discrete_laplace` with :py:func:`opendp.trans.make_bounded_sum`.

.. doctest::

    >>> from opendp.trans import make_bounded_sum
    >>> from opendp.meas import make_base_discrete_laplace
    >>> from opendp.comb import make_chain_mt
    ...
    >>> # call a constructor to produce a transformation
    >>> bounded_sum = make_bounded_sum(bounds=(0, 1))
    >>> # call a constructor to produce a measurement
    >>> base_discrete_laplace = make_base_discrete_laplace(scale=1.0)
    >>> noisy_sum = make_chain_mt(base_discrete_laplace, bounded_sum)
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
The syntax automatically chooses between :func:`make_chain_mt <opendp.mod.make_chain_mt>` and :func:`make_chain_tt <opendp.mod.make_chain_tt>`.

.. doctest::

    >>> noisy_sum = bounded_sum >> base_discrete_laplace

.. _chaining-mismatch:

In this example the chaining was successful because:

* bounded_sum's output domain is equivalent to base_discrete_laplace's input domain
* bounded_sum's output metric is equivalent to base_discrete_laplace's input metric

Chaining fails if we were to adjust the domains such that they won't match.
In the below example, the adjustment is subtle, but the bounds were adjusted to floats.
``make_bounded_sum`` is equally capable of summing floats,
but the chaining fails because the sum emits floats and the discrete_laplace mechanism expects integers.

.. doctest::

    >>> from opendp.mod import OpenDPException
    >>> try:
    ...     make_bounded_sum(bounds=(0., 1.)) >> base_discrete_laplace
    ... except OpenDPException as err:
    ...     print(err.message[:-1])
    Intermediate domains don't match. See https://github.com/opendp/opendp/discussions/297
        The structure of the intermediate domains are the same, but the types or parameters differ.
        shared_domain: AllDomain()

Note that ``noisy_sum``'s input domain and input metric come from ``bounded_sum``'s input domain and input metric.
This is intended to enable further chaining with preprocessors like :py:func:`make_cast <opendp.trans.make_cast>`, :py:func:`make_impute_constant <opendp.trans.make_impute_constant>`, :py:func:`make_clamp <opendp.trans.make_clamp>` and :py:func:`make_bounded_resize <opendp.trans.make_bounded_resize>`.
See the section on :ref:`transformation-constructors` for more information on how to preprocess data in OpenDP.

Composition
-----------

OpenDP has a simple composition combinator for building a tuple of measurements out of two measurements.
You can nest this composition combinator arbitrarily.

.. doctest::

    >>> from opendp.comb import make_basic_composition
    >>> noisy_sum_pair = make_basic_composition(noisy_sum, noisy_sum)


Progress on more general composition constructors can be found in the following PR's:

:#242: `sequential compositor w/ static distances <https://github.com/opendp/opendp/pull/242>`_
:#264: `f-DP <https://github.com/opendp/opendp/pull/264>`_

Amplification
-------------

If your dataset is a simple sample from a larger population,
you can make the privacy relation more permissive by wrapping your measurement with a privacy amplification combinator.

In order to demonstrate this API, we'll first create a measurement with a sized input domain.
The resulting measurement expects the size of the input dataset to be 10.

.. doctest::

    >>> from opendp.trans import make_sized_bounded_mean
    >>> from opendp.meas import make_base_laplace
    >>> meas = make_sized_bounded_mean(size=10, bounds=(0., 10.)) >> make_base_laplace(scale=0.5)
    >>> print("standard mean:", amplified([1.] * 10)) # -> 1.03 # doctest: +SKIP

We can now use the amplification combinator to construct an amplified measurement.
The function on the amplified measurement is identical to the standard measurement.

.. doctest::

    >>> from opendp.comb import make_population_amplification
    >>> amplified = make_population_amplification(meas, population_size=100)
    >>> print("amplified mean:", amplified([1.] * 10)) # -> .97 # doctest: +SKIP

The privacy relation on the amplified measurement takes into account that the input dataset of size 10
is a simple sample of individuals from a theoretical larger dataset that captures the entire population, with 100 rows.

.. doctest::

    >>> # Where we once had a privacy utilization of 2 epsilon...
    >>> assert meas.check(2, 2.)
    ...
    >>> # ...we now have a privacy utilization of ~.4941 epsilon.
    >>> assert amplified.check(2, .4941)

The efficacy of this combinator improves as n gets larger.
