"""Reference: minimal `opendp/extras/sklearn/cluster/__init__.py` for a
spherical-Lloyd-FIRST PR (i.e. committed off `main`, which has no cluster package
and no HST/PE code yet).

Copy the body below into
`python/src/opendp/extras/sklearn/cluster/__init__.py` in your standalone branch.
When the HST and PE PRs land later, they append their own imports/estimators here.

The exploration branch keeps a larger __init__.py (with the HST KMeans/KMedians and
guarded sibling imports) so its existing tests keep running; that file is NOT what a
spherical-first PR should ship.
"""

_MINIMAL_INIT = '''\
"""
This module requires extra installs: ``pip install \'opendp[scikit-learn]\'``

For convenience, all the members of this module are also available from
:py:mod:`opendp.prelude`.  We suggest importing under the conventional name ``dp``,
after which the members are accessible at ``dp.sklearn.cluster``.
"""

from __future__ import annotations

from opendp.extras.sklearn.cluster._spherical_lloyd_binary import (
    SphericalKMeans,
    SphericalLloydMeans,
    SphericalLloydConfig,
    SphericalLloydResult,
    make_sparse_binary_domain,
    make_cluster_feature_sums,
    then_cluster_feature_sums,
    make_private_cluster_feature_sums,
    then_private_cluster_feature_sums,
    make_private_spherical_lloyd_binary,
    then_private_spherical_lloyd_binary,
    nearest_center_labels,
)

__all__ = [
    "SphericalKMeans",
    "SphericalLloydMeans",
    "SphericalLloydConfig",
    "SphericalLloydResult",
    "make_sparse_binary_domain",
    "make_cluster_feature_sums",
    "then_cluster_feature_sums",
    "make_private_cluster_feature_sums",
    "then_private_cluster_feature_sums",
    "make_private_spherical_lloyd_binary",
    "then_private_spherical_lloyd_binary",
    "nearest_center_labels",
]
'''

if __name__ == "__main__":
    print(_MINIMAL_INIT)
