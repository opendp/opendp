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
privacy cost.  Each of ``T`` Lloyd iterations releases the per-cluster feature-sum
matrix ``S`` (shape ``k x n_features``, integer feature counts) once, via the
discrete Gaussian mechanism:

  * Assigning each row to its cosine-nearest center is postprocessing of the
    already-private centers and carries no privacy cost.
  * Under add/remove adjacency (``dp.symmetric_distance``), one row is assigned to a
    single cluster and contributes at most ``max_active`` nonzeros to that cluster's
    block, so the L2 sensitivity of the flattened ``S`` is ``sqrt(max_active)`` --
    independent of ``k`` and ``n_features``.  Each row is clipped to ``max_active``
    nonzero features (a fixed, public, data-independent bound) to enforce that.
  * Each new center is the L2-normalized top-``center_active`` features of its noisy
    sum.  Projecting to a few hundred features removes the ``n_features``-scale
    Gaussian-noise accumulation that would otherwise dominate a dense center.

The ``T`` releases are composed with adaptive composition.  No cluster-size release
is performed: the cosine center direction is ``normalize(sum)`` and does not require
the count.  To release cluster sizes, run a separate DP count primitive over
assignments to the released centers and account for it separately.
"""

from __future__ import annotations

from dataclasses import dataclass, replace
import math
from operator import index
from math import sqrt
from typing import Any, Literal, TYPE_CHECKING

import opendp.prelude as dp
from opendp._internal import (
    _extrinsic_domain,
    _make_transformation,
    _new_pure_function,
)
from opendp._lib import import_optional_dependency
from opendp.extras._utilities import to_then
from opendp.extras.sklearn._estimator import DPEstimator
from opendp.measurements import then_gaussian
from opendp.mod import Domain, Measure, Measurement, Metric, Transformation

if TYPE_CHECKING:  # pragma: no cover
    from scipy import sparse  # type: ignore[import-untyped]


Distance = Literal["cosine", "jaccard", "hamming"]


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
class SphericalKMeansConfig:
    """Algorithm hyperparameters for DP spherical k-means.

    :param iterations: Number of Lloyd iterations ``T`` (one DP release each).
    :param center_active: Number of nonzero features kept per center (top-m).
    :param max_active: Per-row bound on nonzero features; each row is clipped to this
        many, bounding the per-release L2 sensitivity to ``sqrt(max_active)``.
    :param init_active: Number of nonzero features in each random initial center.
    :param distance: Assignment distance; ``"cosine"`` is recommended for binary data.
    :param seed: Public seed for the (public) initialization and center updates.
    """

    iterations: int = 5
    center_active: int = 96
    max_active: int = 128
    init_active: int = 96
    distance: Distance = "cosine"
    seed: int = 0


# stable transformation: per-cluster feature sums
def make_cluster_feature_sums(
    input_domain: Domain,
    input_metric: Metric,
    *,
    centers,
    max_active: int,
    distance: Distance = "cosine",
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
    :param distance: assignment distance, one of ``"cosine"``, ``"jaccard"``, ``"hamming"``
    :return: a Transformation from the dataset to the flattened per-cluster feature sums
    """
    _check_symmetric_distance(input_metric)
    _validate_distance(distance)
    np = _np()
    n_features = _n_features_from_domain(input_domain)
    centers = (
        _as_csr_center(centers) if distance == "cosine" else _ensure_csr_binary(centers)
    )
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
        labels = nearest_center_labels(xc_binary, centers, distance=distance)
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
    n_clusters: int,
    config: SphericalKMeansConfig | None = None,
) -> Measurement:
    """Construct a Measurement that releases spherical (cosine) k-means centers.

    Follows the calibrated-mechanism convention
    ``(input_domain, input_metric, output_measure, d_in, d_out, *, <algorithm params>)``:
    the units of ``d_in`` are defined by ``input_metric`` and those of ``d_out`` by
    ``output_measure``. The per-release discrete-Gaussian scale is chosen analytically
    so the composed measurement satisfies ``map(d_in) <= d_out``; there is no
    noise-scale knob to search.

    Each of ``config.iterations`` cluster-sum releases (see
    ``make_cluster_feature_sums``) is composed with adaptive composition;
    initialization and center updates (top-m projection) are public postprocessing.
    ``n_features`` is read from ``input_domain``.

    Currently narrowed to add/remove adjacency
    (``input_metric == symmetric_distance()``) and zero-concentrated DP
    (``output_measure == zero_concentrated_divergence()``, so ``d_out`` is a scalar ρ).

    :param input_domain: instance of ``sparse_binary_domain(n_features=_)``
    :param input_metric: instance of ``symmetric_distance()``
    :param output_measure: instance of ``zero_concentrated_divergence()``
    :param d_in: upper bound on the number of records that may be added or removed
    :param d_out: privacy budget ρ (zero-concentrated DP)
    :param n_clusters: number of cluster directions to release
    :param config: algorithm hyperparameters, an instance of :class:`SphericalKMeansConfig`
    :return: a Measurement releasing the ``(n_clusters, n_features)`` CSR matrix of L2-normalized centers
    """
    _check_symmetric_distance(input_metric)
    d_in, rho = _check_zcdp_budget(output_measure, d_in, d_out)
    n_features = _n_features_from_domain(input_domain)
    cfg = _validate_config(
        config or SphericalKMeansConfig(), n_clusters=n_clusters, n_features=n_features
    )
    n_clusters = int(n_clusters)
    L = int(cfg.max_active)
    T = int(cfg.iterations)

    # Analytic calibration.  One release has L2 sensitivity ``d_in * sqrt(L)``, so its
    # zCDP loss is ``d_in**2 * L / (2 * scale**2)``.  Requiring the T-fold composition
    # to spend exactly ``rho`` gives ``scale = d_in * sqrt(L * T / (2 * rho))``.
    # Rounding scale up keeps each release at or below ``rho / T``, so the composed map
    # stays at or below ``rho``.
    scale = _scale_with_slack(float(d_in) * sqrt(L * T / (2.0 * rho)))

    init_centers = _random_unit_centers(
        n_clusters, n_features, cfg.init_active, cfg.seed
    )

    def m_sum_for(centers):
        # ``dp.as_array`` keeps the noisy sums as a NumPy array across the FFI boundary.
        return (
            make_cluster_feature_sums(
                input_domain,
                input_metric,
                centers=centers,
                max_active=L,
                distance=cfg.distance,
            )
            >> then_gaussian(scale)
            >> dp.as_array()  # type: ignore[operator]
        )

    step_budget = m_sum_for(init_centers).map(d_in)  # type: ignore[attr-defined]  # <= rho / T
    comp = dp.c.make_adaptive_composition(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        d_in=d_in,
        d_mids=[step_budget] * T,
    )

    def postprocess(qbl):
        centers = init_centers
        for _iteration in range(T):
            noisy = qbl(m_sum_for(centers)).reshape(n_clusters, n_features)
            centers = _project_centers_topm(noisy, cfg.center_active)
        return centers

    return comp >> _new_pure_function(postprocess, TO="ExtrinsicObject")


then_private_spherical_kmeans = to_then(make_private_spherical_kmeans)


# sparse math
def _validate_distance(distance: Distance) -> None:
    if distance not in ("cosine", "jaccard", "hamming"):
        raise ValueError("distance must be 'cosine', 'jaccard', or 'hamming'")


def _center_distances(x, centers, distance: Distance):
    """Dense ``(n_rows, n_centers)`` distance from each row of ``x`` to each center.

    ``cosine`` uses the centers' real-valued weights (cluster directions); the set
    distances ``jaccard`` and ``hamming`` treat centers as binary.
    """
    _validate_distance(distance)
    np = _np()
    x = _ensure_csr_binary(x)
    if distance == "cosine":
        centers = _as_csr_center(centers)
        sims = _l2_normalize_rows(x).dot(_l2_normalize_rows(centers).T)
        sims = sims.toarray() if _sp().issparse(sims) else np.asarray(sims)
        return 1.0 - sims
    centers = _ensure_csr_binary(centers)
    inter = x.dot(centers.T)
    inter = inter.toarray() if _sp().issparse(inter) else np.asarray(inter)
    x_sizes = np.diff(x.indptr)[:, None]
    c_sizes = np.diff(centers.indptr)[None, :]
    if distance == "hamming":
        return x_sizes + c_sizes - 2.0 * inter
    return 1.0 - inter / np.maximum(x_sizes + c_sizes - inter, 1e-12)  # jaccard


def nearest_center_labels(x, centers, *, distance: Distance = "cosine"):
    """Assign each row of ``x`` to its nearest ``center`` under ``distance``.

    :param x: sparse binary matrix of rows to label
    :param centers: cluster centers (real-valued for ``"cosine"``, treated as binary otherwise)
    :param distance: one of ``"cosine"``, ``"jaccard"``, ``"hamming"``
    :return: a NumPy array of nearest-center indices, one per row of ``x``
    """
    return _center_distances(x, centers, distance).argmin(axis=1).astype(_np().int32)


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


def _random_unit_centers(k: int, d: int, active: int, seed: int):
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


def _validate_config(
    config: SphericalKMeansConfig, *, n_clusters: int, n_features: int
) -> SphericalKMeansConfig:
    if n_clusters <= 0:  # pragma: no cover
        raise ValueError("n_clusters must be positive")
    if n_features <= 0:  # pragma: no cover
        raise ValueError("n_features must be positive")
    if config.iterations <= 0:  # pragma: no cover
        raise ValueError("iterations must be positive")
    if config.center_active <= 0:  # pragma: no cover
        raise ValueError("center_active must be positive")
    if config.max_active <= 0:  # pragma: no cover
        raise ValueError("max_active must be positive")
    if config.init_active <= 0:  # pragma: no cover
        raise ValueError("init_active must be positive")
    _validate_distance(config.distance)
    return replace(
        config,
        max_active=min(config.max_active, n_features),
        center_active=min(config.center_active, n_features),
        init_active=min(config.init_active, n_features),
    )


def _scale_with_slack(scale: float) -> float:
    np = _np()
    if scale <= 0:  # pragma: no cover
        raise ValueError("scale must be positive")
    return float(np.nextafter(float(scale), np.inf))


class SphericalKMeans(DPEstimator):
    """Differentially private spherical (cosine) k-means for sparse binary data.

    A scikit-learn-style estimator over :func:`make_private_spherical_kmeans`.  The
    instance carries only algorithm hyperparameters; the privacy budget, input
    domain/metric and output measure are supplied by a Context at fit time::

        est = SphericalKMeans(n_clusters=16)
        est.fit(context.query(rho=0.5))     # Context fills domain/metric/measure/d_in/d_out
        centers = est.cluster_centers_
        labels = est.predict(my_rows)       # postprocessing of the released centers

    Equivalently: ``context.query(rho=0.5).sklearn(est).release()``.  Or without a
    Context, supplying the pieces directly::

        m = est.make(input_domain, input_metric, output_measure, d_in, d_out)
        centers = m(data)

    No labels for the fitted (private) data are produced; assign rows with
    :meth:`predict` on data you hold (postprocessing of the released centers).

    :param n_clusters: Number of cluster directions to release.
    :param config: A :class:`SphericalKMeansConfig` of algorithm hyperparameters.
    :param random_state: Public seed for initialization and center updates.
    """

    def __init__(
        self,
        *,
        n_clusters: int = 8,
        config: SphericalKMeansConfig | None = None,
        random_state: int | None = None,
    ):
        # Keep constructor parameters unchanged for sklearn introspection and clone.
        # Validation and public-seed normalization happen when the domain and budget
        # are available in make.
        self.n_clusters = n_clusters
        self.random_state = random_state
        self.config = config

    def make(
        self, input_domain, input_metric, output_measure, d_in, d_out
    ) -> Measurement:
        """Construct the measurement via :func:`make_private_spherical_kmeans`.

        See :meth:`DPEstimator.make` for the argument convention.

        :return: a Measurement releasing the fitted cluster centers
        """
        config = self.config or SphericalKMeansConfig()
        if self.random_state is not None:
            config = replace(config, seed=int(self.random_state))
        self._fit_config = config
        return make_private_spherical_kmeans(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            n_clusters=self.n_clusters,
            config=config,
        )

    def _prepare_fit_query(self, X, y=None, **fit_params):
        """Accept sklearn's optional ``y`` argument, which clustering ignores."""
        self._reject_fit_params(fit_params)
        return X

    def _ingest_release(self, release) -> None:
        centers = (
            release  # the released value is the (n_clusters, n_features) centers matrix
        )
        self.cluster_centers_ = centers
        self.n_features_in_ = int(centers.shape[1]) if centers.shape[0] else None
        self.config_ = self._fit_config
        self.distance_ = self.config_.distance
        self.n_clusters_ = int(self.n_clusters)
        self.n_iter_ = int(self.config_.iterations)
        del self._fit_config

    # Postprocessing over caller-held data (the released centers are public).
    # These are ordinary sklearn methods; explicit transformation constructors
    # are provided separately for users composing OpenDP operations.
    def _fitted_space(self):
        n_features = getattr(self, "n_features_in_", None)
        if getattr(self, "cluster_centers_", None) is None or n_features is None:  # pragma: no cover
            raise ValueError("model has not been fitted")
        return sparse_binary_domain(n_features), dp.symmetric_distance()

    def _fit_distance_value(self):
        if hasattr(self, "distance_"):
            return self.distance_
        config = self.config or SphericalKMeansConfig()  # pragma: no cover
        return config.distance  # pragma: no cover

    def transform(self, X):
        """Return distances from caller-held rows to each fitted center."""
        self._fitted_space()
        return _center_distances(X, self.cluster_centers_, self._fit_distance_value())

    def predict(self, X):
        """Return the nearest fitted-center label for each caller-held row."""
        np = _np()
        return np.argmin(self.transform(X), axis=1).astype(np.int32)

    def score(self, X, y=None):
        """Return the negative sum of distances to the assigned centers."""
        np = _np()
        return -float(np.sum(np.min(self.transform(X), axis=1)))

    def make_transform(self):
        """Return an OpenDP transformation for explicit composition."""
        input_domain, input_metric = self._fitted_space()
        centers = self.cluster_centers_
        distance = self._fit_distance_value()
        return _make_transformation(
            input_domain,
            input_metric,
            dp.numpy.array2_domain(T=float, num_columns=self.n_clusters_),
            dp.symmetric_distance(),
            lambda x: _center_distances(x, centers, distance),
            lambda d_in: d_in,
        )

    def make_predict(self):
        """Return an OpenDP transformation for explicit label composition."""
        np = _np()
        return self.make_transform() >> _make_transformation(
            dp.numpy.array2_domain(T=float, num_columns=self.n_clusters_),
            dp.symmetric_distance(),
            dp.vector_domain(dp.atom_domain(T=dp.i32)),
            dp.symmetric_distance(),
            lambda dists: np.argmin(dists, axis=1).astype(np.int32),
            lambda d_in: d_in,
        )

    def make_score(self):
        """Return an OpenDP transformation for explicit score composition."""
        np = _np()
        n_features = self.n_features_in_ or 0
        row_bound = n_features if self._fit_distance_value() == "hamming" else 1
        return self.make_transform() >> _make_transformation(
            dp.numpy.array2_domain(T=float, num_columns=self.n_clusters_),
            dp.symmetric_distance(),
            dp.atom_domain(T=dp.f64, nan=False),
            dp.absolute_distance(T=dp.f64),
            lambda dists: -float(np.sum(np.min(dists, axis=1))),
            lambda d_in: float(d_in) * row_bound,
        )

    # -- separately-accounted DP diagnostics, released through a Context query --
    def release_cluster_sizes(self, query):
        """Release DP cluster sizes over the fitted centers, on ``query``'s budget.

        Postprocesses :attr:`predict` (nearest-center labels) into per-cluster counts,
        then adds discrete Gaussian noise. One row falls in one cluster, so the counts
        are ``d_in``-stable. This is a separate release from :meth:`fit` and consumes
        its own share of the budget.

        :param query: a Context query, e.g. ``context.query(rho=...)``
        :return: a length-``n_clusters`` NumPy array of noisy cluster sizes
        """
        return query.sklearn(_MeasurementRelease(self._cluster_sizes)).release()

    def cluster_sizes(self, query):
        """Deprecated alias for :meth:`release_cluster_sizes`."""
        return self.release_cluster_sizes(query)

    def release_silhouette(self, query):
        """Release a DP center-based silhouette score over the fitted centers.

        Postprocesses :attr:`transform` (distances to each center) into the simplified
        silhouette ``s = (b - a) / max(a, b) in [0, 1]`` per row -- ``a``/``b`` being the
        nearest/second-nearest center distances -- then releases the noisy mean. This
        is the cheap center-based variant, not the O(n^2) pairwise silhouette. Separate
        release; consumes its own share of the budget.

        :param query: a Context query, e.g. ``context.query(rho=...)``
        :return: the mean silhouette in ``[0, 1]``
        """
        return query.sklearn(_MeasurementRelease(self._silhouette)).release()

    def silhouette(self, query):
        """Deprecated alias for :meth:`release_silhouette`."""
        return self.release_silhouette(query)

    def _cluster_sizes(self, output_measure, d_in, d_out) -> Measurement:
        np = _np()
        d_in, rho = _check_zcdp_budget(output_measure, d_in, d_out)
        k = self.n_clusters_
        # one row -> one bucket, so the label histogram has L2 sensitivity 1 per row.
        scale = _scale_with_slack(float(d_in) * sqrt(1.0 / (2.0 * rho)))
        counts = _make_transformation(
            dp.vector_domain(dp.atom_domain(T=dp.i32)),
            dp.symmetric_distance(),
            dp.vector_domain(dp.atom_domain(T=dp.i32), size=k),
            dp.l2_distance(T=dp.f64),
            lambda labels: np.bincount(labels, minlength=k).astype(np.int32),
            lambda d_in_: float(d_in_),
        )
        return self.make_predict() >> counts >> then_gaussian(scale) >> dp.as_array()  # type: ignore[operator]

    def _silhouette(self, output_measure, d_in, d_out) -> Measurement:
        np = _np()
        d_in, rho = _check_zcdp_budget(output_measure, d_in, d_out)
        if self.n_clusters_ < 2:
            raise ValueError("silhouette requires at least 2 clusters")
        # per row the statistic is (s, 1) with s in [0, 1], so the L2 sensitivity is sqrt(2).
        scale = _scale_with_slack(float(d_in) * sqrt(2.0) / sqrt(2.0 * rho))

        def reduce_(dists):
            two_nearest = np.partition(dists, 1, axis=1)[:, :2]
            a, b = two_nearest[:, 0], two_nearest[:, 1]
            s = np.where(b > 0, (b - a) / b, 0.0)  # max(a, b) == b since b >= a >= 0
            return np.array([float(s.sum()), float(dists.shape[0])], dtype=np.float64)

        reduce_t = _make_transformation(
            dp.numpy.array2_domain(T=float, num_columns=self.n_clusters_),
            dp.symmetric_distance(),
            dp.vector_domain(dp.atom_domain(T=dp.f64, nan=False), size=2),
            dp.l2_distance(T=dp.f64),
            reduce_,
            lambda d_in_: float(d_in_) * sqrt(2.0),
        )
        noisy = self.make_transform() >> reduce_t >> then_gaussian(scale) >> dp.as_array()  # type: ignore[operator]
        # postprocess: mean = sum(s) / count, clamped to the valid silhouette range [0, 1]
        return noisy >> _new_pure_function(  # type: ignore[operator]
            lambda pair: (
                float(min(1.0, max(0.0, pair[0] / pair[1]))) if pair[1] > 0 else 0.0
            )
        )


class _MeasurementRelease(DPEstimator):
    """Internal adapter that lets a fitted estimator release a self-built measurement
    through the Context ``.sklearn(...)`` bridge (which supplies ``d_in``/``d_out``)."""

    def __init__(self, build):
        self._build = build  # (output_measure, d_in, d_out) -> Measurement

    def make(
        self, input_domain, input_metric, output_measure, d_in, d_out
    ) -> Measurement:
        return self._build(output_measure, d_in, d_out)

    def _ingest_release(
        self, release
    ) -> None:  # unused: released directly via .release()
        pass  # pragma: no cover
