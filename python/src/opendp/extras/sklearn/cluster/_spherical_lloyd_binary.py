"""Differentially private spherical k-means for boolean CSR datasets.

Dataset rows are sets of active features. Fitting keeps sparse integer centers
and saturating integer feature counts; cosine assignment uses floating point.
Privacy accounting and noise calibration also use floating point.
"""

from __future__ import annotations

import math
from dataclasses import replace
from operator import index
from typing import Any, Callable

from sklearn.base import TransformerMixin

from opendp.extras.scipy import CSRArray2Domain
import opendp.prelude as dp
from opendp._internal import (
    _make_measurement,
    _make_transformation,
    _new_pure_function,
)
from opendp._lib import import_optional_dependency
from opendp.extras._utilities import to_then
from opendp.extras.sklearn._domains import SklearnFitDomain, sklearn_fit_domain
from opendp.extras.sklearn._estimator import _DPTransformer
from opendp.measurements import then_noise
from opendp.mod import Domain, Measure, Measurement, Metric, Transformation


def _np() -> Any:
    return import_optional_dependency("numpy")


def _sp() -> Any:
    return import_optional_dependency("scipy.sparse")


def _int(value, name: str, *, allow_zero: bool = False) -> int:
    if isinstance(value, bool):
        raise ValueError(f"{name} must be an integer")
    try:
        value = index(value)
    except TypeError:
        raise TypeError(f"{name} must be an integer") from None
    if value < (0 if allow_zero else 1):
        raise ValueError(f"{name} must be {'nonnegative' if allow_zero else 'positive'}")
    return value


def _check_symmetric(metric: Metric) -> None:
    if metric != dp.symmetric_distance():
        raise ValueError("expected symmetric_distance() (add/remove adjacency)")


def _check_count_domain(domain: Domain) -> CSRArray2Domain:
    desc = domain.cast(CSRArray2Domain)
    if desc.T != "bool":
        raise ValueError("expected a boolean CSR domain (T=bool)")
    return desc


def _check_budget(output_measure: Measure, d_in, d_out) -> tuple[int, float]:
    if output_measure not in (dp.max_divergence(), dp.zero_concentrated_divergence()):
        raise ValueError("expected max_divergence() or zero_concentrated_divergence()")
    d_in = _int(d_in, "d_in")
    budget = float(d_out)
    if not math.isfinite(budget) or budget <= 0:
        raise ValueError("d_out must be finite and positive")
    return d_in, budget


# --- Sparse math ---


def _check_centers(centers, n_features: int):
    """Centers are already canonical, nonnegative int32 CSR arrays."""
    np, sp = _np(), _sp()
    if (
        not isinstance(centers, sp.csr_array)
        or centers.ndim != 2
        or centers.dtype != np.int32
        or not centers.has_canonical_format
        or (centers.data < 0).any()
        or not centers.data.all()
        or centers.shape[0] == 0
        or centers.shape[1] != n_features
    ):
        raise ValueError("expected nonempty canonical int32 CSR centers of matching width")
    return centers


def _initial_centers(k: int, d: int, m: int, seed: int | None):
    """Public random 0/1 directions, stored as int32 CSR."""
    np, sp = _np(), _sp()
    rng = np.random.default_rng(seed)
    cols = np.concatenate([rng.choice(d, m, replace=False) for _ in range(k)])
    rows = np.repeat(np.arange(k), m)
    return sp.csr_array((np.ones(k * m, np.int32), (rows, cols)), shape=(k, d))


def _clip_rows(x, limit: int):
    """Keep the lowest `limit` feature IDs per row; input is canonical boolean CSR."""
    np, sp = _np(), _sp()
    sizes = np.diff(x.indptr)
    if sizes.max(initial=0) <= limit:
        return x
    within_row = np.arange(x.nnz) - np.repeat(x.indptr[:-1], sizes)
    cols = x.indices[within_row < limit]
    ptr = np.r_[0, np.cumsum(np.minimum(sizes, limit))]
    return sp.csr_array((np.ones(cols.size, bool), cols, ptr), shape=x.shape)


def _cosine_scores(x, centers):
    """Cosine scores up to each input row's common normalization factor.

    Only center normalization matters when selecting the nearest center.
    Zero centers have zero similarity; ties go to the first center.
    """
    np = _np()
    unit = centers.astype(np.float64)
    norms = np.sqrt(np.asarray(unit.power(2).sum(axis=1)).ravel())
    return (x @ unit.T).toarray() / np.maximum(norms, 1.0)


def nearest_center_labels(x, centers):
    """Assign each boolean row to its nearest center by cosine similarity."""
    return _cosine_scores(x, centers).argmax(axis=1).astype(_np().int32)


def _cluster_feature_sums(x, centers, max_active: int):
    """Saturating per-cluster int32 feature counts, flattened in row-major order."""
    np = _np()
    x = _clip_rows(x, max_active)
    labels = nearest_center_labels(x, centers)
    positions = (
        np.repeat(labels.astype(np.int64), np.diff(x.indptr)) * x.shape[1]
        + x.indices
    )
    counts = np.bincount(positions, minlength=centers.shape[0] * x.shape[1])
    counts = np.minimum(counts, np.iinfo(np.int32).max)
    return counts.astype(np.int32)


def _project_centers(noisy_counts, k: int, d: int, m: int):
    """Clamp noisy integer counts to zero and retain m largest per center."""
    np, sp = _np(), _sp()
    noisy = np.asarray(noisy_counts)
    if noisy.dtype != np.int32:
        raise TypeError("the noise release must contain int32 counts")
    weights = np.maximum(noisy.reshape(k, d), 0)
    if m < d:
        keep = np.argpartition(weights, -m, axis=1)[:, -m:]
        mask = np.zeros((k, d), dtype=bool)
        np.put_along_axis(mask, keep, True, axis=1)
        weights = np.where(mask, weights, 0)
    return sp.csr_array(weights)


def _cosine_distances(x, centers):
    """Return cosine distances, including 1 for zero rows or centers."""
    np = _np()
    scores = _cosine_scores(x, centers)
    row_norms = np.sqrt(np.diff(x.indptr).astype(np.float64))
    similarities = np.clip(scores / np.maximum(row_norms[:, None], 1.0), 0.0, 1.0)
    return 1.0 - similarities


def make_cluster_feature_sums(
    input_domain: Domain,
    input_metric: Metric,
    *,
    centers,
    max_active: int,
    norm: int = 2,
) -> Transformation:
    """Per-cluster binary counts; add/remove sensitivity L (L1) or sqrt(L) (L2)."""
    _check_symmetric(input_metric)
    desc = _check_count_domain(input_domain)
    centers = _check_centers(centers, desc.n_features).copy()
    L = min(_int(max_active, "max_active"), desc.n_features)
    k, d = centers.shape[0], desc.n_features

    stability: Callable[[Any], int | float]
    if norm == 1:
        output_metric = dp.l1_distance(T=dp.i64)
        stability = lambda d_in: _int(d_in, "d_in", allow_zero=True) * L
    elif norm == 2:
        output_metric = dp.l2_distance(T=dp.f64)
        stability = lambda distance: float(distance) * math.sqrt(L)
    else:
        raise ValueError("norm must be 1 or 2")

    return _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=dp.i32), size=k * d),
        output_metric,
        lambda x: _cluster_feature_sums(x, centers, L),
        stability,
    )


then_cluster_feature_sums = to_then(make_cluster_feature_sums)


def make_private_spherical_kmeans(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    d_in: int,
    d_out: float,
    *,
    n_clusters: int = 8,
    max_iter: int = 5,
    max_features_per_record: int = 128,
    max_features_per_center: int = 96,
    random_state: int | None = None,
) -> Measurement:
    """Release sparse integer centers via adaptive pure DP or zCDP composition."""
    _check_symmetric(input_metric)
    d_in, budget = _check_budget(output_measure, d_in, d_out)
    d = _check_count_domain(input_domain).n_features
    k = _int(n_clusters, "n_clusters")
    n_iter = _int(max_iter, "max_iter")
    L = min(_int(max_features_per_record, "max_features_per_record"), d)
    m = min(_int(max_features_per_center, "max_features_per_center"), d)
    if random_state is not None:
        random_state = _int(random_state, "random_state", allow_zero=True)

    # The discrete mechanisms retain integer counts and centers.
    # Laplace uses L1 sensitivity L; Gaussian uses L2 sensitivity sqrt(L).
    if output_measure == dp.max_divergence():
        norm = 1
        scale = float(d_in * L * n_iter) / budget
    else:
        norm = 2
        scale = float(d_in) * math.sqrt(L * n_iter / (2.0 * budget))
    if not math.isfinite(scale) or scale <= 0:
        raise ValueError("invalid noise scale")
    scale = math.nextafter(scale, math.inf)
    centers0 = _initial_centers(k, d, m, random_state)

    def release_for(centers):
        return (
            make_cluster_feature_sums(
                input_domain, input_metric, centers=centers, max_active=L,
                norm=norm,
            )
            >> then_noise(output_measure, scale)
            >> dp.as_array()
        )

    per_step = release_for(centers0).map(d_in)
    composition = dp.c.make_adaptive_composition(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        d_in=d_in,
        d_mids=[per_step] * n_iter,
    )

    def fit(queryable):
        centers = centers0
        for _ in range(n_iter):
            noisy = queryable(release_for(centers))
            centers = _project_centers(noisy, k, d, m)
        return centers

    return composition >> _new_pure_function(fit)


then_private_spherical_kmeans = to_then(make_private_spherical_kmeans)


class SphericalKMeans(TransformerMixin, _DPTransformer):
    """Private spherical k-means on boolean CSR arrays."""

    def __init__(
        self,
        n_clusters: int = 8,
        *,
        max_iter: int = 5,
        max_features_per_record: int = 128,
        max_features_per_center: int = 96,
        random_state: int | None = None,
    ):
        # Keep constructor parameters unchanged for sklearn.clone/get_params.
        self.n_clusters = n_clusters
        self.max_iter = max_iter
        self.max_features_per_record = max_features_per_record
        self.max_features_per_center = max_features_per_center
        self.random_state = random_state

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        fit_desc = input_domain.cast(SklearnFitDomain)
        measurement = make_private_spherical_kmeans(
            fit_desc.X, input_metric, output_measure, d_in, d_out,
            n_clusters=self.n_clusters,
            max_iter=self.max_iter,
            max_features_per_record=self.max_features_per_record,
            max_features_per_center=self.max_features_per_center,
            random_state=self.random_state,
        )
        return _make_measurement(
            input_domain, input_metric, measurement.output_measure,
            lambda data: measurement(data.X), measurement.map,
        )

    def _ingest_release(self, centers):
        # Releases may carry __opendp_dict__ references to the measurement and
        # its private queryable. A CSR copy retains only the released arrays.
        self.cluster_centers_ = centers.copy()
        self.n_features_in_ = centers.shape[1]

    def _require_fitted(self):
        if not hasattr(self, "cluster_centers_"):
            raise ValueError("model has not been fitted")

    def predict(self, X):
        self._require_fitted()
        return nearest_center_labels(X, self.cluster_centers_)

    def transform(self, X):
        self._require_fitted()
        return _cosine_distances(X, self.cluster_centers_)

    def score(self, X, y=None):
        np = _np()
        self._require_fitted()

        return -math.fsum(np.min(_cosine_distances(X, self.cluster_centers_), axis=1))

    def _make_rowwise(self, input_domain, input_metric, output_x_domain, fn):
        """Lift a fixed-center, per-row operation into an OpenDP transformation."""
        self._require_fitted()
        _check_symmetric(input_metric)
        fit_desc = input_domain.cast(SklearnFitDomain)
        desc = fit_desc.X.cast(CSRArray2Domain)
        if desc.T != "bool":
            raise ValueError("expected a boolean CSR domain (T=bool)")
        if desc.n_features != self.n_features_in_:
            raise ValueError("input_domain must match the fitted number of features")
        
        output_domain = sklearn_fit_domain(output_x_domain, fit_desc.y)
        function = lambda data: replace(data, X=fn(data.X))
        return _make_transformation(
            input_domain, input_metric, output_domain, input_metric,
            function, lambda d: d,
        )

    def make_transform(self, input_domain: Domain, input_metric: Metric) -> Transformation:
        """Map input rows to cosine distances from the released centers."""
        self._require_fitted()
        centers = self.cluster_centers_.copy()
        output_domain = dp.numpy.array2_domain(
            T=float, num_columns=centers.shape[0]
        )
        return self._make_rowwise(
            input_domain, input_metric, output_domain,
            lambda x: _cosine_distances(x, centers),
        )

    def make_predict(self, input_domain: Domain, input_metric: Metric) -> Transformation:
        """Map input rows to nearest-center labels with unit row stability."""
        self._require_fitted()
        centers = self.cluster_centers_.copy()
        output_domain = dp.vector_domain(dp.atom_domain(T=dp.i32))
        return self._make_rowwise(
            input_domain, input_metric, output_domain,
            lambda x: nearest_center_labels(x, centers),
        )
