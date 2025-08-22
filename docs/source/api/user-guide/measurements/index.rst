.. _measurement-constructors:

Measurements
============

This section gives a high-level overview of the measurements that are available in the library.
Refer to the :ref:`section on measurements <measurements-user-guide>` in the discussion of core structures
for a defintion of "measurement" in OpenDP.

The intermediate domains and metrics need to match when chaining.
This means you will need to choose a measurement that chains with your :ref:`aggregator <aggregators>`.


.. toctree::
  :hidden:

  additive-noise-mechanisms
  thresholded-noise-mechanisms
  canonical-noise-mechanism
  randomized-response

Additive Noise Mechanisms
-------------------------

See the `Additive Noise Mechanisms notebook <additive-noise-mechanisms.html>`_ for code examples and more exposition.

Notice that there is a symmetric structure to the additive noise measurements:

.. list-table::
   :header-rows: 1

   * - Vector Input Metric
     - Constructor
   * - ``L1Distance<QI>``
     - :func:`make_laplace <opendp.measurements.make_laplace>`
   * - ``L2Distance<QI>``
     - :func:`make_gaussian <opendp.measurements.make_gaussian>`
    
`QI` can be any numeric type (the data type of the sensitivity can vary independently from the data type of the input).

In the following sections, scalar-valued and vector-valued versions of each measurement are listed separately.
You can choose whether to construct scalar or vector-valued versions of mechanisms by passing the appropriate input space.
See the notebook above for examples.

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
     - ``AbsoluteDistance<QI>``
     - ``MaxDivergence``
   * - :func:`opendp.measurements.make_laplace`
     - ``VectorDomain<AtomDomain<T>>``
     - ``L1Distance<QI>``
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


Canonical Noise
***************
The canonical noise mechanism (:func:`opendp.measurements.make_canonical_noise`) is discussed in detail in `these examples <canonical-noise-mechanism.html>`_


Thresholded Noise Mechanisms
----------------------------

Thresholded noise mechanisms are generalizations of the additive noise mechanisms
that also release a set of keys, whose values are greater than the ``threshold`` parameter.

See the `Thresholded Noise Mechanisms documentation <thresholded-noise-mechanisms.html>`_ for code examples and more exposition.

Just like the additive noise mechanisms, the thresholded noise mechanisms have a symmetric structure:

.. list-table::
   :header-rows: 1

   * - Vector Input Metric
     - Constructor
   * - ``L01InfDistance<AbsoluteDistance<T>>``
     - :func:`make_laplace_threshold <opendp.measurements.make_laplace_threshold>`
   * - ``L02InfDistance<AbsoluteDistance<T>>``
     - :func:`make_gaussian_threshold <opendp.measurements.make_gaussian_threshold>`

Laplacian Noise
***************

The algorithm accepts ``L0``, ``L1`` and ``L∞`` sensitivities and measures privacy in terms of epsilon and delta. 
Use the :func:`opendp.accuracy.laplacian_scale_to_accuracy` and :func:`opendp.accuracy.accuracy_to_laplacian_scale` 
functions to convert to/from accuracy estimates.

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_laplace_threshold`
     - ``MapDomain<AtomDomain<TK>, AtomDomain<TV>>``
     - ``L01InfDistance<AbsoluteDistance<QI>>``
     - ``Approximate<MaxDivergence>``


Gaussian Noise
**************

The algorithm accepts ``L0``, ``L2`` and ``L∞`` sensitivities and measures privacy in terms of rho and delta. 
Use the :func:`opendp.accuracy.gaussian_scale_to_accuracy` and :func:`opendp.accuracy.accuracy_to_gaussian_scale` 
functions to convert to/from accuracy estimates.
Refer to :ref:`measure-casting` to convert to approximate DP.

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_gaussian_threshold`
     - ``MapDomain<AtomDomain<TK>, AtomDomain<TV>>``
     - ``L02InfDistance<AbsoluteDistance<QI>>``
     - ``Approximate<ZeroConcentratedDivergence>``


Noisy Top K and Noisy Max
-------------------------

The report noisy top-k mechanism is used to privately release the indices of the maximum k values in a vector.
This is useful for private selection, and overlaps with the exponential mechanism.
Exponential noise is added to scores when the output measure is ``MaxDivergence``,
and Gumbel noise is added when the output measure is ``ZeroConcentratedDivergence``. 

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_noisy_top_k`
     - ``VectorDomain<AtomDomain<T>>``
     - ``LInfDistance<T>``
     - ``MaxDivergence`` or ``ZeroConcentratedDivergence``
   * - :func:`opendp.measurements.make_noisy_max`
     - ``VectorDomain<AtomDomain<T>>``
     - ``LInfDistance<T>``
     - ``MaxDivergence`` or ``ZeroConcentratedDivergence``

Report noisy max is a special case of noisy top k when k equals one.

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
