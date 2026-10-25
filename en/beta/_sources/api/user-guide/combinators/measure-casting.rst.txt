.. _measure-casting:

Measure Casting
---------------
These combinators are used to cast the output measure of a Measurement.

.. list-table::
   :header-rows: 1

   * - Input Measure
     - Output Measure
     - Constructor
   * - ``MaxDivergence``
     - ``Approximate<MaxDivergence>``
     - :func:`~opendp.combinators.make_approximate`
   * - ``ZeroConcentratedDivergence``
     - ``Approximate<ZeroConcentratedDivergence>``
     - :func:`~opendp.combinators.make_approximate`
   * - ``MaxDivergence``
     - ``SmoothedMaxDivergence``
     - :func:`~opendp.combinators.make_fixed_approxDP_to_approxDP`
   * - ``MaxDivergence``
     - ``ZeroConcentratedDivergence``
     - :func:`~opendp.combinators.make_pureDP_to_zCDP`
   * - ``ZeroConcentratedDivergence``
     - ``SmoothedMaxDivergence``
     - :func:`~opendp.combinators.make_zCDP_to_approxDP`
   * - ``SmoothedMaxDivergence``
     - ``Approximate<MaxDivergence>``
     - :func:`~opendp.combinators.make_fix_delta`

:func:`~opendp.combinators.make_approximate` is used for casting an output measure from ``MaxDivergence`` to ``Approximate<MaxDivergence>``.
This is useful if you want to compose pure-DP measurements with approximate-DP measurements.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> input_space = dp.atom_domain(
        ...     T=float, nan=False
        ... ), dp.absolute_distance(T=float)
        >>> meas_pureDP = input_space >> dp.m.then_laplace(scale=10.0)
        >>> # convert the output measure to `Approximate<MaxDivergence>`
        >>> meas_fixed_approxDP = dp.c.make_approximate(meas_pureDP)
        >>> # `Approximate<MaxDivergence>` distances are (ε, δ) tuples
        >>> meas_fixed_approxDP.map(d_in=1.0)
        (0.1, 0.0)

The combinator can also be used on measurements with a ``ZeroConcentratedDivergence`` privacy measure.

:func:`~opendp.combinators.make_pureDP_to_zCDP` is used for casting an output measure from ``MaxDivergence`` to ``ZeroConcentratedDivergence``.
:func:`~opendp.combinators.make_zCDP_to_approxDP` is used for casting an output measure from ``ZeroConcentratedDivergence`` to ``SmoothedMaxDivergence``.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> meas_zCDP = input_space >> dp.m.then_gaussian(scale=0.5)
        >>> # convert the output measure to `SmoothedMaxDivergence`
        >>> meas_approxDP = dp.c.make_zCDP_to_approxDP(meas_zCDP)
        >>> # SmoothedMaxDivergence distances are privacy profiles (ε(δ) curves)
        >>> profile = meas_approxDP.map(d_in=1.0)
        >>> profile.epsilon(delta=1e-6)
        11.688596249354896

:func:`~opendp.combinators.make_fix_delta` changes the output measure from ``SmoothedMaxDivergence`` to ``Approximate<MaxDivergence>``.
It fixes the delta parameter in the curve, so that the resulting measurement can be composed with other ``Approximate<MaxDivergence>`` measurements.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> # convert the output measure to `FixedSmoothedMaxDivergence`
        >>> meas_fixed_approxDP = dp.c.make_fix_delta(
        ...     meas_approxDP, delta=1e-8
        ... )
        >>> # FixedSmoothedMaxDivergence distances are (ε, δ) tuples
        >>> meas_fixed_approxDP.map(d_in=1.0)
        (13.3861046488579, 1e-08)

These last two combinators allow you to convert output distances in terms of ρ-zCDP to ε(δ)-approxDP, and then to (ε, δ)-approxDP.
