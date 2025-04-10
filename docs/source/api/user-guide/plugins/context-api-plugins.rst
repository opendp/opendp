Context API Plugins
===================

Constructor functions built from plugins can be registered in the Context API.

We'll start by briefly copying over the constructor from the :ref:`measurement-plugin` and registering it.
The constructor function may now be called from the Context API.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> def make_int_constant(constant):
        ...     return dp.m.make_user_measurement(
        ...         input_domain=dp.atom_domain(T=int),
        ...         input_metric=dp.absolute_distance(T=int),
        ...         output_measure=dp.max_divergence(),
        ...         function=lambda _: constant,
        ...         privacy_map=lambda _: 0.0,
        ...     )
        ...
        >>> dp.register(make_int_constant)
        ...
        >>> context = dp.Context.compositor(
        ...     data=[1, 2, 3],
        ...     privacy_unit=dp.unit_of(contributions=36),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=2,
        ... )
        >>> context.query().clamp((0, 5)).sum().int_constant("denied").release()
        'denied'

A drawback of this approach is that the constructor function is not very flexible.
The input domain and metric are hard-coded, only accepting integers, 
and can't take into account the output domain and output metric of the previous transformation.

If the first two arguments to the constructor function are the ``input_domain`` and ``input_metric``,
then they can be omitted when you call the function from the context API, and will be passed up from the previous transformation.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> def make_anything_constant(input_domain, input_metric, constant):
        ...     return dp.m.make_user_measurement(
        ...         input_domain=input_domain,
        ...         input_metric=input_metric,
        ...         output_measure=dp.max_divergence(),
        ...         function=lambda _: constant,
        ...         privacy_map=lambda _: 0.0,
        ...     )
        ...
        >>> dp.register(make_anything_constant)
        >>> context.query().anything_constant("denied").release()
        'denied'

This modified constructor function doesn't care what the input domain and metric are,
and will happily build a measurement that will always conform with the previous transformation.
In practice, the constructor function usually contains checks to ensure that the input domain and metric are meaningful for your function.

Constructors in the OpenDP Library almost always accept the input domain and metric as the first two arguments,
and we recommend it when building your own plugins.
