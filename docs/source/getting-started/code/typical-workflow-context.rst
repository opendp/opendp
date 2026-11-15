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


    # public-info
    >>> bounds = (0.0, 100.0)

    # /public-info


    # mediate
    >>> from random import randint
    >>> data = [float(randint(0, 100)) for _ in range(100)]

    >>> context = dp.Context.compositor(
    ...     data=data,
    ...     privacy_unit=privacy_unit,
    ...     privacy_loss=privacy_loss,
    ...     split_evenly_over=3,
    ... )

    # /mediate

