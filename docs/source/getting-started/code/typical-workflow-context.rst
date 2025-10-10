:orphan:

.. code:: pycon

    # unit-of-privacy
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

    >>> privacy_unit = dp.unit_of(contributions=1)
    >>> privacy_unit
    (SymmetricDistance(), 1)

    # /unit-of-privacy


    # privacy-loss
    >>> privacy_loss = dp.loss_of(epsilon=1.0)
    >>> privacy_loss
    (MaxDivergence, 1.0)

    # /privacy-loss

    # mediate
    >>> from random import randint
    >>> data = [randint(0, 100) for _ in range(100)]

    >>> context = dp.Context.compositor(
    ...     data=data,
    ...     privacy_unit=privacy_unit,
    ...     privacy_loss=privacy_loss,
    ... )

    # /mediate


    # count
    >>> count_query = context.query(epsilon=1 / 3).count().laplace()

    >>> # before releasing, check the utility
    >>> scale = count_query.param()
    >>> scale
    3.0000000000000004

    >>> accuracy = dp.discrete_laplacian_scale_to_accuracy(
    ...     scale=scale, alpha=0.05
    ... )
    >>> accuracy
    9.445721638273584

    >>> # (with 100(1-alpha) = 95% confidence, the estimated value will differ
    >>> #    from the true value by no more than ~9)

    >>> dp_count = count_query.release()
    >>> confidence_interval = (
    ...     dp_count - accuracy,
    ...     dp_count + accuracy,
    ... )

    # /count

    # public-info
    >>> bounds = (0, 100)

    # /public-info

    # sum
    >>> sum_query = (
    ...     context.query(epsilon=1 / 3)
    ...     .clamp(bounds)
    ...     .sum()
    ...     .laplace()
    ... )

    >>> dp_sum = sum_query.release()

    # /sum
