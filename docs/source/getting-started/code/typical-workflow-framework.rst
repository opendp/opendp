:orphan:

.. code:: pycon

    # unit-of-privacy
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

    >>> # neighboring data set distance is at most d_in...
    >>> d_in = 1

    >>> # ...in terms of additions/removals
    >>> input_metric = dp.symmetric_distance()

    >>> input_domain = dp.vector_domain(dp.atom_domain(T=int))

    # /unit-of-privacy


    # privacy-loss
    >>> # output distributions have distance at most d_out (ε)...
    >>> d_out = 1.0

    >>> # ...in terms of pure-DP
    >>> privacy_measure = dp.max_divergence()

    # /privacy-loss


    # mediate
    >>> from random import randint
    >>> data = [randint(0, 100) for _ in range(100)]

    >>> o_ac = dp.c.make_fully_adaptive_composition(
    ...     input_domain=input_domain,
    ...     input_metric=input_metric,
    ...     output_measure=privacy_measure,
    ... )

    >>> m_ac = dp.c.make_privacy_filter(
    ...     odometer=o_ac,
    ...     d_in=d_in,
    ...     d_out=d_out,
    ... )

    >>> # Call measurement with data to create a queryable:
    >>> queryable = m_ac(data)

    # /mediate


    # count
    >>> def make_dp_count(scale):
    ...     count = dp.t.make_count(input_domain, input_metric)
    ...     return count >> dp.m.then_laplace(scale)
    ...

    >>> scale = dp.binary_search_param(
    ...     make_dp_count,
    ...     d_in,
    ...     d_out / 3,
    ... )
    >>> scale
    3.0000000000000004

    >>> accuracy = dp.discrete_laplacian_scale_to_accuracy(
    ...     scale=scale, alpha=0.05
    ... )
    >>> accuracy
    9.445721638273584

    >>> # (with 100(1-alpha) = 95% confidence, the estimated value will differ
    >>> #    from the true value by no more than ~9)

    >>> sum_measurement = make_dp_count(scale)

    >>> dp_count = queryable(sum_measurement)
    >>> confidence_interval = (
    ...     dp_count - accuracy,
    ...     dp_count + accuracy,
    ... )

    # /count


    # public-info
    >>> bounds = (0, 100)

    # /public-info

    # sum
    >>> sum_transformation = (
    ...     (input_domain, input_metric)
    ...     >> dp.t.then_clamp(bounds)
    ...     >> dp.t.then_sum()
    ... )

    >>> sum_measurement = dp.binary_search_chain(
    ...     lambda scale: sum_transformation
    ...     >> dp.m.then_laplace(scale),
    ...     d_in,
    ...     d_out / 3,
    ... )

    >>> dp_sum = queryable(sum_measurement)

    # /sum
