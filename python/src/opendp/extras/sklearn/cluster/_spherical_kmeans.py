"""Sklearn interface to discrete normalized spherical k-means."""

import math

import numpy as np
from sklearn.base import (
    BaseEstimator,
    ClassNamePrefixFeaturesOutMixin,
    ClusterMixin,
    TransformerMixin,
)
from sklearn.utils.validation import check_is_fitted

import opendp.prelude as dp
from opendp._internal import _make_measurement
from opendp.extras.scipy import CSRArray2Descriptor
from opendp.extras.scipy._domains import _member
from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from opendp.extras.sklearn.cluster._make_private_spherical_kmeans import (
    _check_space,
    _distances,
    _labels,
    _unit_centers,
    make_private_spherical_kmeans,
)


def _check_X(X, n_features):
    if not _member(X, CSRArray2Descriptor(n_features, None, "bool")):
        raise ValueError(
            f"expected canonical boolean CSR input with {n_features} features"
        )
    return X


class SphericalKMeans(
    MeasurementMixin,
    TransformationMixin,
    ClassNamePrefixFeaturesOutMixin,
    TransformerMixin,
    ClusterMixin,
    BaseEstimator,
):
    """Private spherical k-means on canonical boolean CSR arrays.

    :param n_clusters: Number of center directions.
    :param max_iter: Fixed number of adaptively composed private Lloyd updates.
    :param max_features_per_center: Maximum nonzero center coordinates, or None.
    :param max_l1_contribution: Optional positive L1 cap on each unit-normalized contribution.
    :param init: "random" for a public feature partition, or public/already-DP nonnegative directions of shape (n_clusters, n_features). Do not use unaccounted private records.
    :param random_state: Seed for public initialization only, never DP noise.
    :param T: Integer noise dtype, ``i64`` by default (``i32`` is also supported).
    :ivar cluster_centers_: Nonnegative CSR float64 center directions of shape (n_clusters, n_features). Nonzero directions have unit L2 norm; empty centers remain zero.
    :ivar n_features_in_: Number of input features.

    :example:

    >>> import scipy.sparse as sp
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")
    >>> X = sp.csr_array([[1, 0, 0], [1, 1, 0], [0, 0, 1]], dtype=bool)
    >>> model = dp.sklearn.cluster.SphericalKMeans(n_clusters=2, T=dp.i32)
    >>> _ = model.fit(
    ...     X,
    ...     privacy_unit=dp.unit_of(contributions=1),
    ...     privacy_loss=dp.loss_of(epsilon=1.0),
    ... )
    >>> model.cluster_centers_.shape
    (2, 3)
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
        T=dp.i64,
    ):
        self.n_clusters = n_clusters
        self.max_iter = max_iter
        self.max_features_per_center = max_features_per_center
        self.max_l1_contribution = max_l1_contribution
        self.init = init
        self.random_state = random_state
        self.T = T

    @property
    def _n_features_out(self):
        return self.cluster_centers_.shape[0]

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
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
        # Normalize and detach from the integer measurement release.
        self.cluster_centers_ = _unit_centers(centers)
        self.n_features_in_ = centers.shape[1]

    def predict(self, X):
        check_is_fitted(self, ["cluster_centers_"])
        return _labels(_check_X(X, self.n_features_in_), self.cluster_centers_)

    def transform(self, X):
        check_is_fitted(self, ["cluster_centers_"])
        return _distances(_check_X(X, self.n_features_in_), self.cluster_centers_)

    def score(self, X, y=None):
        """Return the negative sum of minimum cosine distances on public inputs."""
        return -math.fsum(np.min(self.transform(X), axis=1))

    def rowwise_output_domain(self, feature_domain, *, method):
        """Describe row-independent sparse inference on a fixed feature domain."""
        check_is_fitted(self, ["cluster_centers_"])
        desc = _check_space(feature_domain, dp.symmetric_distance())
        if desc.n_features != self.n_features_in_:
            raise ValueError("input_domain must match the fitted number of features")

        if method == "transform":
            return dp.numpy.array2_domain(
                T=dp.f64,
                num_columns=self.cluster_centers_.shape[0],
                size=desc.size,
                nan=False,
            )
        if method == "predict":
            return dp.numpy.array2_domain(
                T=dp.i64,
                num_columns=1,
                size=desc.size,
            )
        raise ValueError(f"Method {method!r} does not support rowwise inference")
