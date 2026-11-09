"""Sklearn interface to discrete normalized spherical k-means."""

from dataclasses import replace
import math

import numpy as np
from sklearn.base import TransformerMixin

import opendp.prelude as dp
from opendp._internal import _make_measurement, _make_transformation
from opendp.extras.scipy import CSRArray2Descriptor
from opendp.extras.scipy._domains import _member
from opendp.extras.sklearn._domains import SklearnFitDescriptor, sklearn_fit_domain
from opendp.extras.sklearn._estimator import DPTransformer
from opendp.extras.sklearn.cluster._make_private_spherical_kmeans import (
    _check_space,
    _distances,
    _labels,
    _unit_centers,
    make_private_spherical_kmeans,
)
from opendp.mod import Domain, Metric, Transformation


def _check_X(X, n_features):
    if not _member(X, CSRArray2Descriptor(n_features, None, "bool")):
        raise ValueError(
            f"expected canonical boolean CSR input with {n_features} features"
        )
    return X


class SphericalKMeans(TransformerMixin, DPTransformer):
    """Private spherical k-means on canonical boolean CSR arrays.

    :param n_clusters: Number of center directions.
    :param max_iter: Fixed number of adaptively composed private Lloyd updates.
    :param max_features_per_center: Maximum nonzero center coordinates, or None.
    :param max_l1_contribution: Optional positive L1 cap on each unit-normalized contribution.
    :param init: "random" for a public feature partition, or public/already-DP nonnegative directions of shape (n_clusters, n_features). Do not use unaccounted private records.
    :param random_state: Seed for public initialization only, never DP noise.
    :ivar cluster_centers_: Nonnegative int64 CSR center directions of shape (n_clusters, n_features).
    :ivar n_features_in_: Number of input features.
    """

    def __init__(
        self,
        n_clusters: int = 8,
        *,
        max_iter: int = 5,
        max_features_per_center: int | None = None,
        max_l1_contribution: float | None = None,
        init="random",
        random_state: int | None = None,
    ):
        self.n_clusters = n_clusters
        self.max_iter = max_iter
        self.max_features_per_center = max_features_per_center
        self.max_l1_contribution = max_l1_contribution
        self.init = init
        self.random_state = random_state

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        desc = input_domain.cast(SklearnFitDescriptor)
        measurement = make_private_spherical_kmeans(
            desc.X, input_metric, output_measure, d_in, d_out, **self.get_params()
        )
        return _make_measurement(
            input_domain,
            input_metric,
            measurement.output_measure,
            lambda data: measurement(data.X),
            measurement.map,
        )

    def _ingest_release(self, centers):
        # Detach buffers and discard FFI metadata retaining private measurement state.
        self.cluster_centers_ = centers.copy()
        self.n_features_in_ = centers.shape[1]

    def _require_fitted(self):
        if not hasattr(self, "cluster_centers_"):
            raise ValueError("model has not been fitted")

    def predict(self, X):
        self._require_fitted()
        return _labels(
            _check_X(X, self.n_features_in_), _unit_centers(self.cluster_centers_)
        )

    def transform(self, X):
        self._require_fitted()
        return _distances(_check_X(X, self.n_features_in_), self.cluster_centers_)

    def score(self, X, y=None):
        return -math.fsum(np.min(self.transform(X), axis=1))

    def _make_rowwise(self, input_domain, input_metric, output_x_domain, fn):
        desc = input_domain.cast(SklearnFitDescriptor)
        n_features = _check_space(desc.X, input_metric).n_features
        if n_features != self.n_features_in_:
            raise ValueError("input_domain must match the fitted number of features")
        return _make_transformation(
            input_domain,
            input_metric,
            sklearn_fit_domain(output_x_domain, desc.y),
            input_metric,
            lambda data: replace(data, X=fn(_check_X(data.X, n_features))),
            lambda distance: distance,
        )

    def make_transform(
        self, input_domain: Domain, input_metric: Metric
    ) -> Transformation:
        """Map aligned rows to cosine distances from snapshotted released centers."""
        self._require_fitted()
        centers = self.cluster_centers_.copy()
        return self._make_rowwise(
            input_domain,
            input_metric,
            dp.numpy.array2_domain(T=float, num_columns=centers.shape[0]),
            lambda x: _distances(x, centers),
        )

    def make_predict(
        self, input_domain: Domain, input_metric: Metric
    ) -> Transformation:
        """Map aligned rows to nearest-center labels with unit row stability."""
        self._require_fitted()
        unit = _unit_centers(self.cluster_centers_)
        dtype = np.dtype(
            np.int32
            if self.cluster_centers_.dtype == np.int32 and unit.shape[0] <= 2**31
            else np.int64
        )
        return self._make_rowwise(
            input_domain,
            input_metric,
            dp.vector_domain(dp.atom_domain(T=dtype.type)),
            lambda x: _labels(x, unit).astype(dtype),
        )
