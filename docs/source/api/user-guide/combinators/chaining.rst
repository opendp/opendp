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
