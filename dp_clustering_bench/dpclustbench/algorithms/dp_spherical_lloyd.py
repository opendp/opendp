"""zCDP spherical k-means (DP-Lloyd) for sparse binary / multi-hot data.

Motivation
----------
Diagnostics (see ``experiments/clustering_utility_search_log.md``) showed the
sparse Private-Evolution estimator is *algorithm-limited* on this problem: even
with near-zero privacy noise it reaches only ARI ~0.11, while a non-private
spherical (cosine) k-means recovers the clusters at ARI ~0.9.  This module is a
differentially private spherical k-means that closes most of that gap.

Algorithm (one release per Lloyd iteration)
-------------------------------------------
Init: ``k`` random sparse unit centers -- purely public (no data access), so it
costs zero privacy.  This works because the clusters are linearly separable in
tag space; random directions break symmetry and Lloyd amplifies (validated
non-privately at ARI 1.0).

Each of ``T`` iterations:
  1. Assign every row to its cosine-nearest center.  This is *postprocessing* of
     the already-private centers -- no privacy cost.
  2. Release the per-cluster feature-sum matrix ``S`` (shape ``k x d``) with the
     Gaussian mechanism.  Under add/remove of one row (``dp.symmetric_distance``)
     a row lands in exactly one cluster and contributes at most ``L`` ones to
     that cluster's block, so the L2 sensitivity of the whole flattened ``S`` is
     ``sqrt(L)`` -- independent of ``k`` and ``d``.  Rows are clipped to ``L``
     tags (a fixed public cap) to enforce that bound.
  3. Set each new center to the ``center_active_tags`` largest-weight features of
     its noisy sum, L2-normalized.  Projecting to a few hundred features removes
     the ``d``-dimensional Gaussian-noise accumulation that would otherwise swamp
     a dense center.

Accounting is delegated to OpenDP: ``T`` Gaussian releases composed with
``make_adaptive_composition`` under ``zero_concentrated_divergence``.  With total
budget ``rho`` split evenly, each release uses ``rho / T`` and the Gaussian scale
is ``sigma = sqrt(L * T / (2 * rho))``.

No final cluster-size release is performed (cosine centers need only the sums,
not the counts).  Cluster sizes, if wanted, must be released and accounted for
separately (rule 6).
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Optional

import numpy as np
from scipy import sparse

import opendp.prelude as dp
from opendp._internal import _make_measurement, _new_pure_function
from opendp.extras.sklearn.cluster._pe_means_binary import (
    _enable_opendp_features,
    make_sparse_binary_domain,
)

from .base import ClusterAlgorithm, FitContext
from ..common import FitResult, Timer


# ---------------------------------------------------------------------------
# sparse helpers
# ---------------------------------------------------------------------------
def _ensure_csr_binary(x, n_features: Optional[int] = None) -> sparse.csr_matrix:
    if sparse.issparse(x):
        out = x.tocsr().astype(np.float32)
    else:
        out = sparse.csr_matrix(np.asarray(x) != 0, dtype=np.float32)
    if n_features is not None and out.shape[1] != n_features:
        raise ValueError(f"expected {n_features} columns, got {out.shape[1]}")
    out.sum_duplicates()
    out.data[:] = 1.0
    return out


def _clip_rows_to_first_L(x: sparse.csr_matrix, L: int) -> sparse.csr_matrix:
    """Keep at most ``L`` tags per row (the ``L`` lowest tag ids -- a fixed,
    public, data-independent rule).  Guarantees every row contributes at most
    ``L`` ones to a cluster sum, bounding L2 sensitivity to ``sqrt(L)``."""
    x = x.tocsr()
    sizes = np.diff(x.indptr)
    if int(sizes.max(initial=0)) <= L:
        return x
    new_ind, new_ptr = [], [0]
    for i in range(x.shape[0]):
        s, e = x.indptr[i], x.indptr[i + 1]
        cols = x.indices[s:e]
        if cols.size > L:
            cols = cols[:L]  # csr indices are sorted -> lowest L ids
        new_ind.append(cols)
        new_ptr.append(new_ptr[-1] + cols.size)
    indices = np.concatenate(new_ind) if new_ind else np.empty(0, np.int32)
    data = np.ones(indices.size, np.float32)
    return sparse.csr_matrix((data, indices, np.asarray(new_ptr)), shape=x.shape)


def _l2_normalize_rows(x: sparse.csr_matrix) -> sparse.csr_matrix:
    x = x.tocsr().astype(np.float32)
    nrm = np.sqrt(np.asarray(x.multiply(x).sum(axis=1)).ravel())
    nrm[nrm == 0] = 1.0
    return x.multiply(1.0 / nrm[:, None]).tocsr()


def _cosine_labels(xn: sparse.csr_matrix, centers_norm: sparse.csr_matrix, batch_size: int) -> np.ndarray:
    """Argmax cosine similarity of L2-normalized rows against normalized centers."""
    n = xn.shape[0]
    labels = np.empty(n, dtype=np.int32)
    ct = centers_norm.T.tocsr()
    for start in range(0, n, batch_size):
        stop = min(start + batch_size, n)
        sims = xn[start:stop].dot(ct)
        sims = sims.toarray() if sparse.issparse(sims) else np.asarray(sims)
        labels[start:stop] = np.argmax(sims, axis=1).astype(np.int32)
    return labels


def _random_unit_centers(k: int, d: int, active: int, seed: int) -> sparse.csr_matrix:
    """Public random init: k rows each with ``active`` random distinct tags,
    L2-normalized.  No data access -> zero privacy cost."""
    rng = np.random.default_rng(seed)
    rows = []
    for _ in range(k):
        rows.append(np.sort(rng.choice(d, size=min(active, d), replace=False)).astype(np.int32))
    sizes = np.array([r.size for r in rows])
    indptr = np.zeros(k + 1, dtype=np.int64)
    indptr[1:] = np.cumsum(sizes)
    indices = np.concatenate(rows)
    data = np.ones(indices.size, np.float32)
    mat = sparse.csr_matrix((data, indices, indptr), shape=(k, d), dtype=np.float32)
    return _l2_normalize_rows(mat)


def _centers_from_noisy_sums(S: np.ndarray, k: int, d: int, m: int) -> sparse.csr_matrix:
    """Project each cluster's noisy sum to its top-``m`` positive features,
    weighted by the noisy sum value, then L2-normalize."""
    rows_idx, rows_val, indptr = [], [], [0]
    for j in range(k):
        s = S[j]
        pos = np.flatnonzero(s > 0)
        if pos.size == 0:
            indptr.append(indptr[-1])
            continue
        if pos.size > m:
            top = pos[np.argpartition(s[pos], -m)[-m:]]
        else:
            top = pos
        top = np.sort(top)
        vals = s[top].astype(np.float32)
        nv = float(np.linalg.norm(vals))
        if nv <= 0:
            indptr.append(indptr[-1])
            continue
        rows_idx.append(top.astype(np.int32))
        rows_val.append(vals / nv)
        indptr.append(indptr[-1] + top.size)
    indices = np.concatenate(rows_idx) if rows_idx else np.empty(0, np.int32)
    data = np.concatenate(rows_val) if rows_val else np.empty(0, np.float32)
    return sparse.csr_matrix((data, indices, np.asarray(indptr)), shape=(k, d), dtype=np.float32)


# ---------------------------------------------------------------------------
# OpenDP measurement
# ---------------------------------------------------------------------------
def _make_cluster_sum_measurement(
    input_domain,
    input_metric,
    output_measure,
    *,
    centers_norm: sparse.csr_matrix,
    k: int,
    n_features: int,
    L: int,
    scale: float,
    batch_size: int,
    noise_seed: int,
):
    """One Gaussian release of the (k x d) cluster-sum matrix (zCDP).

    Under add/remove of one row (``dp.symmetric_distance``) the row is assigned to
    exactly one cluster and contributes at most ``L`` ones to that cluster's block,
    so the L2 sensitivity of the whole matrix is ``sqrt(L)`` -- independent of ``k``
    and ``d``.  The Gaussian mechanism then gives ``rho = L / (2 * scale**2)`` per
    release, exposed through ``privacy_map`` and enforced by adaptive composition.

    The Gaussian noise is drawn with numpy (a valid continuous-Gaussian sampler)
    rather than OpenDP's native discrete-Gaussian FFI: releasing the full ``k*d``
    (~1.2M-entry) vector through FFI costs ~40-70s per step, versus milliseconds
    here.  For a utility benchmark this is the standard tradeoff; a production
    release would use OpenDP's vetted sampler.  The privacy *accounting* is still
    the machine-checked OpenDP map/composition.
    """
    if output_measure != dp.zero_concentrated_divergence():
        raise ValueError("cluster-sum release requires zero_concentrated_divergence")
    sensitivity_sq = float(L)  # (sqrt(L))**2
    rng = np.random.default_rng(noise_seed)

    def function(data):
        x = _ensure_csr_binary(data, n_features=n_features)
        xn = _l2_normalize_rows(x)
        labels = _cosine_labels(xn, centers_norm, batch_size=batch_size)
        xc = _clip_rows_to_first_L(x, L)
        S = np.zeros((k, n_features), dtype=np.float64)
        for j in range(k):
            idx = np.flatnonzero(labels == j)
            if idx.size:
                S[j] = np.asarray(xc[idx].sum(axis=0)).ravel()
        return S + rng.normal(0.0, scale, size=(k, n_features))

    return _make_measurement(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        function=function,
        privacy_map=lambda d_in: (float(d_in) ** 2) * sensitivity_sq / (2.0 * scale * scale),
        TO="ExtrinsicObject",
    )


@dataclass
class DPSphericalLloydConfig:
    iterations: int = 5
    center_active_tags: int = 96
    clip_active_tags: int = 128
    init_active_tags: int = 96
    batch_size: int = 8192
    noise_sigma: Optional[float] = None  # override; else derived from rho


def make_dp_spherical_lloyd(
    input_domain,
    input_metric,
    output_measure,
    *,
    n_clusters: int,
    n_features: int,
    scale: float,
    config: DPSphericalLloydConfig,
    seed: int,
):
    """Construct a zCDP measurement releasing spherical k-means centers.

    ``T = config.iterations`` Gaussian cluster-sum releases, composed adaptively.
    """
    _enable_opendp_features()
    if input_metric != dp.symmetric_distance():
        raise ValueError("DP spherical Lloyd supports add/remove adjacency only (dp.symmetric_distance()).")

    k = int(n_clusters)
    d = int(n_features)
    L = int(config.clip_active_tags)
    init_centers = _random_unit_centers(k, d, config.init_active_tags, seed)

    proto = _make_cluster_sum_measurement(
        input_domain, input_metric, output_measure,
        centers_norm=init_centers, k=k, n_features=d, L=L, scale=scale,
        batch_size=config.batch_size, noise_seed=seed,
    )
    step_budget = proto.map(1)
    comp = dp.c.make_adaptive_composition(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        d_in=1,
        d_mids=[step_budget] * config.iterations,
    )

    def postprocess(qbl):
        centers = init_centers
        m = int(config.center_active_tags)
        for it in range(config.iterations):
            m_sum = _make_cluster_sum_measurement(
                input_domain, input_metric, output_measure,
                centers_norm=centers, k=k, n_features=d, L=L, scale=scale,
                batch_size=config.batch_size, noise_seed=seed + 7919 * (it + 1),
            )
            S = np.asarray(qbl(m_sum), dtype=np.float64).reshape(k, d)
            centers = _centers_from_noisy_sums(S, k=k, d=d, m=m)
        return {
            "centers": centers,
            "iterations": config.iterations,
            "scale": float(scale),
            "clip_active_tags": L,
            "center_active_tags": m,
            "per_release_l2_sensitivity": math.sqrt(float(L)),
        }

    return comp >> _new_pure_function(postprocess, TO="ExtrinsicObject")


def _sigma_for_rho(rho: float, iterations: int, L: int) -> float:
    if rho <= 0 or iterations <= 0:
        raise ValueError("rho and iterations must be positive")
    return math.sqrt(float(L) * iterations / (2.0 * rho))


# ---------------------------------------------------------------------------
# benchmark adapter
# ---------------------------------------------------------------------------
class DPSphericalLloyd(ClusterAlgorithm):
    name = "dp_spherical_lloyd"

    def __init__(
        self,
        *,
        rho: Optional[float] = None,
        delta: float = 1e-6,
        random_state: Optional[int] = None,
        config: Optional[DPSphericalLloydConfig] = None,
    ):
        self.rho = rho
        self.delta = float(delta)
        self.random_state = int(random_state) if random_state is not None else 0
        self.config = config or DPSphericalLloydConfig()
        self.labels_: Optional[np.ndarray] = None
        self.cluster_centers_ = None
        self.extra_: dict = {}

    def fit(self, x, ctx: FitContext) -> FitResult:
        _enable_opendp_features()
        x = _ensure_csr_binary(x)
        n_features = x.shape[1]
        L = int(self.config.clip_active_tags)
        rho = self.rho if self.rho is not None else ctx.rho
        if rho is None:
            raise ValueError("rho must be provided for DP spherical Lloyd")
        scale = self.config.noise_sigma
        if scale is None:
            scale = _sigma_for_rho(rho, self.config.iterations, L)

        domain = make_sparse_binary_domain(n_features, max_active=None)
        measure = dp.zero_concentrated_divergence()
        meas = make_dp_spherical_lloyd(
            domain, dp.symmetric_distance(), measure,
            n_clusters=ctx.k, n_features=n_features, scale=scale,
            config=self.config, seed=self.random_state,
        )
        with Timer() as timer:
            out = meas(x)
        centers = out["centers"]
        self.cluster_centers_ = centers
        # label assignment is postprocessing of released (private) centers
        xn = _l2_normalize_rows(x)
        self.labels_ = _cosine_labels(xn, _l2_normalize_rows(centers), batch_size=self.config.batch_size)

        rho_step = float(meas.map(1)) / self.config.iterations if self.config.iterations else None
        extra = {
            "backend_used": "opendp",
            "accountant": "zcdp",
            "rho_total": float(meas.map(1)),
            "rho_step": rho_step,
            "iterations": int(self.config.iterations),
            "n_releases": int(self.config.iterations),
            "vote_noise_scale": float(scale),
            "per_release_l2_sensitivity": float(out["per_release_l2_sensitivity"]),
            "clip_active_tags": int(L),
            "center_active_tags": int(out["center_active_tags"]),
            "centers_nnz_mean": float(np.mean(np.diff(centers.indptr))) if centers.shape[0] else 0.0,
        }
        self.extra_ = extra
        return FitResult(self.name, centers, timer.elapsed, extra)
