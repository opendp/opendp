:orphan:

.. code:: pycon

    # init
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

    # /init

    # demo
    >>> context = dp.Context.compositor(
    ...     data=123,
    ...     privacy_unit=dp.unit_of(absolute=1.0),
    ...     privacy_loss=dp.loss_of(epsilon=1.0),
    ... )
    >>> dp_value = context.query(epsilon=1.0).laplace().release()

    # /demo
