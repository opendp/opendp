"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.cluster``.
"""

from __future__ import annotations

from opendp._lib import import_optional_dependency
from opendp.extras.sklearn.cluster._make_private_spherical_kmeans import (
    make_private_spherical_kmeans,
    then_private_spherical_kmeans,
)

if import_optional_dependency("sklearn", raise_error=False) is not None:
    from opendp.extras.sklearn.cluster._spherical_kmeans import SphericalKMeans

__all__ = [
    "SphericalKMeans",
    "make_private_spherical_kmeans",
    "then_private_spherical_kmeans",
]
