Amplification
-------------

If your dataset is a simple sample from a larger population,
you can make the privacy relation more permissive by wrapping your measurement with a privacy amplification combinator:
:func:`~opendp.combinators.make_population_amplification`.

.. note::

    Sampling: The input dataset must be a simple random sample from the stated
    population, and sample membership must remain confidential.
    Terminal amplification: The amplified measurement must encompass all processing
    and releases involving the sample, and must be its only analysis.

    .. code:: pycon


        >>> dp.enable_features("honest-but-curious")


In order to demonstrate this API, we'll first create a measurement with a sized input domain.
The resulting measurement expects the size of the input dataset to be 10.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> atom_domain = dp.atom_domain(bounds=(0.0, 10.0), nan=False)
        >>> input_space = (
        ...     dp.vector_domain(atom_domain, size=10),
        ...     dp.symmetric_distance(),
        ... )
        >>> meas = (
        ...     input_space
        ...     >> dp.t.then_mean()
        ...     >> dp.m.then_laplace(scale=0.5)
        ... )
        >>> print(
        ...     "standard mean:", meas([1.0] * 10)
        ... )  # -> 1.03 # doctest: +ELLIPSIS
        standard mean: ...

We can now use the amplification combinator to construct an amplified measurement.
The function on the amplified measurement is identical to the standard measurement.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

      >>> amplified = dp.c.make_population_amplification(
      ...     meas, population_size=100
      ... )
      >>> print(
      ...     "amplified mean:", amplified([1.0] * 10)
      ... )  # -> .97 # doctest: +ELLIPSIS
      amplified mean: ...

The privacy relation on the amplified measurement takes into account that the input dataset of size 10
is a simple sample of individuals from a theoretical larger dataset that captures the entire population, with 100 rows.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> # Where we once had a privacy utilization of ~2 epsilon...
        >>> assert meas.check(2, 2.0 + 1e-6)
        >>> # ...we now have a privacy utilization of ~.4941 epsilon.
        >>> assert amplified.check(2, 0.4941)

The efficacy of this combinator improves as n gets larger.

All processing of the sample, including transformations, must be part of the
measurement passed to the amplifier rather than chained before the amplified measurement.
When releasing multiple results from the same sample, compose the measurements
before amplification:

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> jointly_amplified = dp.c.make_population_amplification(
        ...     dp.c.make_composition([meas, meas]), population_size=100
        ... )

Composing independently amplified measurements on that same sample instead
violates the terminal amplification requirement:

.. code:: pycon

    >>> # Invalid: amplifies each release separately on the same sample.
    >>> dp.c.make_composition(
    ...     [amplified, amplified]
    ... )  # doctest: +SKIP
