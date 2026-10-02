Linear Regression
==========================

Theil-Sen regression is documented with examples in the API Reference
(:py:mod:`opendp.extras.sklearn.linear_model`). Private feature and target data
are represented by :class:`opendp.extras.sklearn.Aligned`, which preserves their
row correspondence while a whole aligned row remains one adjacency event::

    import opendp.prelude as dp

    training = dp.sklearn.Aligned(X=X[:, :1], y=y)
    domain = dp.sklearn.aligned_domain(
        X=dp.numpy.array2_domain(
            num_columns=1, size=len(training.X), T=float, nan=False
        ),
        y=dp.vector_domain(dp.atom_domain(T=float, nan=False), size=len(training.y)),
    )
    context = dp.Context.compositor(
        data=training,
        domain=domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    model = dp.sklearn.linear_model.TheilSenRegressor(
        x_bounds=((-3.0, 3.0),), y_bounds=(-10.0, 10.0)
    )
    model.fit(context.query())

The underlying algorithm is also used as
`an example of a plug-in <../../api/user-guide/plugins/theil-sen-regression.html>`_.
