.. _measure-casting:

Measure Casting
---------------
These combinators are used to cast the output measure of a Measurement.

.. list-table::
   :header-rows: 1

   * - Input Measure
     - Output Measure
     - Constructor
   * - ``PureDP``
     - ``ApproxDP``
     - :func:`~opendp.combinators.make_approximate`
   * - ``zCDP``
     - ``ApproxZCDP``
     - :func:`~opendp.combinators.make_approximate`
   * - ``PureDP``
     - ``PrivacyCurveDP``
     - :func:`~opendp.combinators.make_approxDP_to_curveDP`
   * - ``PureDP``
     - ``zCDP``
     - :func:`~opendp.combinators.make_pureDP_to_zCDP`
   * - ``zCDP``
     - ``PrivacyCurveDP``
     - :func:`~opendp.combinators.make_zCDP_to_curveDP`
   * - ``PrivacyCurveDP``
     - ``ApproxDP``
     - :func:`~opendp.combinators.make_fix_delta`

:func:`~opendp.combinators.make_approximate` is useful when you want to compose pure-DP measurements with approximate-DP measurements,
or zCDP measurements with approx-zCDP measurements.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> input_space = dp.atom_domain(
        ...     T=float, nan=False
        ... ), dp.absolute_distance(T=float)
        >>> meas_pureDP = input_space >> dp.m.then_laplace(scale=10.0)
        >>> # convert the output measure to `ApproxDP`
        >>> meas_fixed_approxDP = dp.c.make_approximate(meas_pureDP)
        >>> # `ApproxDP` distances are (ε, δ) tuples
        >>> meas_fixed_approxDP.map(d_in=1.0)
        (0.1, 0.0)

The combinator can also be used on measurements with a ``zCDP`` privacy measure.

:func:`~opendp.combinators.make_pureDP_to_zCDP` is used for casting an output measure from ``PureDP`` to ``zCDP``.
:func:`~opendp.combinators.make_zCDP_to_curveDP` is used for casting an output measure from ``zCDP`` to ``PrivacyCurveDP``.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> meas_zCDP = input_space >> dp.m.then_gaussian(scale=0.5)
        >>> # convert the output measure to `PrivacyCurveDP`
        >>> meas_approxDP = dp.c.make_zCDP_to_curveDP(meas_zCDP)
        >>> # `PrivacyCurveDP` distances are privacy profiles (ε(δ) curves)
        >>> profile = meas_approxDP.map(d_in=1.0)
        >>> profile.epsilon(delta=1e-6)
        11.688596249354896

:func:`~opendp.combinators.make_fix_delta` changes the output measure from ``PrivacyCurveDP`` to ``ApproxDP``.
It fixes the delta parameter in the curve, so that the resulting measurement can be composed with other ``ApproxDP`` measurements.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> # convert the output measure to `ApproxDP`
        >>> meas_fixed_approxDP = dp.c.make_fix_delta(
        ...     meas_approxDP, delta=1e-8
        ... )
        >>> # `ApproxDP` distances are (ε, δ) tuples
        >>> meas_fixed_approxDP.map(d_in=1.0)
        (13.3861046488579, 1e-08)

These last two combinators allow you to convert output distances in terms of ρ-zCDP to ε(δ)-approxDP, and then to (ε, δ)-approxDP.
