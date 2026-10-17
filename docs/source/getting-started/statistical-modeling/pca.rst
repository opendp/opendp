.. _dp-pca:

Differentially Private PCA
=========================

Fit a private PCA model
-----------------------

OpenDP's :class:`~opendp.extras.sklearn.decomposition.PCA` follows sklearn's
fitting conventions. Privacy assumptions belong to ``fit``, not the estimator
constructor. Install ``opendp[scikit-learn]`` to use this interface.

.. code:: pycon

    >>> import numpy as np
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib", "idealized-numerics", "honest-but-curious")
    >>> public_size = 100
    >>> X = np.random.default_rng(0).normal(size=(public_size, 4))

Here X stands in for private training data. Its dtype, feature width, and exact
sample count must be public. NumPy domain inference supplies dtype and width,
but intentionally leaves the number of rows unspecified. The current PCA
mechanism needs an exact public sample count, so supply it explicitly:

.. code:: pycon

    >>> schema = dp.domain_of(X, infer=True).descriptor
    >>> X_domain = dp.numpy.array2_domain(
    ...     T=schema.T, num_columns=schema.num_columns, size=public_size,
    ... )
    >>> fit_domain = dp.sklearn.sklearn_fit_domain(X_domain)

For this fixed-size domain, protect **one row replacement**: under symmetric
distance, replacing one row costs two edits (one removal and one addition).
Thus use ``contributions=2``, not ``contributions=1``. An add/remove-only adjacency
at distance one has no distinct neighbors within an exact-size domain.
Use an externally fixed public size, not a private row count or a noisy released
count: this domain requires the exact sample count.

.. code:: pycon

    >>> model = dp.sklearn.decomposition.PCA(n_components=2, row_norm=1.0)
    >>> model.fit(
    ...     X, domain=fit_domain,
    ...     privacy_unit=dp.unit_of(contributions=2),
    ...     privacy_loss=dp.loss_of(epsilon=1),
    ... ) is model
    True
    >>> model.components_.shape, model.singular_values_.shape
    ((2, 4), (2,))
    >>> public_rows = np.zeros((3, 4))
    >>> model.transform(public_rows).shape
    (3, 2)

``row_norm`` is a public L2 clipping bound applied internally during fitting.
It may be omitted when the input domain already supplies an L2 bound.
``n_components`` also accepts a variance fraction, ``'mle'``, or None;
``whiten=True`` scales projections using the released variances.
This implementation currently supports pure epsilon-DP.

Fit within a Context
--------------------

A Context supplies and accounts for the privacy budget. Its release fits the
original estimator in place, just like standalone ``fit``:

.. code:: pycon

    >>> context = dp.Context.compositor(
    ...     dp.sklearn.SklearnFit(X), domain=fit_domain,
    ...     privacy_unit=dp.unit_of(contributions=2),
    ...     privacy_loss=dp.loss_of(epsilon=1), split_evenly_over=1,
    ... )
    >>> context.query().sklearn(model).release() is model
    True
    >>> model.components_.shape
    (2, 4)
    >>> context.current_privacy_loss()
    [1.0]

These are alternative fitting routes. Running both examples on the same private
data spends their composed privacy loss, here epsilon 2. The Context does not
track the earlier standalone fit. Refitting replaces learned state; it does not
reuse a previous fit's privacy budget.

Advanced composition
--------------------

For lower-level composition, ``make`` constructs an OpenDP
:class:`~opendp.mod.Measurement` without executing a fit or updating the estimator:

.. code:: pycon

    >>> measurement = model.make(
    ...     fit_domain, dp.symmetric_distance(), dp.max_divergence(),
    ...     d_in=2, d_out=1.0,
    ... )
    >>> measurement.check(2, 1.0)
    True

This measurement consumes ``SklearnFit`` and releases
:class:`~opendp.extras.sklearn.decomposition.PCARelease`. See the
:mod:`~opendp.combinators` API for composition and
:func:`~opendp.extras.sklearn.decomposition.make_private_pca` for the natural
X-only framework mechanism.

For preprocessing within a subsequent private query, ``model.make_transform``
constructs a stable transformation from released parameters, retaining any y
in ``SklearnFit``. Ordinary ``transform`` outputs on private training rows are
**not DP releases** and must not be handed to a nonprivate downstream estimator
as though they were. Ordinary sklearn pipelines do not account for end-to-end
privacy.
