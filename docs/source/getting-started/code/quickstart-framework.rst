:orphan:

.. code:: pycon

    # init
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

    # /init

    # demo
    >>> domain = dp.atom_domain(T=float, nan=False)
    >>> metric = dp.absolute_distance(T=float)
    >>> laplace_mechanism = (domain, metric) >> dp.m.then_laplace(scale=1.0)
    >>> dp_value = laplace_mechanism(123.0)

    # /demo
