"""Discrete normalized spherical Lloyd updates on boolean CSR records.

For a row with r active features, the fixed-point contribution is w(r) * x,
where w(r) = floor(Q / sqrt(r)), optionally capped by floor(Q * B / r).
Integer square roots and rational arithmetic establish the sensitivity bounds;
cosine assignment is row-local and need not be exact for the privacy argument.
"""

from __future__ import annotations

import math
from fractions import Fraction
from numbers import Real
from operator import index

import opendp.prelude as dp
from opendp._internal import _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.extras._utilities import to_then
from opendp.extras.scipy import CSRArray2Domain
from opendp.mod import Domain, Measure, Measurement, Metric

_MAX = (1 << 63) - 1


def _np():
    return import_optional_dependency("numpy")


def _sp():
    return import_optional_dependency("scipy.sparse")


def _positive_int(value, name):
    if isinstance(value, bool):
        raise ValueError(f"{name} must be a positive integer")
    value = index(value)
    if value <= 0:
        raise ValueError(f"{name} must be a positive integer")
    return value


def _check_space(domain, metric):
    if metric != dp.symmetric_distance():
        raise ValueError("expected symmetric_distance() (add/remove adjacency)")
    desc = domain.cast(CSRArray2Domain)
    if desc.T != "bool":
        raise ValueError("expected a boolean CSR domain (T=bool)")
    return desc


def _cap(value):
    if value is None:
        return None
    if isinstance(value, bool) or not isinstance(value, Real):
        raise ValueError("max_l1_contribution must be finite and positive")
    # Preserve the exact represented public value, including Fraction and NumPy reals.
    try:
        cap = (
            Fraction(*value.as_integer_ratio())
            if hasattr(value, "as_integer_ratio")
            else Fraction(value)
        )
    except (ValueError, OverflowError):
        raise ValueError("max_l1_contribution must be finite and positive") from None
    if cap <= 0:
        raise ValueError("max_l1_contribution must be finite and positive")
    return cap


def _quantization(d, cap):
    # At least 24 significant bits in even the smallest nonzero row weight.
    # Usually Q=2**32. Increase it using only the public feature count and cap.
    required = math.isqrt(d - 1) + 1
    if cap is not None:
        required = max(
            required, (d * cap.denominator + cap.numerator - 1) // cap.numerator
        )
    q = 1 << max(32, (required * (1 << 24) - 1).bit_length())
    if q > _MAX:
        raise ValueError(
            "feature count and contribution cap exceed int64 fixed-point precision"
        )
    return q


def _row_weights(x, q, cap):
    np = _np()
    sizes, inverse = np.unique(np.diff(x.indptr), return_inverse=True)
    weights = []
    for r in map(int, sizes):
        w = math.isqrt(q * q // r) if r else 0
        if cap is not None and r:
            w = min(w, q * cap.numerator // (cap.denominator * r))
        weights.append(w)
    return np.asarray(weights, dtype=np.int64)[inverse]


def _aggregate(x, labels, weights, k):
    """Coordinatewise min(sum_i w_i*x_i, INT64_MAX), without overflowing.

    Each sparse product has at most INT64_MAX // max(w) rows. Thus *every*
    nonnegative intermediate sum fits int64, regardless of the SciPy summation
    order. Ordinary datasets use one A @ X product. The fallback saturates between
    safe blocks by adding min(block, INT64_MAX - total), never clipping a wrap.
    """
    np, sp = _np(), _sp()
    n, d = x.shape
    block_size = _MAX // max(1, int(weights.max(initial=0)))

    def block(start, stop):
        a = sp.csr_array(
            (weights[start:stop], (labels[start:stop], np.arange(stop - start))),
            shape=(k, stop - start),
        )
        return (a @ x[start:stop]).toarray().ravel()

    if n and n <= block_size:
        return block(0, n)
    total = np.zeros(k * d, dtype=np.int64)
    for start in range(0, n, block_size):
        values = block(start, min(start + block_size, n))
        total += np.minimum(values, _MAX - total)
    return total


def _unit_centers(centers):
    np = _np()
    unit = centers.astype(np.float64)
    # Scale before squaring: supplied public directions may be near float64 limits.
    maxima = unit.max(axis=1).toarray().ravel()
    unit.data /= np.repeat(np.where(maxima > 0, maxima, 1.0), np.diff(unit.indptr))
    norms = np.sqrt(np.asarray(unit.power(2).sum(axis=1)).ravel())
    unit.data /= np.repeat(np.maximum(norms, 1.0), np.diff(unit.indptr))
    return unit


def _labels(x, unit_centers):
    # Full records determine assignment. Ties (including zero rows/centers) go first.
    return (x @ unit_centers.T).toarray().argmax(axis=1)


def _distances(x, centers):
    np = _np()
    scores = (x @ _unit_centers(centers).T).toarray()
    norms = np.sqrt(np.diff(x.indptr).astype(np.float64))
    return 1.0 - np.clip(scores / np.maximum(norms[:, None], 1.0), 0.0, 1.0)


def _initialize(k, d, init, seed):
    np, sp = _np(), _sp()
    if isinstance(init, str):
        if init != "random":
            raise ValueError(
                "init must be 'random' or public nonnegative center directions"
            )
        # A public feature-to-cluster hash table: every feature is represented,
        # unlike sparse random directions that miss almost all rare features.
        buckets = np.random.default_rng(seed).integers(k, size=d)
        return sp.csr_array((np.ones(d), (buckets, np.arange(d))), shape=(k, d))
    source = init if sp.issparse(init) else np.asarray(init)
    if (
        source.shape != (k, d)
        or source.dtype.kind not in "biuf"
        or not np.isfinite(source.data if sp.issparse(source) else source).all()
        or ((source.data if sp.issparse(source) else source) < 0).any()
    ):
        raise ValueError(
            "init must contain finite nonnegative public centers of shape (n_clusters, n_features)"
        )
    # Canonicalize *public* input in floating point, avoiding integer duplicate overflow.
    centers = sp.csr_array(source.astype(np.float64), copy=True)
    centers.sum_duplicates()
    if not np.isfinite(centers.data).all():
        raise ValueError("init contains overflowing duplicate entries")
    centers.eliminate_zeros()
    return centers


def _project(noisy, k, d, m):
    np, sp = _np(), _sp()
    values = np.maximum(np.asarray(noisy, dtype=np.int64).reshape(k, d), 0)
    if m < d:
        for row in values:
            threshold = np.partition(row, -m)[-m]
            above = np.flatnonzero(row > threshold)
            ties = np.flatnonzero(row == threshold)[: m - above.size]
            keep = np.concatenate((above, ties))
            projected = np.zeros(d, dtype=np.int64)
            projected[keep] = row[keep]
            row[:] = projected
    return sp.csr_array(values)


def _upper_float(value):
    """Convert a nonnegative integer bound to f64 without rounding downward."""
    bound = float(value)
    return math.nextafter(bound, math.inf) if bound < value else bound


def _make_sums(domain, metric, centers, q, cap, norm):
    d = domain.cast(CSRArray2Domain).n_features
    k = centers.shape[0]
    unit = _unit_centers(centers)
    sensitivity = q if norm == 2 else math.isqrt(q * q * d)
    if cap is not None:
        # L2 is also bounded by L1, useful when the public cap is below one.
        sensitivity = min(sensitivity, q * cap.numerator // cap.denominator)
    output_metric = dp.l1_distance(T=float) if norm == 1 else dp.l2_distance(T=float)
    return _make_transformation(
        domain,
        metric,
        dp.vector_domain(dp.atom_domain(T=dp.i64), size=k * d),
        output_metric,
        lambda x: _aggregate(x, _labels(x, unit), _row_weights(x, q, cap), k),
        lambda distance: _upper_float(index(distance) * sensitivity),
    )


def make_private_spherical_kmeans(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    d_in: int,
    d_out: float,
    *,
    n_clusters: int = 8,
    max_iter: int = 5,
    max_features_per_center: int | None = None,
    max_l1_contribution: float | None = None,
    init="random",
    random_state: int | None = None,
) -> Measurement:
    """Release normalized spherical Lloyd centers on a public boolean CSR schema.

    Each record contributes its unit direction, optionally uniformly downweighted
    to L1 norm at most ``max_l1_contribution``. Conservative fixed-point int64 sums
    receive discrete Laplace (pure DP) or discrete Gaussian (zCDP) noise. All
    iterations use equal adaptively composed privacy allocations. Negative noisy
    coordinates are clamped, followed by optional top-coordinate projection.

    ``init`` is a public random feature partition or supplied public nonnegative
    directions of shape ``(n_clusters, n_features)``. Never initialize from private
    records without accounting for that access. ``random_state`` affects only
    public initialization, not DP noise. Noise requires O(n_clusters * n_features)
    work and memory, even when input and released centers are sparse.
    """
    d = _check_space(input_domain, input_metric).n_features
    if output_measure not in (dp.max_divergence(), dp.zero_concentrated_divergence()):
        raise ValueError("expected max_divergence() or zero_concentrated_divergence()")
    d_in = _positive_int(d_in, "d_in")
    if (
        isinstance(d_out, bool)
        or not isinstance(d_out, Real)
        or not 0 < d_out < math.inf
    ):
        raise ValueError("d_out must be finite and positive")
    k, iterations = (
        _positive_int(n_clusters, "n_clusters"),
        _positive_int(max_iter, "max_iter"),
    )
    m = (
        d
        if max_features_per_center is None
        else min(d, _positive_int(max_features_per_center, "max_features_per_center"))
    )
    cap = _cap(max_l1_contribution)
    q = _quantization(d, cap)
    centers0 = _initialize(k, d, init, random_state)
    norm = 1 if output_measure == dp.max_divergence() else 2

    def compose(budget):
        return dp.c.make_adaptive_composition(
            input_domain, input_metric, output_measure, d_in, [budget] * iterations
        )

    # A single allocation is exact; otherwise search the rounded composition.
    budget = (
        float(d_out)
        if iterations == 1
        else dp.binary_search_param(
            compose, d_in=d_in, d_out=float(d_out), bounds=(0.0, float(d_out)), T=float
        )
    )
    if budget == 0:
        raise ValueError("d_out is too small to allocate positive iteration budgets")
    composition = compose(budget)
    first = _make_sums(input_domain, input_metric, centers0, q, cap, norm)
    noise = dp.binary_search_chain(
        lambda scale: dp.m.make_noise(
            first.output_domain, first.output_metric, output_measure, scale=scale
        ),
        d_in=first.map(d_in),
        d_out=budget,
        T=float,
    )

    def fit(queryable):
        centers = centers0
        for _ in range(iterations):
            sums = _make_sums(input_domain, input_metric, centers, q, cap, norm)
            centers = _project(queryable(sums >> noise), k, d, m)
        return centers

    return composition >> _new_pure_function(fit)


then_private_spherical_kmeans = to_then(make_private_spherical_kmeans)
