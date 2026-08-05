'''
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.cluster``.
'''

from __future__ import annotations

from opendp.extras.sklearn.cluster._spherical_lloyd_binary import (
    SphericalKMeans,
    SphericalKMeansConfig,
    sparse_binary_domain,
    make_cluster_feature_sums,
    then_cluster_feature_sums,
    make_private_spherical_kmeans,
    then_private_spherical_kmeans,
    nearest_center_labels,
)

__all__ = [
    "SphericalKMeans",
    "SphericalKMeansConfig",
    "sparse_binary_domain",
    "make_cluster_feature_sums",
    "then_cluster_feature_sums",
    "make_private_spherical_kmeans",
    "then_private_spherical_kmeans",
    "nearest_center_labels",
]
