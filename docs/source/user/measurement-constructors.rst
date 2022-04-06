.. _measurement-constructors:

Measurement Constructors
========================

This section gives a high-level overview of the measurements that are available in the library.
Refer to the :ref:`measurement` section for an explanation of what a measurement is.

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
   * - :func:`opendp.meas.make_base_geometric`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.meas.make_base_geometric`
     - ``VectorDomain<AllDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.meas.make_base_laplace`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.meas.make_base_laplace`
     - ``VectorDomain<AllDomain<T>>``
     - ``L1Distance<T>``
     - ``MaxDivergence<T>``
   * - :func:`opendp.meas.make_base_gaussian`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``SmoothedMaxDivergence<T>``
   * - :func:`opendp.meas.make_base_gaussian`
     - ``VectorDomain<AllDomain<T>>``
     - ``L2Distance<T>``
     - ``SmoothedMaxDivergence<T>``
   * - :func:`opendp.meas.make_base_analytic_gaussian`
     - ``AllDomain<T>``
     - ``AbsoluteDistance<T>``
     - ``SmoothedMaxDivergence<T>``
   * - :func:`opendp.meas.make_base_analytic_gaussian`
     - ``VectorDomain<AllDomain<T>>``
     - ``L2Distance<T>``
     - ``SmoothedMaxDivergence<T>``
   * - :func:`opendp.meas.make_base_ptr`
     - ``MapDomain<AllDomain<TIA>, AllDomain<TOA>>``
     - ``L1Distance<T>``
     - ``SmoothedMaxDivergence<T>``
   * - :func:`opendp.meas.make_randomized_response_bool`
     - ``AllDomain<bool>``
     - ``SymmetricDistance``
     - ``MaxDivergence<T>``
   * - :func:`opendp.meas.make_randomized_response`
     - ``AllDomain<T>``
     - ``SymmetricDistance``
     - ``MaxDivergence<T>``

.. _floating-point:

Floating-Point
--------------

Given the context of measurements, this section goes into greater detail than :ref:`limitations` on floating-point issues.
Be warned that :func:`opendp.meas.make_base_laplace`, :func:`opendp.meas.make_base_gaussian` and :func:`opendp.meas.make_base_ptr`
depend on continuous distributions that are poorly approximated by finite computers.

At this time these mechanisms are present in the library, but require explicit opt-in:

.. doctest::

    >>> from opendp.mod import enable_features
    >>> enable_features("floating-point")

The canonical paper on this and introduction of the snapping mechanism is here:
`On Significance of the Least Significant Bits For Differential Privacy <https://www.microsoft.com/en-us/research/wp-content/uploads/2012/10/lsbs.pdf>`_.

Precautions have been made to sample noise using the GNU MPFR library in a way
that provides cryptographically secure, non-porous noise at standard scale.
Noise at arbitrary scale is achieved through a combination of preprocessing and postprocessing
that preserves the properties of differential privacy.

Precautions have also been made to explicitly specify floating-point rounding modes
in such a way that the privacy budget is always slightly overestimated.

We acknowledge the snapping mechanism and have an implementation of it `in PR #84 <https://github.com/opendp/opendp/pull/84>`_.

We are also working towards adding support for fixed-point data types `in PR #184 <https://github.com/opendp/opendp/pull/184>`_.
