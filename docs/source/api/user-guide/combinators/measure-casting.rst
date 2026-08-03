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
   * - ``ApproxDP``
     - ``MultiDP``
     - :func:`~opendp.combinators.make_approxDP_to_multiDP`
   * - ``PureDP``
     - ``zCDP``
     - :func:`~opendp.combinators.make_pureDP_to_zCDP`
   * - ``zCDP``
     - ``MultiDP``
     - :func:`~opendp.combinators.make_zCDP_to_multiDP`
   * - ``MultiDP``
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
:func:`~opendp.combinators.make_zCDP_to_multiDP` casts an output measure from ``zCDP`` to ``MultiDP``.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> meas_zCDP = input_space >> dp.m.then_gaussian(scale=0.5)
        >>> # convert the output measure to `MultiDP`
        >>> meas_multiDP = dp.c.make_zCDP_to_multiDP(meas_zCDP)
        >>> # MultiDP distances are PrivacyGuarantees
        >>> guarantee = meas_multiDP.map(d_in=1.0)
        >>> guarantee.epsilon(delta=1e-6)
        11.688596249354896

A ``PrivacyGuarantee`` contains multiple simultaneously valid privacy
representations for the same mechanism and neighboring relation.

:func:`~opendp.combinators.make_fix_delta` changes the output measure from ``MultiDP`` to ``ApproxDP``.
It fixes delta when querying the guarantee so that the resulting measurement can be composed with other ``ApproxDP`` measurements.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> # convert the output measure to `ApproxDP`
        >>> meas_fixed_approxDP = dp.c.make_fix_delta(
        ...     meas_multiDP, delta=1e-8
        ... )
        >>> # `ApproxDP` distances are (ε, δ) tuples
        >>> meas_fixed_approxDP.map(d_in=1.0)
        (13.3861046488579, 1e-08)

These last two combinators allow you to retain a zCDP representation in a
``PrivacyGuarantee`` and later query an (ε, δ)-DP bound.
