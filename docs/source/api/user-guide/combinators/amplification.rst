Amplification
-------------

If your dataset is a simple sample from a larger population, you can make the
privacy relation more permissive by accounting for privacy amplification by
sampling without replacement.

:func:`~opendp.combinators.amplify_without_replacement` takes the privacy map
of an analysis on the sample and returns a privacy map for the larger
population. Both datasets have a fixed size, and distances count substitutions
(``ChangeOneDistance``).

.. note::

    The privacy guarantees are only valid if the input dataset is a simple
    random sample from a population with `population_size` records. Sampling
    must be independent of the data, and sample membership must remain
    confidential. The input privacy map must account for all processing and
    releases involving the sample.

    .. code:: pycon

        >>> dp.enable_features("honest-but-curious")

In order to demonstrate this API, consider an analysis of a sample of 10
records from a population of 100. The privacy map below spends one epsilon
for each substitution in the sample.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> sample_map = lambda d: float(d)
        >>> population_map = dp.c.amplify_without_replacement(
        ...     dp.change_one_distance(),
        ...     dp.max_divergence(),
        ...     sample_map,
        ...     population_size=100,
        ...     sample_size=10,
        ... )
        >>> assert sample_map(1) == 1.0
        >>> assert 0.158 < population_map(1) < 0.159
        >>> assert 0.316 < population_map(2) < 0.318

The privacy relation on the population takes into account that the input
dataset of size 10 is a simple sample of individuals from a larger dataset
with 100 rows.
