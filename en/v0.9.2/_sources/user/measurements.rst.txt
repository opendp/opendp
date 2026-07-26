.. _measurement-constructors:

Measurements
============

This section gives a high-level overview of the measurements that are available in the library.
Refer to the :ref:`measurement` section for an explanation of what a measurement is.

As mentioned in `Getting Started <getting-started.html#Chaining>`_, the intermediate domains and metrics need to match when chaining.
This means you will need to choose a measurement that chains with your :ref:`aggregator <aggregators>`.


.. toctree::

   measurements/additive-noise-mechanisms
   measurements/randomized-response
   measurements/pca

Additive Noise Mechanisms
-------------------------

See the `Additive Noise Mechanisms notebook <measurements/additive-noise-mechanisms.html>`_ for code examples and more exposition.

Notice that there is a symmetric structure to the additive noise measurements:

.. list-table::
   :header-rows: 1

   * - Vector Input Metric
     - Integer
     - Float
   * - ``L1Distance<T>``
     - :func:`make_base_discrete_laplace <opendp.measurements.make_base_discrete_laplace>`
     - :func:`make_base_laplace <opendp.measurements.make_base_laplace>`
   * - ``L2Distance<T>``
     - :func:`make_base_discrete_gaussian <opendp.measurements.make_base_discrete_gaussian>`
     - :func:`make_base_gaussian <opendp.measurements.make_base_gaussian>`

In the following sections, scalar-valued and vector-valued versions of each measurement are listed separately.
You can choose whether to construct scalar or vector-valued versions by setting the ``D`` type argument when calling the constructor.

:Scalar: ``D=AtomDomain[T]`` (default)
:Vector: ``D=VectorDomain[AtomDomain[T]]``

Laplacian Noise
***************

These algorithms accept sensitivities in terms of the absolute or L2 metrics and measure privacy in terms of epsilon. 
Use the :func:`opendp.accuracy.laplacian_scale_to_accuracy` and :func:`opendp.accuracy.accuracy_to_laplacian_scale` functions to convert to/from accuracy estimates.

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_base_discrete_laplace`
     - ``AtomDomain<int>``
     - ``AbsoluteDistance<int>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_laplace`
     - ``VectorDomain<AtomDomain<int>>``
     - ``L1Distance<int>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_laplace`
     - ``AtomDomain<float>``
     - ``AbsoluteDistance<float>``
     - ``MaxDivergence<float>``
   * - :func:`opendp.measurements.make_base_laplace`
     - ``VectorDomain<AtomDomain<float>>``
     - ``L1Distance<float>``
     - ``MaxDivergence<float>``


There are more granular versions of these constructors, should you need them:

.. raw:: html

   <details style="margin:-1em 0 2em 4em">
   <summary><a>Expand Me</a></summary>

The primary constructors above switch between the cks20 and linear-time sampling algorithms depending on the noise scale. 
If the noise scale is greater than 10, the cks20 algorithm is more efficient.
You can use these constructors to invoke the underlying algorithm directly.
In addition, the linear-time algorithm supports a constant-time execution mode if noise bounds are passed.

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_base_discrete_laplace_cks20`
     - ``AtomDomain<int>``
     - ``AbsoluteDistance<int>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_laplace_cks20`
     - ``VectorDomain<AtomDomain<int>>``
     - ``L1Distance<int>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_laplace_linear`
     - ``AtomDomain<int>``
     - ``AbsoluteDistance<int>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_laplace_linear`
     - ``VectorDomain<AtomDomain<int>>``
     - ``L1Distance<int>``
     - ``MaxDivergence<QO>``

.. raw:: html

   </details>


Gaussian Noise
**************

These algorithms accept sensitivities in terms of the absolute or L2 metrics and measure privacy in terms of rho (zero-concentrated differential privacy). 
Use the :func:`opendp.accuracy.gaussian_scale_to_accuracy` and :func:`opendp.accuracy.accuracy_to_gaussian_scale` functions to convert to/from accuracy estimates.
Refer to :ref:`measure-casting` to convert to approximate DP.

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_base_discrete_gaussian`
     - ``AtomDomain<int>``
     - ``AbsoluteDistance<QI>``
     - ``ZeroConcentratedDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_gaussian`
     - ``VectorDomain<AtomDomain<int>>``
     - ``L2Distance<QI>``
     - ``ZeroConcentratedDivergence<QO>``
   * - :func:`opendp.measurements.make_base_gaussian`
     - ``AtomDomain<float>``
     - ``AbsoluteDistance<float>``
     - ``ZeroConcentratedDivergence<float>``
   * - :func:`opendp.measurements.make_base_gaussian`
     - ``VectorDomain<AtomDomain<float>>``
     - ``L2Distance<float>``
     - ``ZeroConcentratedDivergence<float>``


Geometric Noise
***************
The geometric mechanism (:func:`make_base_geometric <opendp.measurements.make_base_geometric>`) is an alias for the discrete laplace (:func:`make_base_discrete_laplace <opendp.measurements.make_base_discrete_laplace>`).
If you need constant-time execution to protect against timing side-channels, use :func:`opendp.measurements.make_base_discrete_laplace_linear`, which is equivalent to the previous algorithm.


Noise Addition with Thresholding
---------------------------------
When releasing data grouped by an unknown key-set,
it is necessary to use a mechanism that only releases keys which are "stable".
That is, keys which are present among all neighboring datasets.

The stability histogram is used to release a category set and frequency counts, and is useful when the category set is unknown or very large.
`make_count_by` is included here because it is currently the only transformation that `make_base_laplace_threshold` chains with.

See the `Histograms notebook <../examples/histograms.html>`_ for code examples and more exposition.

.. list-table::
   :header-rows: 1

   * - Constructor
     - Input Domain
     - Input Metric
     - Output Metric/Measure
   * - :func:`opendp.transformations.make_count_by`
     - ``VectorDomain<AtomDomain<TK>>``
     - ``SymmetricDistance``
     - ``L1Distance<TV>``
   * - :func:`opendp.measurements.make_base_laplace_threshold`
     - ``MapDomain<AtomDomain<TK>, AtomDomain<TV>>``
     - ``L1Distance<TV>``
     - ``SmoothedMaxDivergence<TV>``

Randomized Response
-------------------
These measurements are used to randomize an individual's response to a query in the local-DP model.

See the `Randomized Response notebook <measurements/randomized-response.html>`_ for code examples and more exposition.


.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_randomized_response_bool`
     - ``AtomDomain<bool>``
     - ``DiscreteDistance``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_randomized_response`
     - ``AtomDomain<T>``
     - ``DiscreteDistance``
     - ``MaxDivergence<QO>``


Bring Your Own
--------------

Use :func:`opendp.measurements.make_user_measurement` to construct a measurement for your own mechanism.

.. note::

    This requires a looser trust model, as we cannot verify any privacy or stability properties of user-defined functions.

    .. doctest::

        >>> import opendp.prelude as dp
        >>> dp.enable_features("honest-but-curious", "contrib")

This example mocks the typical API of the OpenDP library to make the *most private* DP mechanism ever!

.. doctest::

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

.. doctest::

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

You can also define your own transformations (:func:`opendp.transformations.make_user_transformation`) and postprocessors (:func:`opendp.core.new_function`).
