.. _dp-pca:

Differentially Private PCA
==========================

This notebook documents making a differentially private PCA release.

--------------

Any functions that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> dp.enable_features(
            ...     "contrib", "idealized-numerics", "honest-but-curious"
            ... )

            >>> import numpy as np

            >>> def sample_microdata(
            ...     *, num_columns=None, num_rows=None, cov=None
            ... ):
            ...     cov = cov or sample_covariance(num_columns)
            ...     microdata = np.random.multivariate_normal(
            ...         np.zeros(cov.shape[0]),
            ...         cov,
            ...         size=num_rows or 100_000,
            ...     )
            ...     microdata -= microdata.mean(axis=0)
            ...     return microdata
            ...

            >>> def sample_covariance(num_features):
            ...     A = np.random.uniform(
            ...         0, num_features, size=(num_features, num_features)
            ...     )
            ...     return A.T @ A
            ...


In this notebook we’ll be working with an example dataset generated from
a random covariance matrix.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> num_columns = 4
            >>> num_rows = 10_000
            >>> example_dataset = sample_microdata(
            ...     num_columns=num_columns, num_rows=num_rows
            ... )


Recommended API
---------------

The Context-based workflow is the recommended public API and has an API
akin to scikit-learn:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> domain = dp.numpy.array2_domain(
            ...     size=num_rows,
            ...     num_columns=num_columns,
            ...     nan=False,
            ...     T=float,
            ... )
            >>> context = dp.Context.compositor(
            ...     data=example_dataset,
            ...     domain=domain,
            ...     privacy_unit=dp.unit_of(contributions=1),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=1,
            ... )
            >>> model = dp.sklearn.decomposition.PCA(
            ...     n_components=2,
            ... )


A private release occurs when you fit the model to the data.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> model.fit(
            ...     context.query().np_clip(
            ...         p=2,
            ...         norm=1.0,
            ...     )
            ... ) is model
            True

PCA requires an L2 norm bound. ``np_clip`` projects each row onto an L2
ball as part of the query preprocessing chain; the estimator itself contains
only PCA parameters.

The fitted model can then be introspected just like Scikit-Learn’s
non-private PCA:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> print(model.singular_values_)
            [... ... ... ...]
            >>> print(model.components_)
            [[... ... ... ...]
             [... ... ... ...]
             [... ... ... ...]
             [... ... ... ...]]

Instead of fitting the model, you could instead retrieve the measurement
used to make the release, just like other OpenDP APIs. This time, we’ll
also only fit 2 components. Because of this, more budget will be
allocated to estimating each eigenvector internally.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> bounded_domain = dp.numpy.array2_domain(
            ...     norm=1.0,
            ...     p=2,
            ...     size=num_rows,
            ...     num_columns=num_columns,
            ...     nan=False,
            ...     T=float,
            ... )
            >>> model = dp.sklearn.decomposition.PCA(n_components=2)
            >>> measurement = model.make(
            ...     bounded_domain,
            ...     dp.symmetric_distance(),
            ...     dp.max_divergence(),
            ...     1,
            ...     1.0,
            ... )


The measurement returns a :class:`~PCARelease` and does not fit or mutate
``model``. Fitting through a query consumes the release and stores sklearn fitted
attributes on ``model``:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> release = measurement(example_dataset)
            >>> release.components.shape
            (2, 4)

``make(...)`` makes it possible to use the calibrated release
with other OpenDP combinators. Ordinary inference methods such as
``transform`` and ``inverse_transform`` operate on caller-held public data.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> print(release.singular_values)
            [... ...]
            >>> print(release.components)
            [[... ... ... ...]
             [... ... ... ...]]


Aligned sklearn workflows
-------------------------

``Aligned`` is a sklearn orchestration carrier, not PCA's framework input
representation. It keeps private targets and metadata under one shared row
adjacency. The sklearn fitting bridge projects X for PCA and leaves the original
aligned query available for later steps:

.. code-block:: python

    aligned_domain = dp.sklearn.aligned_domain(
        X=domain,
        y=dp.vector_domain(dp.atom_domain(T=int), size=num_rows),
    )
    aligned_context = dp.Context.compositor(
        data=dp.sklearn.Aligned(X=example_dataset, y=[0] * num_rows),
        domain=aligned_domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    clip = dp.numpy.make_np_clip(domain, dp.symmetric_distance(), norm=1.0, p=2)
    query = aligned_context.query().lift_X(clip)
    model = dp.sklearn.decomposition.PCA(n_components=2)
    model.fit(query)

    # Keep transformed records private for subsequent pipeline measurements.
    transform = model.make_transform(clip.output_domain)
    next_query = query.lift_X(transform)

``next_query`` still carries y (and any sample weights or groups), with only X
replaced by its released PCA projection. Resolving this preprocessing chain as a
transformation is not a private release of the transformed training rows.
``lift_X`` accepts reviewed rowwise transformations, not arbitrary operations
whose X-only stability map happens to be the identity.

Please reach out on Slack if you need to a more tailored analysis: there
are lower-level APIs for estimating *only* the eigenvalues or
eigenvectors, or to avoid mean estimation when your data is already
bounded.
