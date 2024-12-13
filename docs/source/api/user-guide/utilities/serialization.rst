Serialization
=============

The core classes in OpenDP can be serialized to JSON, and then reinstantiated.
This feature requires opt-in, and is only available in Python.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import opendp.prelude as dp
        >>> dp.enable_features('serialization')

        >>> sum_trans = dp.t.make_sum(
        ...     dp.vector_domain(dp.atom_domain(bounds=(0, 1))),
        ...     dp.symmetric_distance()
        ... )
        >>> laplace = dp.m.make_laplace(
        ...     sum_trans.output_domain,
        ...     sum_trans.output_metric,
        ...     scale=1.0
        ... )
        >>> noisy_sum = sum_trans >> laplace
        >>> noisy_sum
        Measurement(
            input_domain   = VectorDomain(AtomDomain(bounds=[0, 1], T=i32)),
            input_metric   = SymmetricDistance(),
            output_measure = MaxDivergence)

        >>> noisy_sum_json = noisy_sum.to_json()
        >>> noisy_sum_json[:5]
        ???

        >>> new_noisy_sum = dp.make_load_json(noisy_sum_json)
        >>> assert str(noisy_sum) == str(new_noisy_sum)

:py:mod:`opendp.context` and :py:mod:`opendp.extras` are not currently supported.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> context = dp.Context.compositor(
      ...     data=[],
      ...     privacy_unit=dp.unit_of(contributions=1),
      ...     privacy_loss=dp.loss_of(epsilon=1.0),
      ...     split_evenly_over=1,
      ... )
      >>> context.to_json()
      ???
