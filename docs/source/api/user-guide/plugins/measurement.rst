Measurement example
===================

Use :func:`opendp.measurements.make_user_measurement` to construct a measurement for your own mechanism.

.. note::

    This requires a looser trust model, as we cannot verify any privacy or stability properties of user-defined functions.

    .. tab-set::

      .. tab-item:: Python

        .. code:: python

            >>> import opendp.prelude as dp
            >>> dp.enable_features("honest-but-curious", "contrib")

This example mocks the typical API of the OpenDP library to make the *most private* DP mechanism ever!

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> def make_base_constant(constant):
        ...     """Constructs a Measurement that only returns a constant value."""
        ...     def function(_arg: int):
        ...         return constant
        ...
        ...     def privacy_map(d_in: int) -> float:
        ...         return 0.0
        ...
        ...     return dp.m.make_user_measurement(
        ...         input_domain=dp.atom_domain(T=int),
        ...         input_metric=dp.absolute_distance(T=int),
        ...         output_measure=dp.max_divergence(),
        ...         function=function,
        ...         privacy_map=privacy_map,
        ...         TO=type(constant),  # the expected type of the output
        ...     )

The resulting Measurement may be used interchangeably with those constructed via the library:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> meas = (
        ...     (dp.vector_domain(dp.atom_domain((0, 10))), dp.symmetric_distance())
        ...     >> dp.t.then_sum()
        ...     >> make_base_constant("denied")
        ... )
        ...
        >>> meas([2, 3, 4])
        'denied'
        >>> meas.map(1) # computes epsilon, because the output measure is max divergence
        0.0

While this mechanism clearly has no utility,
the code snip may form a basis for you to create own measurements,
or even incorporate mechanisms from other libraries.
