.. _measurement-constructors:

Measurements
============

This section gives a high-level overview of the measurements that are available in the library.
Refer to the :ref:`section on measurments <measurements-user-guide>` in the discussion of core structures
for a defintion of "measurement" in OpenDP.

The intermediate domains and metrics need to match when chaining.
This means you will need to choose a measurement that chains with your :ref:`aggregator <aggregators>`.


.. toctree::
  :hidden:

  additive-noise-mechanisms
  randomized-response

Additive Noise Mechanisms
-------------------------

See the `Additive Noise Mechanisms notebook <additive-noise-mechanisms.html>`_ for code examples and more exposition.

Notice that there is a symmetric structure to the additive noise measurements:

.. list-table::
   :header-rows: 1

   * - Vector Input Metric
     - Constructor
   * - ``L1Distance<T>``
     - :func:`make_laplace <opendp.measurements.make_laplace>`
   * - ``L2Distance<T>``
     - :func:`make_gaussian <opendp.measurements.make_gaussian>`

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
   * - :func:`opendp.measurements.make_laplace`
     - ``AtomDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence``
   * - :func:`opendp.measurements.make_laplace`
     - ``VectorDomain<AtomDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence``


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
   * - :func:`opendp.measurements.make_gaussian`
     - ``AtomDomain<T>``
     - ``AbsoluteDistance<QI>``
     - ``ZeroConcentratedDivergence``
   * - :func:`opendp.measurements.make_gaussian`
     - ``VectorDomain<AtomDomain<T>>``
     - ``L2Distance<QI>``
     - ``ZeroConcentratedDivergence``


Geometric Noise
***************
The geometric mechanism (:func:`make_geometric <opendp.measurements.make_geometric>`) is an alias for the discrete Laplace (:func:`make_laplace <opendp.measurements.make_laplace>`).
If you need constant-time execution to protect against timing side-channels, specify bounds!


Noise Addition with Thresholding
---------------------------------
When releasing data grouped by an unknown key-set,
it is necessary to use a mechanism that only releases keys which are "stable".
That is, keys which are present among all neighboring datasets.

The stability histogram is used to release a category set and frequency counts, and is useful when the category set is unknown or very large.
``make_count_by`` is included here because it is currently the only transformation that ``make_laplace_threshold`` chains with.

See the `Histograms notebook <../../../getting-started/examples/histograms.html>`_ for code examples and more exposition.

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
   * - :func:`opendp.measurements.make_laplace_threshold`
     - ``MapDomain<AtomDomain<TK>, AtomDomain<TV>>``
     - ``L1Distance<TV>``
     - ``SmoothedMaxDivergence``

Randomized Response
-------------------
These measurements are used to randomize an individual's response to a query in the local-DP model.

See the `Randomized Response notebook <randomized-response.html>`_ for code examples and more exposition.


.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_randomized_response_bool`
     - ``AtomDomain<bool>``
     - ``DiscreteDistance``
     - ``MaxDivergence``
   * - :func:`opendp.measurements.make_randomized_response`
     - ``AtomDomain<T>``
     - ``DiscreteDistance``
     - ``MaxDivergence``
