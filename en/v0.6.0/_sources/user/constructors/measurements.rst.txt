.. _measurement-constructors:

Measurements
============

This section gives a high-level overview of the measurements that are available in the library.
Refer to the :ref:`measurement` section for an explanation of what a measurement is.

As covered in the :ref:`chaining` section, the intermediate domains and metrics need to match when chaining.
This means you will need to choose a measurement that chains with your :ref:`aggregator <aggregators>`.

In the following sections, scalar-valued and vector-valued versions of each measurement are listed separately.
You can choose whether to construct scalar or vector-valued versions by setting the ``D`` type argument when calling the constructor.

:Scalar: ``D=AllDomain[T]`` (default)
:Vector: ``D=VectorDomain[AllDomain[T]]``

Notice that there is a symmetric structure to the core measurements:

.. list-table::
   :header-rows: 1

   * - Input Metric
     - Integer
     - Float
   * - ``L1Distance<T>``
     - :func:`make_base_discrete_laplace <opendp.measurements.make_base_discrete_laplace>`
     - :func:`make_base_laplace <opendp.measurements.make_base_laplace>`
   * - ``L2Distance<T>``
     - :func:`make_base_discrete_gaussian <opendp.measurements.make_base_discrete_gaussian>`
     - :func:`make_base_gaussian <opendp.measurements.make_base_gaussian>`


Laplacian Noise
---------------

These algorithms accept L1 sensitivities and measure privacy in terms of epsilon. 
Use the :func:`opendp.accuracy.laplacian_scale_to_accuracy` and :func:`opendp.accuracy.accuracy_to_laplacian_scale` functions to convert to/from accuracy estimates.

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_base_discrete_laplace`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_laplace`
     - ``VectorDomain<AllDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_laplace`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.measurements.make_base_laplace`
     - ``VectorDomain<AllDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence<T>``


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
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_laplace_cks20`
     - ``VectorDomain<AllDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_laplace_linear`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_laplace_linear`
     - ``VectorDomain<AllDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence<QO>``

.. raw:: html

   </details>


Gaussian Noise
--------------

These algorithms accept L2 sensitivities and measure privacy in terms of rho (zero-concentrated differential privacy). 
Use the :func:`opendp.accuracy.gaussian_scale_to_accuracy` and :func:`opendp.accuracy.accuracy_to_gaussian_scale` functions to convert to/from accuracy estimates.
Refer to :ref:`measure-casting` to convert to approximate DP.

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_base_discrete_gaussian`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<QI>``
     - ``ZeroConcentratedDivergence<QO>``
   * - :func:`opendp.measurements.make_base_discrete_gaussian`
     - ``VectorDomain<AllDomain<T>>``
     - ``L2Distance<QI>``
     - ``ZeroConcentratedDivergence<QO>``
   * - :func:`opendp.measurements.make_base_gaussian`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``ZeroConcentratedDivergence<T>``
   * - :func:`opendp.measurements.make_base_gaussian`
     - ``VectorDomain<AllDomain<T>>``
     - ``L2Distance<T>``
     - ``ZeroConcentratedDivergence<T>``


Geometric Noise
---------------
The geometric mechanism (:func:`make_base_geometric <opendp.measurements.make_base_geometric>`) has been deprecated in favor of the discrete laplace (:func:`make_base_discrete_laplace <opendp.measurements.make_base_discrete_laplace>`).
:func:`make_base_discrete_laplace <opendp.measurements.make_base_discrete_laplace>` is overall more computationally efficient than the previous algorithm.
If you need constant-time execution to protect against timing side-channels, use :func:`opendp.measurements.make_base_discrete_laplace_linear`, which is equivalent to the previous algorithm.


Stability Histogram
-------------------
The stability histogram is used to release a category set and frequency counts, and is useful when the category set is unknown or very large.
`make_count_by` is included here because it is currently the only transformation that `make_base_ptr` chains with.

.. list-table::
   :header-rows: 1

   * - Constructor
     - Input Domain
     - Input Metric
     - Output Metric/Measure
   * - :func:`opendp.transformations.make_count_by`
     - ``VectorDomain<BoundedDomain<TK>>``
     - ``SymmetricDistance``
     - ``L1Distance<TV>``
   * - :func:`opendp.measurements.make_base_ptr`
     - ``MapDomain<AllDomain<TK>, AllDomain<TV>>``
     - ``L1Distance<TV>``
     - ``SmoothedMaxDivergence<TV>``

Randomized Response
-------------------
These measurements are used to randomize an individual's response to a query. 

.. testsetup::

    from opendp.mod import enable_features
    enable_features('contrib', 'floating-point')

.. doctest::

    >>> from opendp.measurements import make_randomized_response_bool
    >>> meas = make_randomized_response_bool(prob=0.75)
    >>> release = meas(True)
    >>> epsilon = meas.map(1)

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_randomized_response_bool`
     - ``AllDomain<bool>``
     - ``DiscreteDistance``
     - ``MaxDivergence<QO>``
   * - :func:`opendp.measurements.make_randomized_response`
     - ``AllDomain<T>``
     - ``DiscreteDistance``
     - ``MaxDivergence<QO>``
