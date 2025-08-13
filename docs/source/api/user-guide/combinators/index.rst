.. _combinators-user-guide:

Combinators
===========

(See also :py:mod:`opendp.combinators` in the API reference.)

Combinator constructors use Transformations or Measurements to produce a new Transformation or Measurement.

.. _chaining:

Chaining
--------

Two of the most essential constructors are the "chainers" that chain transformations with transformations, and transformations with measurements.
Chainers are used to incrementally piece Transformations or Measurements together that represent longer computational pipelines.

The :py:func:`opendp.combinators.make_chain_tt` constructor creates a new Transformation by stitching together two Transformations sequentially.
The resulting Transformation contains a function that sequentially executes the function of the constituent Transformations.
It also contains a privacy map that takes an input distance bound on the inner Transformation and emits an output distance bound on the outer transformation.

The :py:func:`opendp.combinators.make_chain_mt` constructor similarly creates a new Measurement by combining an inner Transformation with an outer Measurement.
Notice that `there is no` ``make_chain_mm`` for chaining measurements together!
Any computation beyond a measurement is postprocessing and need not be governed by relations.

Postprocessing functionality is provided by the :py:func:`opendp.combinators.make_chain_pm` constructor that allows transformations to be chained onto a Measurement.
Since the outer Transformation is postprocessing, the metrics and stability map of the outer Transformation are ignored.
In this case, it is only necessary for the types to conform.

In the following example we chain :py:func:`opendp.measurements.make_laplace` with :py:func:`opendp.transformations.make_sum`.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> import opendp.prelude as dp
        >>> dp.enable_features("contrib", "floating-point")
        >>> # call a constructor to produce a transformation
        >>> sum_trans = dp.t.make_sum(
        ...     dp.vector_domain(dp.atom_domain(bounds=(0, 1))),
        ...     dp.symmetric_distance(),
        ... )
        >>> # call a constructor to produce a measurement
        >>> lap_meas = dp.m.make_laplace(
        ...     sum_trans.output_domain,
        ...     sum_trans.output_metric,
        ...     scale=1.0,
        ... )
        >>> noisy_sum = dp.c.make_chain_mt(lap_meas, sum_trans)
        >>> # investigate the privacy relation
        >>> symmetric_distance = 1
        >>> epsilon = 1.0
        >>> assert noisy_sum.check(
        ...     d_in=symmetric_distance, d_out=epsilon
        ... )
        >>> # invoke the chained measurement's function
        >>> dataset = [0, 0, 1, 1, 0, 1, 1, 1]
        >>> release = noisy_sum(dataset)

In practice, these chainers are used so frequently that we've written a shorthand (``>>``).
The syntax automatically chooses between :func:`make_chain_mt <opendp.combinators.make_chain_mt>`, 
:func:`make_chain_tt <opendp.combinators.make_chain_tt>`, 
and :func:`make_chain_pm <opendp.combinators.make_chain_pm>`.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> noisy_sum_meas = sum_trans >> lap_meas

.. _chaining-mismatch:

In this example the chaining was successful because:

* bounded_sum's output domain is equivalent to base_dl's input domain
* bounded_sum's output metric is equivalent to base_dl's input metric

Chaining fails if we adjust the domains such that they don't match.
In the below example, the adjustment is subtle, but the bounds were adjusted to floats.
``make_sum`` is equally capable of summing floats, but the chaining fails because the sum emits floats and the discrete Laplace mechanism expects integers.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> # call a constructor to produce a transformation, but this time with float bounds
        >>> sum_trans = dp.t.make_sum(
        ...     dp.vector_domain(dp.atom_domain(bounds=(0.0, 1.0))),
        ...     dp.symmetric_distance(),
        ... )
        >>> sum_trans >> lap_meas
        Traceback (most recent call last):
        ...
        opendp.mod.OpenDPException: 
          DomainMismatch("Intermediate domains don't match. See https://github.com/opendp/opendp/discussions/297
            output_domain: AtomDomain(T=f64)
            input_domain:  AtomDomain(T=i32)
        ")

Note that ``noisy_sum_trans``'s input domain and input metric come from ``sum_trans``'s input domain and input metric.
This is intended to enable further chaining with preprocessors such as:
* :py:func:`make_cast <opendp.transformations.make_cast>`
* :py:func:`make_impute_constant <opendp.transformations.make_impute_constant>`
* :py:func:`make_clamp <opendp.transformations.make_clamp>` 
* :py:func:`make_resize <opendp.transformations.make_resize>`.
See the section on :ref:`transformations-user-guide` for more information on how to preprocess data in OpenDP.

Composition
-----------

OpenDP has several compositors for making multiple releases on the same dataset:

.. list-table::
   :header-rows: 1

   * - Function
     - Description
   * - :func:`make_composition <opendp.combinators.make_composition>`
     - Non-interactive
   * - :func:`make_adaptive_composition <opendp.combinators.make_adaptive_composition>`
     - Interactive

Composition combinators can compose Measurements with ``ZeroConcentratedDivergence``, ``MaxDivergence`` and ``FixedSmoothedMaxDivergence`` output measures,
and arbitrary input metrics and domains.

See the notebook for code examples and more thorough explanations:

.. toctree::
   :glob:
   :titlesonly:

   compositors

.. _measure-casting:

Measure Casting
---------------
These combinators are used to cast the output measure of a Measurement.

.. list-table::
   :header-rows: 1

   * - Input Measure
     - Output Measure
     - Constructor
   * - ``MaxDivergence``
     - ``Approximate<MaxDivergence>``
     - :func:`opendp.combinators.make_approximate`
   * - ``ZeroConcentratedDivergence``
     - ``Approximate<ZeroConcentratedDivergence>``
     - :func:`opendp.combinators.make_approximate`
   * - ``MaxDivergence``
     - ``SmoothedMaxDivergence``
     - :func:`opendp.combinators.make_fixed_approxDP_to_approxDP`
   * - ``MaxDivergence``
     - ``ZeroConcentratedDivergence``
     - :func:`opendp.combinators.make_pureDP_to_zCDP`
   * - ``ZeroConcentratedDivergence``
     - ``SmoothedMaxDivergence``
     - :func:`opendp.combinators.make_zCDP_to_approxDP`
   * - ``SmoothedMaxDivergence``
     - ``Approximate<MaxDivergence>``
     - :func:`opendp.combinators.make_fix_delta`

:func:`opendp.combinators.make_approximate` is used for casting an output measure from ``MaxDivergence`` to ``Approximate<MaxDivergence>``.
This is useful if you want to compose pure-DP measurements with approximate-DP measurements.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> input_space = dp.atom_domain(
        ...     T=float, nan=False
        ... ), dp.absolute_distance(T=float)
        >>> meas_pureDP = input_space >> dp.m.then_laplace(scale=10.0)
        >>> # convert the output measure to `Approximate<MaxDivergence>`
        >>> meas_fixed_approxDP = dp.c.make_approximate(meas_pureDP)
        >>> # `Approximate<MaxDivergence>` distances are (ε, δ) tuples
        >>> meas_fixed_approxDP.map(d_in=1.0)
        (0.1, 0.0)

The combinator can also be used on measurements with a ``ZeroConcentratedDivergence`` privacy measure.

:func:`opendp.combinators.make_pureDP_to_zCDP` is used for casting an output measure from ``MaxDivergence`` to ``ZeroConcentratedDivergence``.
:func:`opendp.combinators.make_zCDP_to_approxDP` is used for casting an output measure from ``ZeroConcentratedDivergence`` to ``SmoothedMaxDivergence``.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> meas_zCDP = input_space >> dp.m.then_gaussian(scale=0.5)
        >>> # convert the output measure to `SmoothedMaxDivergence`
        >>> meas_approxDP = dp.c.make_zCDP_to_approxDP(meas_zCDP)
        >>> # SmoothedMaxDivergence distances are privacy profiles (ε(δ) curves)
        >>> profile = meas_approxDP.map(d_in=1.0)
        >>> profile.epsilon(delta=1e-6)
        11.688596249354896

:func:`opendp.combinators.make_fix_delta` changes the output measure from ``SmoothedMaxDivergence`` to ``Approximate<MaxDivergence>``.
It fixes the delta parameter in the curve, so that the resulting measurement can be composed with other ``Approximate<MaxDivergence>`` measurements.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> # convert the output measure to `FixedSmoothedMaxDivergence`
        >>> meas_fixed_approxDP = dp.c.make_fix_delta(
        ...     meas_approxDP, delta=1e-8
        ... )
        >>> # FixedSmoothedMaxDivergence distances are (ε, δ) tuples
        >>> meas_fixed_approxDP.map(d_in=1.0)
        (13.3861046488579, 1e-08)

These last two combinators allow you to convert output distances in terms of ρ-zCDP to ε(δ)-approxDP, and then to (ε, δ)-approxDP.


Amplification
-------------

If your dataset is a simple sample from a larger population,
you can make the privacy relation more permissive by wrapping your measurement with a privacy amplification combinator:
:func:`opendp.combinators.make_population_amplification`.

.. note::

    The amplifier requires a looser trust model, as the population size can be set arbitrarily.

    .. code:: pycon


        >>> dp.enable_features("honest-but-curious")


In order to demonstrate this API, we'll first create a measurement with a sized input domain.
The resulting measurement expects the size of the input dataset to be 10.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> atom_domain = dp.atom_domain(bounds=(0.0, 10.0), nan=False)
        >>> input_space = (
        ...     dp.vector_domain(atom_domain, size=10),
        ...     dp.symmetric_distance(),
        ... )
        >>> meas = (
        ...     input_space
        ...     >> dp.t.then_mean()
        ...     >> dp.m.then_laplace(scale=0.5)
        ... )
        >>> print(
        ...     "standard mean:", meas([1.0] * 10)
        ... )  # -> 1.03 # doctest: +ELLIPSIS
        standard mean: ...

We can now use the amplification combinator to construct an amplified measurement.
The function on the amplified measurement is identical to the standard measurement.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon
      
      >>> amplified = dp.c.make_population_amplification(
      ...     meas, population_size=100
      ... )
      >>> print(
      ...     "amplified mean:", amplified([1.0] * 10)
      ... )  # -> .97 # doctest: +ELLIPSIS
      amplified mean: ...

The privacy relation on the amplified measurement takes into account that the input dataset of size 10
is a simple sample of individuals from a theoretical larger dataset that captures the entire population, with 100 rows.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> # Where we once had a privacy utilization of ~2 epsilon...
        >>> assert meas.check(2, 2.0 + 1e-6)
        >>> # ...we now have a privacy utilization of ~.4941 epsilon.
        >>> assert amplified.check(2, 0.4941)

The efficacy of this combinator improves as n gets larger.

