"""Differentially private spherical (cosine) k-means for sparse binary data.

The estimator clusters sparse binary / multi-hot records -- rows over a large,
mostly-zero feature space, such as one-hot expansions of high-cardinality
categorical columns.  Such data lives in set / cosine space rather than Euclidean
space, so a spherical (cosine) k-means, whose sufficient statistic is the
per-cluster feature-sum vector, fits it well and admits a clean differentially
private realization.

Mechanism (zero-concentrated DP)
--------------------------------
Initialization is public (``k`` random sparse unit centers) and therefore free of
privacy cost. Each of ``max_iter`` Lloyd iterations releases the per-cluster
feature-sum matrix ``S`` (shape ``n_clusters x n_features``, integer feature
counts) once, via the discrete Gaussian mechanism:

  * Assigning each row to its cosine-nearest center is postprocessing of the
    already-private centers and carries no privacy cost.
  * Under add/remove adjacency (``dp.symmetric_distance``), one row is assigned to a
    single cluster and contributes at most ``max_features_per_record`` nonzeros to
    that cluster's block, so the L2 sensitivity of the flattened ``S`` is
    ``sqrt(max_features_per_record)`` -- independent of the cluster and feature
    counts. Each row is clipped to this fixed public bound to enforce that.
  * Each new center is the L2-normalized top-``max_features_per_center`` features
    of its noisy sum. Projecting to a few hundred features removes the ``n_features``-scale
    Gaussian-noise accumulation that would otherwise dominate a dense center.

The ``max_iter`` releases are composed with adaptive composition. No cluster-size
release is performed by the core mechanism: the cosine center direction is
``normalize(sum)`` and does not require the count. Cluster sizes can be released
separately over the already released centers and accounted for independently.
"""

from __future__ import annotations

from dataclasses import dataclass
import math
from operator import index
from math import sqrt
from typing import Any, TYPE_CHECKING

import opendp.prelude as dp
from opendp._internal import (
    _extrinsic_domain,
    _make_transformation,
    _new_pure_function,
)
from opendp._lib import import_optional_dependency
from opendp.extras._utilities import to_then
from opendp.extras.sklearn._estimator import _DPEstimator, _DPFitMixin
from opendp.measurements import then_gaussian
from opendp.mod import Domain, Measure, Measurement, Metric, Transformation

if TYPE_CHECKING:  # pragma: no cover
    from scipy import sparse  # type: ignore[import-untyped]


def _np() -> Any:
    return import_optional_dependency("numpy")


def _sp() -> Any:
    return import_optional_dependency("scipy.sparse")


@dataclass(frozen=True)
class SparseBinaryDomainDescriptor:
    n_features: int
    max_rows: int


def sparse_binary_domain(n_features: int) -> Domain:
    """Construct a domain describing sparse binary row datasets.

    Members are scipy sparse matrices, or array-likes that coerce to a 2D binary
    matrix with exactly ``n_features`` columns. Pair with ``symmetric_distance()``
    for add/remove adjacency.

    :param n_features: number of columns (features) in each dataset
    :return: an ``ExtrinsicDomain`` of sparse binary matrices
    """
    n_features = int(n_features)
    if n_features <= 0:  # pragma: no cover
        raise ValueError("n_features must be positive")

    # Counts are released through OpenDP's i32 discrete Gaussian, so bound the
    # number of rows to keep every per-feature count representable. The ID Graph
    # deployment (roughly 261 million rows) is well below this limit.
    max_rows = 2**31 - 1
    descriptor = SparseBinaryDomainDescriptor(n_features=n_features, max_rows=max_rows)

    def member(arg) -> bool:
        try:
            value = _ensure_csr_binary(arg, n_features=n_features)
            return value.shape[0] <= max_rows
        except Exception:
            return False

    return _extrinsic_domain(
        f"SparseBinaryMatrix(n_features={n_features})",
        member,
        descriptor=descriptor,
    )


@dataclass(frozen=True)
class _SphericalKMeansParams:
    """Validated algorithm parameters retained only while constructing a measurement."""

    n_clusters: int
    max_iter: int
    max_features_per_record: int
    max_features_per_center: int
    random_state: int | None


# stable transformation: per-cluster feature sums
def make_cluster_feature_sums(
    input_domain: Domain,
    input_metric: Metric,
    *,
    centers,
    max_active: int,
) -> Transformation:
    """Construct a Transformation that sums each cluster's assigned rows feature-wise.

    Each row is assigned to its nearest ``center`` (the centers are fixed public
    input, so the assignment of one row is independent of the others), clipped to
    ``max_active`` nonzero features, and added into its cluster's block. The flattened
    ``(n_clusters * n_features,)`` integer counts are returned as a NumPy array.
    The transformation is ``sqrt(max_active)``-stable from ``symmetric_distance()``
    into ``l2_distance(T=f64)``. ``n_features`` is taken from ``input_domain``.

    :param input_domain: instance of ``sparse_binary_domain(n_features=_)``
    :param input_metric: instance of ``symmetric_distance()``
    :param centers: public cluster centers, an ``(n_clusters, n_features)`` matrix
    :param max_active: per-row bound on nonzero features; rows are clipped to this many
    :return: a Transformation from the dataset to the flattened per-cluster feature sums
    """
    _check_symmetric_distance(input_metric)
    np = _np()
    n_features = _n_features_from_domain(input_domain)
    centers = _as_csr_center(centers)
    if centers.shape[0] == 0:  # pragma: no cover
        raise ValueError("centers must be nonempty")
    if centers.shape[1] != n_features:  # pragma: no cover
        raise ValueError(
            f"centers must have {n_features} columns, got {centers.shape[1]}"
        )
    k = centers.shape[0]
    L = int(max_active)
    if L <= 0:  # pragma: no cover
        raise ValueError("max_active must be positive")
    sensitivity = sqrt(float(L))

    def function(data):
        x = _ensure_csr_binary(data, n_features=n_features)
        # Apply the contribution bound before both assignment and aggregation so
        # each iteration is Lloyd's method on one well-defined clipped dataset.
        xc_binary = _clip_rows(x, L)
        labels = nearest_center_labels(xc_binary, centers)
        # Sum in exact integer arithmetic. Float32 accumulation can jump by more
        # than one above 2**24, invalidating the sensitivity bound.
        xc = xc_binary.astype(np.int64)
        sums = np.zeros((k, n_features), dtype=np.int64)
        for j in range(k):
            idx = np.flatnonzero(labels == j)
            if idx.size:
                sums[j] = np.asarray(xc[idx].sum(axis=0)).ravel()
        return sums.ravel().astype(np.int32)

    # Accumulation is exact in i64. The input domain bounds the row count to i32::MAX,
    # making this cast exact; the Gaussian mechanism below is therefore discrete.
    return _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=dp.i32), size=k * n_features),
        dp.l2_distance(T=dp.f64),
        function,
        # Sensitivity (verified here, since _make_transformation asserts nothing):
        # centers are fixed public input, so each row's assignment is independent of
        # the others. Adding/removing one row changes only its cluster's block, by at
        # most ``max_active`` unit entries (rows are clipped), so the L2 change of the
        # flattened sums is <= sqrt(max_active) per row -- and <= d_in * sqrt(max_active)
        # for d_in changed rows. Independent of k and n_features.
        lambda d_in: float(d_in) * sensitivity,
    )


then_cluster_feature_sums = to_then(make_cluster_feature_sums)


# full mechanism
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
    """Construct a Measurement that releases spherical (cosine) k-means centers.

    The calibrated constructor follows ``(input_domain, input_metric,
    output_measure, d_in, d_out, *, <algorithm params>)``. ``d_in`` is measured
    by ``input_metric`` and ``d_out`` by ``output_measure``. Each of ``max_iter``
    cluster-sum releases is adaptively composed; initialization and center updates
    are public postprocessing.

    :param input_domain: instance of ``sparse_binary_domain(n_features=_)``
    :param input_metric: instance of ``symmetric_distance()``
    :param output_measure: instance of ``zero_concentrated_divergence()``
    :param d_in: upper bound on added or removed records
    :param d_out: privacy budget ρ (zero-concentrated DP)
    :param n_clusters: number of cluster directions to release
    :param max_iter: number of Lloyd iterations (one DP release each)
    :param max_features_per_record: contribution bound for nonzero row features
    :param max_features_per_center: top features retained in every center
    :param random_state: public seed for initialization, or ``None`` for fresh entropy
    :return: a Measurement releasing an ``(n_clusters, n_features)`` CSR center matrix
    """
    _check_symmetric_distance(input_metric)
    d_in, rho = _check_zcdp_budget(output_measure, d_in, d_out)
    n_features = _n_features_from_domain(input_domain)
    params = _validate_params(
        n_clusters=n_clusters,
        max_iter=max_iter,
        max_features_per_record=max_features_per_record,
        max_features_per_center=max_features_per_center,
        random_state=random_state,
        n_features=n_features,
    )
    k = params.n_clusters
    L = params.max_features_per_record
    T = params.max_iter

    # One release has L2 sensitivity ``d_in * sqrt(L)``. Rounding the scale up
    # preserves the composed zCDP bound despite finite precision.
    scale = _scale_with_slack(float(d_in) * sqrt(L * T / (2.0 * rho)))
    init_centers = _random_unit_centers(
        k, n_features, params.max_features_per_center, params.random_state
    )

    def m_sum_for(centers):
        return (
            make_cluster_feature_sums(
                input_domain, input_metric, centers=centers, max_active=L
            )
            >> then_gaussian(scale)
            >> dp.as_array()  # type: ignore[operator]
        )

    step_budget = m_sum_for(init_centers).map(d_in)  # type: ignore[attr-defined]
    comp = dp.c.make_adaptive_composition(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        d_in=d_in,
        d_mids=[step_budget] * T,
    )

    def postprocess(qbl):
        centers = init_centers
        for _ in range(T):
            noisy = qbl(m_sum_for(centers)).reshape(k, n_features)
            centers = _project_centers_topm(noisy, params.max_features_per_center)
        return centers

    return comp >> _new_pure_function(postprocess, TO="ExtrinsicObject")


then_private_spherical_kmeans = to_then(make_private_spherical_kmeans)


# sparse math
def _center_distances(x, centers):
    """Dense cosine distances from each row of ``x`` to each center."""
    np = _np()
    x = _ensure_csr_binary(x)
    centers = _as_csr_center(centers)
    sims = _l2_normalize_rows(x).dot(_l2_normalize_rows(centers).T)
    sims = sims.toarray() if _sp().issparse(sims) else np.asarray(sims)
    return 1.0 - sims


def nearest_center_labels(x, centers):
    """Assign each row of ``x`` to its nearest cosine center."""
    return _center_distances(x, centers).argmin(axis=1).astype(_np().int32)


def _project_centers_topm(noisy_sums, m: int):
    """Each center = L2-normalized top-``m`` positive-weight features of its noisy sum."""
    np = _np()
    sums = np.where(noisy_sums > 0, noisy_sums, 0.0).astype(np.float64)
    if sums.shape[1] > m:
        keep = np.argpartition(sums, -m, axis=1)[:, -m:]
        mask = np.zeros(sums.shape, dtype=bool)
        np.put_along_axis(mask, keep, True, axis=1)
        sums *= mask
    norms = np.linalg.norm(sums, axis=1, keepdims=True)
    return _sp().csr_matrix(sums / np.where(norms > 0, norms, 1.0), dtype=np.float32)


def _random_unit_centers(k: int, d: int, active: int, seed: int | None):
    """Public random init: ``k`` L2-normalized rows, each with ``active`` random nonzeros."""
    np = _np()
    rng = np.random.default_rng(seed)
    centers = np.zeros((k, d), dtype=np.float32)
    for j in range(k):
        centers[j, rng.choice(d, size=min(active, d), replace=False)] = 1.0
    norms = np.linalg.norm(centers, axis=1, keepdims=True)
    return _sp().csr_matrix(centers / np.where(norms > 0, norms, 1.0))


def _clip_rows(x, L: int):
    """Keep at most ``L`` nonzero features per row (the ``L`` lowest column ids -- a
    fixed, public, data-independent rule) so each row's L2 contribution is <= sqrt(L).
    """
    np = _np()
    sp = _sp()
    x = x.tocsr()
    if int(np.diff(x.indptr).max(initial=0)) <= L:
        return x
    new_ind, new_ptr = [], [0]
    for i in range(x.shape[0]):
        cols = x.indices[x.indptr[i] : x.indptr[i + 1]][:L]  # csr indices are sorted
        new_ind.append(cols)
        new_ptr.append(new_ptr[-1] + cols.size)
    indices = np.concatenate(new_ind) if new_ind else np.empty(0, np.int32)
    data = np.ones(indices.size, np.float32)
    return sp.csr_matrix(
        (data, indices, np.asarray(new_ptr)), shape=x.shape, dtype=np.float32
    )


def _l2_normalize_rows(x) -> Any:
    np = _np()
    x = x.tocsr().astype(np.float32)
    nrm = np.sqrt(np.asarray(x.multiply(x).sum(axis=1)).ravel())
    nrm[nrm == 0] = 1.0
    return x.multiply(1.0 / nrm[:, None]).tocsr()


def _ensure_csr_binary(x, *, n_features: int | None = None):
    """Coerce ``x`` to a binary (0/1) CSR matrix, optionally checking the column count."""
    np = _np()
    sp = _sp()
    if sp.issparse(x):
        out = x.tocsr().astype(np.float32)
        if out.data.size and not np.all(np.isfinite(out.data)):
            raise ValueError("data must be finite")
        if out.data.size and not np.all((out.data == 0) | (out.data == 1)):
            raise ValueError("data must contain only 0/1 values")
        out.eliminate_zeros()
    else:
        arr = np.asarray(x)
        if arr.ndim == 1:
            arr = (
                arr.reshape((1, n_features))
                if (n_features is not None and arr.size == n_features)
                else arr.reshape((-1, 1))
            )
        if not np.all(np.isfinite(arr)):
            raise ValueError("data must be finite")
        if not np.all((arr == 0) | (arr == 1)):
            raise ValueError("data must contain only 0/1 values")
        out = sp.csr_matrix(arr, dtype=np.float32)
    if out.ndim != 2:  # pragma: no cover
        raise ValueError("expected a 2D binary matrix")
    if n_features is not None and out.shape[1] != int(n_features):  # pragma: no cover
        raise ValueError(f"expected {n_features} columns, got {out.shape[1]}")
    out.sum_duplicates()
    if out.data.size and not np.all((out.data == 0) | (out.data == 1)):
        raise ValueError("data must contain only 0/1 values")
    out.data[:] = 1.0
    return out


def _as_csr_center(x):
    """Coerce ``x`` to a CSR matrix, preserving real-valued (cosine) center weights."""
    np = _np()
    sp = _sp()
    out = (
        x.tocsr().astype(np.float32)
        if sp.issparse(x)
        else sp.csr_matrix(np.asarray(x, dtype=np.float32))
    )
    out.sum_duplicates()
    return out


# validation helpers
def _n_features_from_domain(input_domain) -> int:
    """Read ``n_features`` from a :func:`sparse_binary_domain` descriptor."""
    descriptor = getattr(input_domain, "descriptor", None)
    n_features = getattr(descriptor, "n_features", None)
    if n_features is None:  # pragma: no cover
        raise ValueError(
            "input_domain must be a make_sparse_binary_domain (missing n_features)"
        )
    return int(n_features)


def _check_symmetric_distance(input_metric) -> None:
    if input_metric != dp.symmetric_distance():  # pragma: no cover
        raise ValueError(
            "spherical k-means (binary) supports add/remove adjacency only: use dp.symmetric_distance()."
        )


def _check_zcdp_budget(output_measure, d_in, d_out) -> tuple[int, float]:
    """Validate the zCDP budget shared by the spherical constructors.

    :return: ``(d_in, rho)`` with ``d_in`` a positive int (``symmetric_distance``
        counts changed rows) and ``rho`` a finite, positive float.
    """
    if output_measure != dp.zero_concentrated_divergence():  # pragma: no cover
        raise ValueError(
            "spherical k-means (binary) supports zero_concentrated_divergence only: "
            "d_out must be a scalar rho."
        )
    if isinstance(d_in, bool):
        raise ValueError("d_in must be a positive integer")
    try:
        d_in = index(d_in)
    except TypeError:
        raise TypeError("d_in must be a positive integer") from None
    if d_in <= 0:
        raise ValueError("d_in must be a positive integer")
    rho = float(d_out)
    if not math.isfinite(rho) or rho <= 0:
        raise ValueError("d_out (rho) must be finite and positive")
    return d_in, rho


def _positive_int(value, name: str) -> int:
    if isinstance(value, bool):
        raise ValueError(f"{name} must be a positive integer")
    try:
        value = index(value)
    except TypeError:
        raise TypeError(f"{name} must be a positive integer") from None
    if value <= 0:
        raise ValueError(f"{name} must be a positive integer")
    return value


def _nonnegative_int(value, name: str) -> int:
    if isinstance(value, bool):
        raise ValueError(f"{name} must be a nonnegative integer")
    try:
        value = index(value)
    except TypeError:
        raise TypeError(f"{name} must be a nonnegative integer") from None
    if value < 0:
        raise ValueError(f"{name} must be a nonnegative integer")
    return value


def _validate_params(
    *,
    n_clusters,
    max_iter,
    max_features_per_record,
    max_features_per_center,
    random_state,
    n_features: int,
) -> _SphericalKMeansParams:
    if n_features <= 0:  # pragma: no cover
        raise ValueError("n_features must be positive")
    if random_state is not None:
        random_state = _nonnegative_int(random_state, "random_state")
    return _SphericalKMeansParams(
        n_clusters=_positive_int(n_clusters, "n_clusters"),
        max_iter=_positive_int(max_iter, "max_iter"),
        max_features_per_record=min(
            _positive_int(max_features_per_record, "max_features_per_record"),
            n_features,
        ),
        max_features_per_center=min(
            _positive_int(max_features_per_center, "max_features_per_center"),
            n_features,
        ),
        random_state=random_state,
    )


def _scale_with_slack(scale: float) -> float:
    np = _np()
    if scale <= 0:  # pragma: no cover
        raise ValueError("scale must be positive")
    return float(np.nextafter(float(scale), np.inf))


def make_private_cluster_sizes(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    d_in: int,
    d_out: float,
    *,
    centers,
) -> Measurement:
    """Construct a Measurement releasing noisy nearest-center cluster sizes.

    This is a separately-accounted diagnostic over fixed, already released centers.
    """
    np = _np()
    _check_symmetric_distance(input_metric)
    d_in, rho = _check_zcdp_budget(output_measure, d_in, d_out)
    n_features = _n_features_from_domain(input_domain)
    centers = _as_csr_center(centers)
    if centers.shape[0] <= 0:
        raise ValueError("centers must be nonempty")
    if centers.shape[1] != n_features:
        raise ValueError(
            f"centers must have {n_features} columns, got {centers.shape[1]}"
        )
    k = int(centers.shape[0])
    # Keep labelling and counting in one transformation. The sparse-binary input
    # domain makes this function total; exposing an intermediate unconstrained i32
    # label vector would not, because np.bincount rejects negative labels.
    counts = _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=dp.i32), size=k),
        dp.l2_distance(T=dp.f64),
        lambda data: np.bincount(
            nearest_center_labels(
                _ensure_csr_binary(data, n_features=n_features), centers
            ),
            minlength=k,
        ).astype(np.int32),
        # Adding or removing one row changes exactly one histogram bucket by one.
        lambda distance: float(distance),
    )
    scale = _scale_with_slack(float(d_in) * sqrt(1.0 / (2.0 * rho)))
    return counts >> then_gaussian(scale) >> dp.as_array()  # type: ignore[operator]


class SphericalKMeans(_DPEstimator):
    """Differentially private spherical (cosine) k-means for sparse binary data.

    The estimator's hyperparameters are ordinary sklearn constructor parameters.
    Privacy accounting and the sparse binary domain are supplied by a Context query
    at fit time. Fitted behavior depends solely on ``cluster_centers_``.
    """

    def __init__(
        self,
        n_clusters: int = 8,
        *,
        max_iter: int = 5,
        max_features_per_record: int = 128,
        max_features_per_center: int = 96,
        random_state: int | None = None,
    ):
        # Keep parameters unchanged for sklearn clone/get_params/set_params.
        self.n_clusters = n_clusters
        self.max_iter = max_iter
        self.max_features_per_record = max_features_per_record
        self.max_features_per_center = max_features_per_center
        self.random_state = random_state

    def make(
        self, input_domain, input_metric, output_measure, d_in, d_out
    ) -> Measurement:
        return make_private_spherical_kmeans(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            n_clusters=self.n_clusters,
            max_iter=self.max_iter,
            max_features_per_record=self.max_features_per_record,
            max_features_per_center=self.max_features_per_center,
            random_state=self.random_state,
        )

    def _prepare_fit_query(self, X, y=None, **fit_params):
        self._reject_fit_params(fit_params)
        return X

    def _ingest_release(self, centers) -> None:
        self.cluster_centers_ = centers
        self.n_features_in_ = int(centers.shape[1])

    def _fitted_space(self):
        n_features = getattr(self, "n_features_in_", None)
        if getattr(self, "cluster_centers_", None) is None or n_features is None:
            raise ValueError("model has not been fitted")
        return sparse_binary_domain(n_features), dp.symmetric_distance()

    def _n_fitted_clusters(self) -> int:
        self._fitted_space()
        return int(self.cluster_centers_.shape[0])

    def transform(self, X):
        """Return cosine distances from caller-held rows to fitted centers."""
        self._fitted_space()
        return _center_distances(X, self.cluster_centers_)

    def predict(self, X):
        """Return the nearest fitted-center label for each caller-held row."""
        return nearest_center_labels(X, self.cluster_centers_)

    def score(self, X, y=None):
        """Return the negative sum of distances to assigned fitted centers."""
        np = _np()
        return -float(np.sum(np.min(self.transform(X), axis=1)))

    def make_transform(self):
        """Return an OpenDP transformation for explicit composition."""
        input_domain, input_metric = self._fitted_space()
        centers = self.cluster_centers_
        return _make_transformation(
            input_domain,
            input_metric,
            dp.numpy.array2_domain(T=float, num_columns=self._n_fitted_clusters()),
            dp.symmetric_distance(),
            lambda data: _center_distances(data, centers),
            lambda distance: distance,
        )

    def make_predict(self):
        """Return an OpenDP transformation for explicit label composition."""
        np = _np()
        k = self._n_fitted_clusters()
        return self.make_transform() >> _make_transformation(
            dp.numpy.array2_domain(T=float, num_columns=k),
            dp.symmetric_distance(),
            dp.vector_domain(dp.atom_domain(T=dp.i32)),
            dp.symmetric_distance(),
            lambda distances: np.argmin(distances, axis=1).astype(np.int32),
            lambda distance: distance,
        )

    def make_score(self):
        """Return an OpenDP transformation for explicit score composition."""
        np = _np()
        k = self._n_fitted_clusters()
        return self.make_transform() >> _make_transformation(
            dp.numpy.array2_domain(T=float, num_columns=k),
            dp.symmetric_distance(),
            dp.atom_domain(T=dp.f64, nan=False),
            dp.absolute_distance(T=dp.f64),
            lambda distances: -float(np.sum(np.min(distances, axis=1))),
            lambda distance: float(distance),
        )

    def release_cluster_sizes(self, query):
        """Release DP cluster sizes over the fitted centers on ``query``'s budget."""
        self._fitted_space()
        adapter = _MeasurementRelease(
            lambda input_domain, input_metric, output_measure, d_in, d_out: make_private_cluster_sizes(
                input_domain,
                input_metric,
                output_measure,
                d_in,
                d_out,
                centers=self.cluster_centers_,
            )
        )
        adapter.fit(query)
        return adapter.release_


class _MeasurementRelease(_DPFitMixin):
    """Internal adapter for calibrated non-estimator releases through a Context."""

    def __init__(self, build):
        self._build = build

    def make(
        self, input_domain, input_metric, output_measure, d_in, d_out
    ) -> Measurement:
        return self._build(input_domain, input_metric, output_measure, d_in, d_out)

    def _ingest_release(self, release) -> None:
        self.release_ = release
