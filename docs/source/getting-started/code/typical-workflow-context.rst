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


