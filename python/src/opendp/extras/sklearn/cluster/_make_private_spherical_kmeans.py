"""Discrete normalized spherical Lloyd updates on boolean CSR records."""

from __future__ import annotations

import math
from fractions import Fraction
from numbers import Real
from operator import index

import numpy as np
import scipy.sparse as sp

import opendp.prelude as dp
from opendp._internal import _make_transformation, _new_pure_function
from opendp.extras._utilities import to_then
from opendp.extras.scipy import CSRArray2Descriptor
from opendp.mod import Domain, Measure, Measurement, Metric
from opendp.typing import RuntimeType, RuntimeTypeDescriptor


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
    desc = domain.cast(CSRArray2Descriptor)
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


def _row_weights(x, q, cap, dtype):
    sizes, inverse = np.unique(np.diff(x.indptr), return_inverse=True)
    weights = []
    for r in map(int, sizes):
        w = math.isqrt(q * q // r) if r else 0
        if cap is not None and r:
            w = min(w, q * cap.numerator // (cap.denominator * r))
        weights.append(w)
    return np.asarray(weights, dtype=dtype)[inverse]


def _aggregate(x, labels, weights, k):
    """Saturating sums in the weights' integer dtype, with no overflowing product."""
    n, d = x.shape
    limit = int(np.iinfo(weights.dtype).max)
    # Each nonnegative product sums at most limit // max(weight) rows, so every
    # intermediate fits. Saturate between safe blocks, never after a wrapped sum.
    block_size = limit // max(1, int(weights.max(initial=0)))

    def block(start, stop):
        a = sp.csr_array(
            (weights[start:stop], (labels[start:stop], np.arange(stop - start))),
            shape=(k, stop - start),
        )
        return (a @ x[start:stop]).toarray().ravel()

    if n and n <= block_size:
        return block(0, n)
    total = np.zeros(k * d, dtype=weights.dtype)
    for start in range(0, n, block_size):
        values = block(start, min(start + block_size, n))
        total += np.minimum(values, limit - total)
    return total


def _unit_centers(centers):
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
    scores = (x @ _unit_centers(centers).T).toarray()
    norms = np.sqrt(np.diff(x.indptr).astype(np.float64))
    return 1.0 - np.clip(scores / np.maximum(norms[:, None], 1.0), 0.0, 1.0)


def _initialize(k, d, init, seed):
    if isinstance(init, str):
        if init != "random":
            raise ValueError(
                "init must be 'random' or public nonnegative center directions"
            )
        # Public random partition of the full feature universe.
        buckets = np.random.default_rng(seed).integers(k, size=d)
        return sp.csr_array((np.ones(d), (buckets, np.arange(d))), shape=(k, d))
    source = init if sp.issparse(init) else np.asarray(init)
    if source.dtype.kind not in "biuf":
        raise ValueError("init must contain real public directions")
    # Convert before combining duplicate coordinates to avoid integer overflow.
    centers = sp.csr_array(source.astype(np.float64), copy=True)
    centers.sum_duplicates()
    if (
        centers.shape != (k, d)
        or not np.isfinite(centers.data).all()
        or (centers.data < 0).any()
    ):
        raise ValueError(
            "init must contain finite nonnegative public centers of shape (n_clusters, n_features)"
        )
    centers.eliminate_zeros()
    return centers


def _project(noisy, k, d, m, dtype):
    values = np.maximum(np.asarray(noisy, dtype=dtype).reshape(k, d), 0)
    if m < d:
        for row in values:
            threshold = np.partition(row, -m)[-m]
            above = np.flatnonzero(row > threshold)
            ties = np.flatnonzero(row == threshold)[: m - above.size]
            keep = np.concatenate((above, ties))
            projected = np.zeros(d, dtype=dtype)
            projected[keep] = row[keep]
            row[:] = projected
    return sp.csr_array(values)


def _upper_float(value):
    """Convert a nonnegative integer bound to f64 without rounding downward."""
    bound = float(value)
    return math.nextafter(bound, math.inf) if bound < value else bound


def _make_sums(domain, metric, centers, q, cap, norm, dtype):
    d = domain.cast(CSRArray2Descriptor).n_features
    k = centers.shape[0]
    unit = _unit_centers(centers)
    # A row contributes at one fixed-center assignment: r*w*w <= Q*Q and
    # r*w <= floor(Q*sqrt(d)). Saturation is nonexpansive in both L1 and L2.
    sensitivity = q if norm == 2 else math.isqrt(q * q * d)
    if cap is not None:
        # L2 is also bounded by L1, useful when the public cap is below one.
        sensitivity = min(sensitivity, q * cap.numerator // cap.denominator)
    output_metric = dp.l1_distance(T=float) if norm == 1 else dp.l2_distance(T=float)
    return _make_transformation(
        domain,
        metric,
        dp.vector_domain(dp.atom_domain(T=dtype.type), size=k * d),
        output_metric,
        lambda x: _aggregate(x, _labels(x, unit), _row_weights(x, q, cap, dtype), k),
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
    T: RuntimeTypeDescriptor = dp.i64,
) -> Measurement:
    """Release sparse center directions using private normalized spherical Lloyd updates.

    Uses discrete Laplace noise for pure DP or discrete Gaussian noise for zCDP,
    with equal adaptively composed iteration budgets. Returns nonnegative CSR
    centers of dtype ``T``. Noise requires O(n_clusters * n_features) memory.

    :param input_domain: Canonical boolean CSR domain with a public feature count.
    :param input_metric: Symmetric distance (add/remove row adjacency).
    :param output_measure: Max divergence or zero-concentrated divergence.
    :param d_in: Positive group distance; a replacement costs two row edits.
    :param d_out: Positive finite privacy allowance.
    :param n_clusters: Number of center directions.
    :param max_iter: Fixed number of private Lloyd updates.
    :param max_features_per_center: Maximum nonzero center coordinates, or None.
    :param max_l1_contribution: Optional L1 cap, uniformly downweighting unit-normalized records.
    :param init: "random" for a public feature partition, or public/already-DP nonnegative center directions of shape (n_clusters, n_features).
    :param random_state: Seed for public initialization only, never DP noise.
    :param T: Integer aggregate/noise dtype: i64 (default) or i32. Configurations that could quantize nonempty records to zero are rejected.
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
    T = RuntimeType.parse(T)
    if T not in (dp.i32, dp.i64):
        raise ValueError("T must be i32 or i64")
    dtype = np.dtype(np.int32 if T == dp.i32 else np.int64)
    # Fractional resolution versus headroom: about 2 million (i32) or 5e11 (i64)
    # unit contributions fit in one coordinate before saturation.
    q = 1 << (10 if T == dp.i32 else 24)
    if d > q * q or (cap is not None and q * cap.numerator < d * cap.denominator):
        raise ValueError(
            "feature count and L1 cap may quantize nonempty records to zero"
        )
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
    first = _make_sums(input_domain, input_metric, centers0, q, cap, norm, dtype)
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
            sums = _make_sums(input_domain, input_metric, centers, q, cap, norm, dtype)
            centers = _project(queryable(sums >> noise), k, d, m, dtype)
        return centers

    return composition >> _new_pure_function(fit)


then_private_spherical_kmeans = to_then(make_private_spherical_kmeans)
