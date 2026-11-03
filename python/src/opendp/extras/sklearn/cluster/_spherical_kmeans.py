"""Sklearn interface to discrete normalized spherical k-means."""

from dataclasses import replace
import math

from sklearn.base import TransformerMixin

import opendp.prelude as dp
from opendp._internal import _make_measurement, _make_transformation
from opendp.extras.sklearn._domains import SklearnFitDescriptor, sklearn_fit_domain
from opendp.extras.sklearn._estimator import DPTransformer
from opendp.extras.sklearn.cluster._make_private_spherical_kmeans import (
    _check_space,
    _distances,
    _labels,
    _np,
    _unit_centers,
    make_private_spherical_kmeans,
)
from opendp.mod import Domain, Metric, Transformation


class SphericalKMeans(TransformerMixin, DPTransformer):
    """Differentially private spherical k-means on canonical boolean CSR arrays.

    Rows represent sets of active features, not count vectors. At each iteration,
    cosine assignment is followed by aggregation of unit-normalized rows and
    discrete DP noise. ``max_l1_contribution`` uniformly downweights long records,
    yielding a weighted spherical objective rather than discarding feature IDs.
    Noisy directions are clamped nonnegative and optionally projected to their
    largest coordinates. Zero rows or zero centers have zero cosine similarity;
    assignment ties choose the first center, and projection ties prefer the lowest
    feature ID. Empty clusters receive DP noise.

    Public schema inference uses only dtype and feature count, never private row
    count. Use ``contributions=1`` for add/remove row adjacency; a substitution
    requires ``contributions=2``. Both epsilon-DP and zCDP are supported.
    Standalone ``fit`` mutates this estimator; Context releases a fitted clone.
    ``score`` is the negative sum of ordinary (unweighted) cosine distances.

    :param n_clusters: Number of center directions.
    :param max_iter: Fixed number of adaptively composed private Lloyd updates.
    :param max_features_per_center: Maximum nonzero coordinates per center, or None for unrestricted directions.
    :param max_l1_contribution: Optional positive L1 cap on each unit-normalized contribution, applied uniformly to the entire record.
    :param init: "random" for a public random feature partition, or public/already-DP nonnegative directions of shape (n_clusters, n_features). Never pass unaccounted private records.
    :param random_state: Seed for public initialization only. DP noise remains cryptographically random.

    .. code:: pycon

        >>> import opendp.prelude as dp
        >>> from scipy.sparse import csr_array
        >>> dp.enable_features("contrib")
        >>> X = csr_array([[True, False], [False, True]])
        >>> model = dp.sklearn.cluster.SphericalKMeans(n_clusters=2, max_iter=2, random_state=0)
        >>> model.fit(X, privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(rho=1.0)) is model
        True
        >>> model.transform(X).shape
        (2, 2)

    Noise is dense over the public feature universe (O(n_clusters * n_features)
    memory/work). Pure-DP L1 sensitivity grows with sqrt(n_features) without the
    optional cap; zCDP needs no contribution cap. Public initialization can be much
    more useful for rare clusters than the data-independent default. See the
    :ref:`spherical k-means numerical and utility analysis <spherical-kmeans-analysis>`.
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
        # scipy.copy detaches buffers and discards the FFI __opendp_dict__, which
        # may otherwise retain a private queryable/measurement through the release.
        self.cluster_centers_ = centers.copy()
        self.n_features_in_ = centers.shape[1]

    def _require_fitted(self):
        if not hasattr(self, "cluster_centers_"):
            raise ValueError("model has not been fitted")

    def predict(self, X):
        self._require_fitted()
        return _labels(X, _unit_centers(self.cluster_centers_))

    def transform(self, X):
        self._require_fitted()
        return _distances(X, self.cluster_centers_)

    def score(self, X, y=None):
        return -math.fsum(_np().min(self.transform(X), axis=1))

    def _make_rowwise(self, input_domain, input_metric, output_x_domain, fn):
        desc = input_domain.cast(SklearnFitDescriptor)
        if _check_space(desc.X, input_metric).n_features != self.n_features_in_:
            raise ValueError("input_domain must match the fitted number of features")
        return _make_transformation(
            input_domain,
            input_metric,
            sklearn_fit_domain(output_x_domain, desc.y),
            input_metric,
            lambda data: replace(data, X=fn(data.X)),
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
        return self._make_rowwise(
            input_domain,
            input_metric,
            dp.vector_domain(dp.atom_domain(T=dp.i64)),
            lambda x: _labels(x, unit),
        )
