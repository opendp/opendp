.. _dp-clustering:

Differentially Private Clustering
=================================

This notebook documents making a differentially private clustering release
with the scikit-learn style clustering API in OpenDP.

--------------

Any functions that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to ``"contrib"``. Please contact
us if you are interested in proof-writing. Thank you!

The clustering implementation uses a recursive private tree release inspired by
*Scalable Differentially Private Clustering via Hierarchically Separated Trees*
(Cohen-Addad et al., KDD 2022, DOI: ``10.1145/3534678.3539409``).

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")
            >>> import numpy as np


We'll work with a simple synthetic dataset containing three blob-like clusters.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> rng = np.random.default_rng(7)
            >>> centers = np.array(
            ...     [
            ...         [-4.0, -4.0],
            ...         [0.0, 5.0],
            ...         [5.5, 0.0],
            ...     ]
            ... )
            >>> X = np.vstack(
            ...     [
            ...         center + rng.normal(scale=0.6, size=(200, 2))
            ...         for center in centers
            ...     ]
            ... )
            >>> lower = X.min(axis=0) - 1.5
            >>> upper = X.max(axis=0) + 1.5


OpenDP provides ``KMeans`` and ``KMedians`` estimators under
``dp.sklearn.cluster``. The API mirrors scikit-learn where possible, while
``scale`` acts as the global, measure-agnostic privacy knob.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> model = dp.sklearn.cluster.KMeans(
            ...     n_features=2,
            ...     n_clusters=3,
            ...     scale=1.0,
            ...     max_depth=7,
            ...     lower=lower,
            ...     upper=upper,
            ... )


The private release occurs when ``fit`` is called.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> model.fit(X)
            <...KMeans object at ...>


Once fitted, the estimator exposes the released cluster centers and can be used
for prediction and scoring.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> model.cluster_centers_.shape
            (3, 2)
            >>> model.predict(X[:5])  # doctest: +ELLIPSIS
            [...]
            >>> model.score(X) <= 0.0
            True


If you want to explicitly distribute privacy budget across levels of the tree,
replace ``max_depth`` with ``level_allocation``. After normalization, each
entry in ``level_allocation`` gives the fraction of the remaining privacy budget
allocated to that level.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> model = dp.sklearn.cluster.KMedians(
            ...     n_features=2,
            ...     n_clusters=3,
            ...     scale=1.0,
            ...     level_allocation=[4.0, 2.0, 1.0, 1.0],
            ...     lower=lower,
            ...     upper=upper,
            ... )
            >>> model.fit(X)
            <...KMedians object at ...>


For lower-level use, see :py:func:`~opendp.extras.sklearn.cluster.make_private_tree_cluster`.
