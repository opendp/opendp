Plugins
=======

Because Differential Privacy is a wide and expanding field,
we can't implement every mechanism for every user,
but users can provide their own code through these methods:

Domains
    :py:func:`user_domain <opendp.domains.user_domain>`
Measurements
    :py:func:`make_user_measurement <opendp.measurements.make_user_measurement>` and :py:func:`then_user_measurement <opendp.measurements.then_user_measurement>`
Measures
    :py:func:`user_divergence <opendp.measures.user_divergence>`
Metrics
    :py:func:`user_distance <opendp.metrics.user_distance>`
Transformations
    :py:func:`make_user_transform <opendp.transformations.make_user_transformation>`
Postprocessors
    :py:func:`new_function <opendp.core.new_function>`

OpenDP itself uses the plugin machinery in some cases.
It is usually easier to implement ideas in Python or R than in Rust,
so this provides a lower barrier to entry to contributing to OpenDP.
If the contribution proves to be useful to the broader community,
it can then be translated to Rust.

Measurement example
-------------------

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
        ...         output_measure=dp.max_divergence(T=float),
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

Transformation example
----------------------

Use :func:`opendp.transformations.make_user_transformation` to construct your own transformation.

.. note::

    This requires a looser trust model, as we cannot verify any privacy or stability properties of user-defined functions.

    .. code:: python

        >>> import opendp.prelude as dp
        >>> dp.enable_features("honest-but-curious")

In this example, we mock the typical API of the OpenDP library to make a transformation that duplicates each record `multiplicity` times:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import opendp.prelude as dp
        >>> from typing import List
        ...
        >>> def make_repeat(multiplicity):
        ...     """Constructs a Transformation that duplicates each record `multiplicity` times"""
        ...     def function(arg: List[int]) -> List[int]:
        ...         return arg * multiplicity
        ... 
        ...     def stability_map(d_in: int) -> int:
        ...         # if a user could influence at most `d_in` records before, 
        ...         # they can now influence `d_in` * `multiplicity` records
        ...         return d_in * multiplicity
        ...
        ...     return dp.t.make_user_transformation(
        ...         input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        ...         input_metric=dp.symmetric_distance(),
        ...         output_domain=dp.vector_domain(dp.atom_domain(T=int)),
        ...         output_metric=dp.symmetric_distance(),
        ...         function=function,
        ...         stability_map=stability_map,
        ...     )
    
The resulting Transformation may be used interchangeably with those constructed via the library:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> trans = (
        ...     (dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance())
        ...     >> dp.t.then_cast_default(TOA=int)
        ...     >> make_repeat(2)  # our custom transformation
        ...     >> dp.t.then_clamp((1, 2))
        ...     >> dp.t.then_sum()
        ...     >> dp.m.then_laplace(1.0)
        ... )
        ...
        >>> release = trans(["0", "1", "2", "3"])
        >>> trans.map(1) # computes epsilon
        4.0

The code snip may form a basis for you to create your own data transformations, 
and mix them into an OpenDP analysis.

Longer examples
---------------

.. toctree::
  :titlesonly:

  selecting-grouping-columns