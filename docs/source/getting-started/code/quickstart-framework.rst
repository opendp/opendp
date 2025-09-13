:orphan:

.. code:: pycon

    # init
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

    # /init

    # demo
    >>> domain = dp.atom_domain(T=int)
    >>> metric = dp.absolute_distance(T=int)
    >>> space = (domain, metric)

    >>> laplace_mechanism = space >> dp.m.then_laplace(scale=1.0)
    >>> dp_value = laplace_mechanism(123)

    # /demo
