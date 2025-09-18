.. _measurement-constructors:

Measurements
============

This page is a high-level overview of the measurements that are available in OpenDP.
In OpenDP, measurements are randomized mappings from datasets to outputs;
Measurements are used to create differentially private releases.

The intermediate domains and metrics need to match when chaining.
This means you will need to choose a measurement that chains with your :ref:`aggregator <aggregators>`.


Additive Noise Mechanisms
-------------------------

There is a symmetric structure to the additive noise measurements:

.. list-table::
   :header-rows: 1

   * - Vector Input Metric
     - Constructor
   * - ``L1Distance<QI>``
     - :func:`make_laplace <opendp.measurements.make_laplace>`
   * - ``L2Distance<QI>``
     - :func:`make_gaussian <opendp.measurements.make_gaussian>`

``QI`` can be any numeric type (the data type of the sensitivity can vary independently from the data type of the input).

By passing the appropriate input space,
you can construct either scalar or vector-valued mechanisms.

More details on `laplacian and gaussian noise mechanisms here <additive-noise-mechanisms.html>`_.

.. toctree::
  :hidden:

  additive-noise-mechanisms


Laplacian Noise
***************

:func:`make_laplace <opendp.measurements.make_laplace>` accepts sensitivities in terms of the absolute or L2 metrics and measure privacy in terms of epsilon. 
Use :func:`laplacian_scale_to_accuracy <opendp.accuracy.laplacian_scale_to_accuracy>`
and :func:`accuracy_to_laplacian_scale <opendp.accuracy.accuracy_to_laplacian_scale>` to convert to/from accuracy estimates.
(:func:`make_geometric <opendp.measurements.make_geometric>`
is equivalent to ``make_laplace`` but restricted to an integer support.
If you need constant-time execution to protect against timing side-channels, specify bounds.)

.. list-table::
   :header-rows: 1

   * - Input Domain
     - Input Metric
     - Output Measure
   * - ``AtomDomain<T>``
     - ``AbsoluteDistance<QI>``
     - ``MaxDivergence``
   * - ``VectorDomain<AtomDomain<T>>``
     - ``L1Distance<QI>``
     - ``MaxDivergence``


Gaussian Noise
**************

:func:`make_gaussian <opendp.measurements.make_gaussian>` accepts sensitivities in terms of the absolute or L2 metrics and measure privacy in terms of rho (zero-concentrated differential privacy). 
Use :func:`gaussian_scale_to_accuracy <opendp.accuracy.gaussian_scale_to_accuracy>` and
:func:`accuracy_to_gaussian_scale <opendp.accuracy.accuracy_to_gaussian_scale>` to convert to/from accuracy estimates.
(Refer to :ref:`measure-casting` to convert to approximate DP.)

.. list-table::
   :header-rows: 1

   * - Input Domain
     - Input Metric
     - Output Measure
   * - ``AtomDomain<T>``
     - ``AbsoluteDistance<QI>``
     - ``ZeroConcentratedDivergence``
   * - ``VectorDomain<AtomDomain<T>>``
     - ``L2Distance<QI>``
     - ``ZeroConcentratedDivergence``

Canonical Noise
***************

The canonical noise mechanism (:func:`make_canonical_noise <opendp.measurements.make_canonical_noise>`)
can privatize any float-valued statistic with finite sensitivity.
Under :math:`(\epsilon, \delta)`-DP,
the canonical noise distribution follows the Tulap distribution,
which is a combination of discrete laplace noise and continuous uniform noise.

More details on `the canonical noise mechanism here <canonical-noise-mechanism.html>`_.

.. toctree::
  :hidden:

  canonical-noise-mechanism


Thresholded Noise Mechanisms
----------------------------

Thresholded noise mechanisms are generalizations of the additive noise mechanisms
that also release a set of keys, whose values are greater than the ``threshold`` parameter.

Just like the additive noise mechanisms, the thresholded noise mechanisms have a symmetric structure:

.. list-table::
   :header-rows: 1

   * - Vector Input Metric
     - Constructor
   * - ``L01InfDistance<AbsoluteDistance<T>>``
     - :func:`make_laplace_threshold <opendp.measurements.make_laplace_threshold>`
   * - ``L02InfDistance<AbsoluteDistance<T>>``
     - :func:`make_gaussian_threshold <opendp.measurements.make_gaussian_threshold>`

More details on `thresholded noise mechanisms here <thresholded-noise-mechanisms.html>`_.

.. toctree::
  :hidden:

  thresholded-noise-mechanisms

Thresholded Laplacian Noise
***************************

:func:`make_laplace_threshold <opendp.measurements.make_laplace_threshold>` accepts ``L0``, ``L1`` and ``L∞`` sensitivities and measures privacy in terms of epsilon and delta. 
Use the :func:`opendp.accuracy.laplacian_scale_to_accuracy` and :func:`opendp.accuracy.accuracy_to_laplacian_scale` 
functions to convert to/from accuracy estimates.

.. list-table::
   :header-rows: 1

   * - Input Domain
     - Input Metric
     - Output Measure
   * - ``MapDomain<AtomDomain<TK>, AtomDomain<TV>>``
     - ``L01InfDistance<AbsoluteDistance<QI>>``
     - ``Approximate<MaxDivergence>``


Thresholded Gaussian Noise
**************************

:func:`make_gaussian_threshold <opendp.measurements.make_gaussian_threshold>` accepts ``L0``, ``L2`` and ``L∞`` sensitivities and measures privacy in terms of rho and delta. 
Use the :func:`opendp.accuracy.gaussian_scale_to_accuracy` and :func:`opendp.accuracy.accuracy_to_gaussian_scale` 
functions to convert to/from accuracy estimates.
Refer to :ref:`measure-casting` to convert to approximate DP.

.. list-table::
   :header-rows: 1

   * - Input Domain
     - Input Metric
     - Output Measure
   * - ``MapDomain<AtomDomain<TK>, AtomDomain<TV>>``
     - ``L02InfDistance<AbsoluteDistance<QI>>``
     - ``Approximate<ZeroConcentratedDivergence>``

Approximate Laplace Projection
------------------------------

In a similar regime as the thresholded noise mechanism, where keys themselves need to be protected,
another approach is to release a differentially private low-dimensional projection of the key-space.


.. list-table::
   :header-rows: 1

   * - Measurement
     - Input Domain
     - Input Metric
     - Output Measure
   * - :func:`opendp.measurements.make_alp_queryable`
     - ``MapDomain<AtomDomain<TK>, AtomDomain<TV>>``
     - ``L01InfDistance<AbsoluteDistance<QI>>``
     - ``MaxDivergence``

More details on :ref:`approximate laplace projection here <approximate-laplace-projection>`.

.. toctree::
  :hidden:

  approximate-laplace-projection

Noisy Max and Noisy Top K
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

More details on `noisy max mechanisms here <noisy-max-mechanisms.html>`_.

.. toctree::
  :hidden:

  noisy-max-mechanisms

Randomized Response
-------------------
These measurements are used to randomize an individual's response to a query in the local-DP model.

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
   * - :func:`opendp.measurements.make_randomized_response_bitvec`
     - ``AtomDomain<T>``
     - ``DiscreteDistance``
     - ``MaxDivergence``

More details on `randomized response here <randomized-response.html>`_.

.. toctree::
  :hidden:

  randomized-response
