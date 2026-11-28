.. _dp-pca:

Differentially Private PCA
==========================

OpenDP provides a scikit-learn-compatible PCA estimator. Fitting privately
releases sufficient statistics, then computes the principal axes and
variances by postprocessing. A public L2 row bound is required.

The pure-DP mechanism uses Wishart-difference noise. The zCDP mechanism
uses Gaussian noise. Both estimate the feature mean by default; use
``center="origin"`` to center around the public domain origin instead.

.. code:: pycon

    >>> import pytest
    >>> _ = pytest.importorskip("sklearn")
    >>> import numpy as np
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib", "idealized-numerics")
    >>> X = np.tile([[0.5, 0.2], [-0.5, -0.2]], (500, 1))
    >>> domain = dp.sklearn.sklearn_fit_domain(
    ...     dp.numpy.array2_domain(T=float, size=1000, num_columns=2)
    ... )
    >>> model = dp.sklearn.decomposition.PCA(n_components=1, clip_norm=1.0)
    >>> _ = model.fit(
    ...     X, domain=domain,
    ...     privacy_unit=dp.unit_of(contributions=2),
    ...     privacy_loss=dp.loss_of(epsilon=1.0),
    ... )
    >>> model.components_.shape
    (1, 2)
    >>> model.transform(np.zeros((3, 2))).shape
    (3, 1)

The privacy unit protects one row replacement when the dataset size is
public. When size is private, omit the size from the domain and choose
the contribution bound for your privacy unit. The released sample count
is then estimated privately.

See :class:`~opendp.extras.sklearn.decomposition.PCA` for the supported
parameters and fitted attributes. The estimator's ``make_measurement``
method supports composition with other OpenDP measurements.
