Differentially Private PCA
==========================

This notebook documents making a differentially private PCA release.

--------------

Any constructors that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib", "floating-point", "honest-but-curious")

            >>> import numpy as np
            
            >>> def sample_microdata(*, num_columns=None, num_rows=None, cov=None):
            ...     cov = cov or sample_covariance(num_columns)
            ...     microdata = np.random.multivariate_normal(
            ...         np.zeros(cov.shape[0]), cov, size=num_rows or 100_000
            ...     )
            ...     microdata -= microdata.mean(axis=0)
            ...     return microdata
            
            >>> def sample_covariance(num_features):
            ...     A = np.random.uniform(0, num_features, size=(num_features, num_features))
            ...     return A.T @ A
            

In this notebook we’ll be working with an example dataset generated from
a random covariance matrix.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> num_columns = 4
            >>> num_rows = 10_000
            >>> example_dataset = sample_microdata(num_columns=num_columns, num_rows=num_rows)
            

Releasing a DP PCA model with the OpenDP Library is easy because it
provides an API similar to scikit-learn:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> model = dp.x.sklearn.PCA(
            ...     epsilon=1.,
            ...     row_norm=1.,
            ...     n_samples=num_rows,
            ...     n_features=4,
            ... )
            

A private release occurs when you fit the model to the data.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> model.fit(example_dataset)
            PCA(epsilon=1.0, n_components=4, n_features=4, n_samples=10000, row_norm=1.0)

The fitted model can then be introspected just like Scikit-Learn’s
non-private PCA:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

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

        .. code:: python

            >>> model = dp.x.sklearn.PCA(
            ...     epsilon=1.,
            ...     row_norm=1.,
            ...     n_samples=num_rows,
            ...     n_features=4,
            ...     n_components=2 # only estimate 2 of 4 components this time
            ... )
            >>> meas = model.measurement()
            

The measurement fits ``model`` and then returns ``model``:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> meas(example_dataset)
            PCA(epsilon=1.0, n_components=2, n_features=4, n_samples=10000, row_norm=1.0)
            
``.measurement()`` makes it more convenient to use the Scikit-Learn API
with other combinators, like compositors.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print(model.singular_values_)
            [... ...]
            >>> print(model.components_)
            [[... ... ... ...]
             [... ... ... ...]]


Please reach out on Slack if you need to a more tailored analysis: there
are lower-level APIs for estimating *only* the eigenvalues or
eigenvectors, or to avoid mean estimation when your data is already
bounded.
