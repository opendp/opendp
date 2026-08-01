Linear Regression
==========================

Theil-Sen regression is documented with examples in the API Reference
(:py:mod:`opendp.extras.sklearn.linear_model`). Private training data is
represented as paired ``[x, y]`` rows and fitted through an OpenDP context::

    import numpy as np
    import opendp.prelude as dp

    training = np.column_stack([X[:, 0], y])
    context = dp.Context.compositor(
        data=training,
        domain=dp.numpy.array2_domain(num_columns=2, size=len(training), T=float),
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