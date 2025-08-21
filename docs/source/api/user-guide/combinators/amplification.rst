Amplification
-------------

If your dataset is a simple sample from a larger population,
you can make the privacy relation more permissive by wrapping your measurement with a privacy amplification combinator:
:func:`opendp.combinators.make_population_amplification`.

.. note::

    The amplifier requires a looser trust model, as the population size can be set arbitrarily.

    .. code:: python


        >>> dp.enable_features("honest-but-curious")


In order to demonstrate this API, we'll first create a measurement with a sized input domain.
The resulting measurement expects the size of the input dataset to be 10.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> atom_domain = dp.atom_domain(bounds=(0., 10.), nan=False)
        >>> input_space = dp.vector_domain(atom_domain, size=10), dp.symmetric_distance()
        >>> meas = input_space >> dp.t.then_mean() >> dp.m.then_laplace(scale=0.5)
        >>> print("standard mean:", meas([1.] * 10)) # -> 1.03 # doctest: +ELLIPSIS
        standard mean: ...

We can now use the amplification combinator to construct an amplified measurement.
The function on the amplified measurement is identical to the standard measurement.

.. tab-set::

  .. tab-item:: Python

    .. code:: python
      
        >>> amplified = dp.c.make_population_amplification(meas, population_size=100)
        >>> print("amplified mean:", amplified([1.] * 10)) # -> .97 # doctest: +ELLIPSIS
        amplified mean: ...

The privacy relation on the amplified measurement takes into account that the input dataset of size 10
is a simple sample of individuals from a theoretical larger dataset that captures the entire population, with 100 rows.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> # Where we once had a privacy utilization of ~2 epsilon...
        >>> assert meas.check(2, 2. + 1e-6)
        ...
        >>> # ...we now have a privacy utilization of ~.4941 epsilon.
        >>> assert amplified.check(2, .4941)

The efficacy of this combinator improves as n gets larger.

