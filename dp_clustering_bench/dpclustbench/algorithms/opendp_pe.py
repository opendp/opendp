from __future__ import annotations

from dataclasses import dataclass
from typing import Optional, Sequence

import numpy as np
from scipy import sparse

from .base import ClusterAlgorithm, FitContext
from ..common import FitResult, Timer


def _ensure_csr_binary(x) -> sparse.csr_matrix:
    if sparse.issparse(x):
        out = x.tocsr().astype(np.float32)
    else:
        out = sparse.csr_matrix(np.asarray(x) != 0, dtype=np.float32)
    out.sum_duplicates()
    out.data[:] = 1.0
    return out


@dataclass
class OpenDPSparsePEMeansConfig:
    iterations: int = 16
    population_size: int = 512
    center_active_tags: int = 96
    min_active_tags: int = 16
    max_active_tags: int = 160
    mutation_drop_prob: float = 0.18
    mutation_add_mean: float = 18.0
    distance: str = "weighted_jaccard"
    batch_size: int = 8192
    backend: str = "auto"
    neighboring: str = "add_remove"
    noise_sigma: Optional[float] = None
    noisy_candidate_weight_threshold_multiplier: float = 1.0
    store_internal_labels: bool = True
    feature_weights: object = None
    feature_prior: object = None
    feature_groups: object = None
    proposal_graph: object = None
    public_seed_candidates: object = None
    init_from_data_sample: bool = False
    use_public_seed_candidates: bool = True


class OpenDPSparsePEMeans(ClusterAlgorithm):
    """Adapter around the OpenDP sparse private-evolution estimator."""

    name = "opendp_pe_means"

    def __init__(
        self,
        *,
        rho: Optional[float] = None,
        epsilon: float = 1.0,
        delta: float = 1e-6,
        random_state: Optional[int] = None,
        config: Optional[OpenDPSparsePEMeansConfig] = None,
    ):
        self.rho = rho
        self.epsilon = float(epsilon)
        self.delta = float(delta)
        self.random_state = random_state
        self.config = config or OpenDPSparsePEMeansConfig()
        self.labels_: Optional[np.ndarray] = None
        self.cluster_centers_ = None
        self.extra_: dict = {}

    def _make_model(self, n_features: int, n_clusters: int):
        from opendp.extras.sklearn.cluster import SparsePrivateEvolutionConfig, SparsePrivateEvolutionMeans

        c = self.config
        pe_cfg = SparsePrivateEvolutionConfig(
            iterations=c.iterations,
            population_size=c.population_size,
            center_active_tags=c.center_active_tags,
            min_active_tags=c.min_active_tags,
            max_active_tags=c.max_active_tags,
            mutation_drop_prob=c.mutation_drop_prob,
            mutation_add_mean=c.mutation_add_mean,
            distance=c.distance,
            batch_size=c.batch_size,
            backend=c.backend,
            neighboring=c.neighboring,
            noise_sigma=c.noise_sigma,
            noisy_candidate_weight_threshold_multiplier=c.noisy_candidate_weight_threshold_multiplier,
            store_internal_labels=c.store_internal_labels,
            feature_weights=c.feature_weights,
            feature_prior=c.feature_prior,
            feature_groups=c.feature_groups,
            proposal_graph=c.proposal_graph,
            public_seed_candidates=c.public_seed_candidates if c.use_public_seed_candidates else None,
            init_from_data_sample=c.init_from_data_sample,
        )
        return SparsePrivateEvolutionMeans(
            n_features=n_features,
            n_clusters=n_clusters,
            rho=self.rho,
            epsilon=self.epsilon,
            delta=self.delta,
            random_state=self.random_state,
            config=pe_cfg,
        )

    def fit(self, x, ctx: FitContext) -> FitResult:
        x = _ensure_csr_binary(x)
        model = self._make_model(x.shape[1], ctx.k)
        with Timer() as timer:
            model.fit(x)
        self.labels_ = model.labels_
        self.cluster_centers_ = model.cluster_centers_
        self.extra_ = dict(model.extra_)
        centers = model.cluster_centers_
        if centers is None:
            raise RuntimeError("OpenDP sparse PE model did not produce centers")
        extra = dict(model.extra_)
        extra["centers_nnz_mean"] = float(np.mean(np.diff(centers.indptr))) if centers.shape[0] else 0.0
        return FitResult(self.name, centers, timer.elapsed, extra)
