.. _measurement-constructors:

Measurements
============

This section gives a high-level overview of the measurements that are available in the library.
Refer to the :ref:`measurement` section for an explanation of what a measurement is.

Central DP
----------

As covered in the :ref:`chaining` section, the intermediate domains and metrics need to match when chaining.
This means you will need to choose a measurement that chains with your :ref:`aggregator <aggregators>`.

In the following table, the scalar-valued and vector-valued versions of each measurement are listed separately.
You can choose whether to construct scalar or vector-valued versions by setting the ``D`` type argument when calling the constructor.

:Scalar: ``D=AllDomain[T]`` (default)
:Vector: ``D=VectorDomain[AllDomain[T]]``

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_base_discrete_laplace`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.measurements.make_base_discrete_laplace`
     - ``VectorDomain<AllDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.measurements.make_base_laplace`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.measurements.make_base_laplace`
     - ``VectorDomain<AllDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.measurements.make_base_discrete_gaussian`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.measurements.make_base_discrete_gaussian`
     - ``VectorDomain<AllDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.measurements.make_base_gaussian`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``SmoothedMaxDivergence<T>``
   * - :func:`opendp.measurements.make_base_gaussian`
     - ``VectorDomain<AllDomain<T>>``
     - ``L2Distance<T>``
     - ``SmoothedMaxDivergence<T>``
   * - :func:`opendp.measurements.make_base_ptr`
     - ``MapDomain<AllDomain<TIA>, AllDomain<TOA>>``
     - ``L1Distance<T>``
     - ``SmoothedMaxDivergence<T>``


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


Local DP
--------

.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_randomized_response_bool`
     - ``AllDomain<bool>``
     - ``DiscreteDistance``
     - ``MaxDivergence<T>``
   * - :func:`opendp.measurements.make_randomized_response`
     - ``AllDomain<T>``
     - ``DiscreteDistance``
     - ``MaxDivergence<T>``
