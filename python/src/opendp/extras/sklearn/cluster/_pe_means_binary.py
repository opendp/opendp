from __future__ import annotations

from math import gamma, pi, sin, sqrt
from dataclasses import dataclass, replace
from typing import Literal, Optional, Sequence, TYPE_CHECKING, cast

import opendp.prelude as dp
from opendp._internal import (
    _extrinsic_distance,
    _extrinsic_domain,
    _make_measurement,
    _make_transformation,
    _new_pure_function,
)
from opendp._lib import import_optional_dependency
from opendp.mod import _PartialConstructor

try:  # pragma: no cover - optional dependency in user environments
    from scipy.optimize import brentq
    from scipy.stats import norm
except Exception:  # pragma: no cover
    brentq = None
    norm = None

if TYPE_CHECKING:  # pragma: no cover
    import numpy
    from scipy import sparse


Distance = Literal["weighted_jaccard", "weighted_cosine", "weighted_hamming", "jaccard", "hamming"]


@dataclass(frozen=True)
class SparseBinaryDomainDescriptor:
    n_features: int
    max_active: int | None = None


@dataclass
class PEBinaryResult:
    """Postprocessed output of binary private evolution.

    ``centers`` is a scipy CSR matrix with shape ``(k, n_features)``.
    No final cluster-size release is performed here.  To release cluster sizes,
    run a separate DP count-by/nearest-center histogram primitive against these
    released centers and account for it separately.
    """

    centers: object
    iterations: int
    population_size: int
    distance: str
    scale: float
    last_candidate_positive_noisy_buckets: int
    last_candidate_weight_sum: float
    rho_total: float | None = None
    rho_step: float | None = None
    candidate_weight_threshold: float | None = None


@dataclass(frozen=True)
class PEProposalConfig:
    """Public or separately-DP side information for sparse prototype proposals."""

    feature_prior: object = None
    feature_weights: object = None
    feature_groups: object = None
    proposal_graph: object = None
    public_seed_candidates: object = None


@dataclass(frozen=True)
class PEBinaryConfig:
    iterations: int = 16
    population_size: int = 512
    elite_count: int | None = None
    center_active_tags: int = 96
    min_active_tags: int = 16
    max_active_tags: int = 160
    init_active_tags: int = 96
    mutation_drop_prob: float = 0.18
    mutation_add_mean: float = 18.0
    mutation_neighbor_prob: float = 0.50
    mutation_same_group_prob: float = 0.35
    distance: Distance = "weighted_jaccard"
    batch_size: int = 8192
    mode_max_iter: int = 8
    backend: Literal["auto", "opendp", "numpy"] = "auto"
    neighboring: Literal["add_remove", "replace_one"] = "add_remove"
    noise_sigma: float | None = None
    noisy_candidate_weight_threshold_multiplier: float = 1.0
    accountant: Literal["zcdp"] = "zcdp"
    store_internal_labels: bool = False
    feature_weights: object = None
    feature_prior: object = None
    feature_groups: object = None
    proposal_graph: object = None
    public_seed_candidates: object = None
    init_from_data_sample: bool = False
    seed: int = 0


def _np():
    return import_optional_dependency("numpy")


def _sp():
    return import_optional_dependency("scipy.sparse")


def make_sparse_binary_domain(
    n_features: int,
    *,
    max_active: int | None = None,
):
    """Construct an extrinsic domain for sparse/dense binary row datasets.

    The metric for add/remove privacy should be ``dp.symmetric_distance()``.
    The domain accepts scipy sparse matrices or array-like values that coerce to
    a 2D binary matrix with exactly ``n_features`` columns.
    """

    n_features = int(n_features)
    if n_features <= 0:
        raise ValueError("n_features must be positive")
    if max_active is not None and max_active <= 0:
        raise ValueError("max_active must be positive when provided")

    descriptor = SparseBinaryDomainDescriptor(n_features=n_features, max_active=max_active)

    def member(arg) -> bool:
        try:
            _ensure_csr_binary(arg, n_features=n_features, max_active=max_active)
            return True
        except Exception:
            return False

    return _extrinsic_domain(
        f"SparseBinaryMatrix(n_features={n_features},max_active={max_active})",
        member,
        descriptor=descriptor,
    )


def make_private_evolution_binary(
    input_domain,
    input_metric,
    output_measure,
    *,
    n_clusters: int,
    n_features: int,
    scale: float,
    config: PEBinaryConfig | None = None,
    proposal: PEProposalConfig | None = None,
):
    """Construct a measurement that releases PE-means sparse binary prototypes.

    This constructor is narrowed to add/remove adjacency by requiring
    ``input_metric == dp.symmetric_distance()``.  Each PE iteration is one DP
    nearest-candidate vote histogram over the current public/postprocessed
    candidate set.  Adaptivity across iterations is accounted for by OpenDP's
    adaptive-composition combinator.

    The measurement output does not contain final cluster sizes.  Cluster sizes
    are intentionally left to a separate DP primitive over assignments to the
    released centers.
    """

    _enable_opendp_features()
    _check_symmetric_distance(input_metric)
    cfg = _validate_config(config or PEBinaryConfig(), n_clusters=n_clusters, n_features=n_features)
    proposal = proposal or PEProposalConfig()
    n_clusters = int(n_clusters)
    n_features = int(n_features)
    scale = _scale_with_slack(scale)
    threshold = float(cfg.noisy_candidate_weight_threshold_multiplier) * scale

    initial_candidates = _make_initial_candidates(
        n_features=n_features,
        population_size=cfg.population_size,
        init_active_tags=cfg.init_active_tags,
        min_active_tags=cfg.min_active_tags,
        max_active_tags=cfg.max_active_tags,
        seed=cfg.seed,
        proposal=proposal,
    )

    m_step_prototype = make_private_candidate_votes(
        input_domain,
        input_metric,
        output_measure,
        candidates=initial_candidates,
        scale=scale,
        distance=cfg.distance,
        feature_weights=proposal.feature_weights,
        batch_size=cfg.batch_size,
    )
    step_budget = _budget_with_slack(m_step_prototype.map(1))

    comp = dp.c.make_adaptive_composition(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        d_in=1,
        d_mids=[step_budget] * cfg.iterations,
    )

    def postprocess(qbl):
        return _run_private_evolution(
            qbl,
            input_domain=input_domain,
            input_metric=input_metric,
            output_measure=output_measure,
            n_clusters=n_clusters,
            n_features=n_features,
            scale=scale,
            config=cfg,
            proposal=proposal,
            initial_candidates=initial_candidates,
        )

    return comp >> _new_pure_function(postprocess, TO="ExtrinsicObject")


def then_private_evolution_binary(
    output_measure,
    *,
    n_clusters: int,
    n_features: int,
    scale: float,
    config: PEBinaryConfig | None = None,
    proposal: PEProposalConfig | None = None,
):
    """Partial constructor for chaining PE binary clustering from an input space."""

    return _PartialConstructor(
        lambda input_domain, input_metric: make_private_evolution_binary(
            input_domain,
            input_metric,
            output_measure,
            n_clusters=n_clusters,
            n_features=n_features,
            scale=scale,
            config=config,
            proposal=proposal,
        )
    )


def make_private_candidate_votes(
    input_domain,
    input_metric,
    output_measure,
    *,
    candidates,
    scale: float,
    distance: Distance = "weighted_jaccard",
    feature_weights: object = None,
    batch_size: int = 8192,
):
    """Release one noisy nearest-candidate vote histogram.

    Under add/remove adjacency represented by ``SymmetricDistance``, adding or
    removing one row changes exactly one candidate count by one.  Therefore the
    vote-count transformation is 1-stable into either L1 or L2 count metrics.
    """

    _enable_opendp_features()
    _check_symmetric_distance(input_metric)
    vote_counts = make_candidate_vote_counts(
        input_domain,
        input_metric,
        output_metric=_count_metric(output_measure),
        candidates=candidates,
        distance=distance,
        feature_weights=feature_weights,
        batch_size=batch_size,
    )
    return vote_counts >> dp.m.then_noise(output_measure, scale)


def then_private_candidate_votes(
    output_measure,
    *,
    candidates,
    scale: float,
    distance: Distance = "weighted_jaccard",
    feature_weights: object = None,
    batch_size: int = 8192,
):
    """Partial constructor for one noisy nearest-candidate vote histogram."""

    return _PartialConstructor(
        lambda input_domain, input_metric: make_private_candidate_votes(
            input_domain,
            input_metric,
            output_measure,
            candidates=candidates,
            scale=scale,
            distance=distance,
            feature_weights=feature_weights,
            batch_size=batch_size,
        )
    )


def make_candidate_vote_counts(
    input_domain,
    input_metric,
    *,
    output_metric,
    candidates,
    distance: Distance = "weighted_jaccard",
    feature_weights: object = None,
    batch_size: int = 8192,
):
    """Construct the stable nearest-candidate vote-count transformation."""

    _enable_opendp_features()
    _check_symmetric_distance(input_metric)
    candidates = _ensure_csr_binary(candidates)
    if candidates.shape[0] == 0:
        raise ValueError("candidates must be nonempty")
    n_candidates, n_features = candidates.shape
    weights = _feature_weights(feature_weights, n_features, distance)

    def function(data):
        x = _ensure_csr_binary(data, n_features=n_features)
        counts = nearest_vote_counts(
            x,
            candidates,
            distance=distance,
            feature_weights=weights,
            batch_size=batch_size,
        )
        return [int(c) for c in counts]

    return _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=dp.vector_domain(dp.atom_domain(T=dp.i32), size=n_candidates),
        output_metric=output_metric,
        function=function,
        stability_map=lambda d_in: int(d_in),
    )


def then_candidate_vote_counts(
    *,
    output_metric,
    candidates,
    distance: Distance = "weighted_jaccard",
    feature_weights: object = None,
    batch_size: int = 8192,
):
    """Partial constructor for nearest-candidate vote counts."""

    return _PartialConstructor(
        lambda input_domain, input_metric: make_candidate_vote_counts(
            input_domain,
            input_metric,
            output_metric=output_metric,
            candidates=candidates,
            distance=distance,
            feature_weights=feature_weights,
            batch_size=batch_size,
        )
    )


def make_parallel_composition(measurements: Sequence):
    """Parallel composition over a vector of disjoint child datasets.

    This mirrors the tree-clustering helper.  It is useful for callers that
    split data externally and want to run one PE release per partition.
    """

    measurements = list(measurements)
    if not measurements:
        raise ValueError("measurements must be nonempty")
    m0 = measurements[0]
    common_domain = m0.input_domain
    common_metric = m0.input_metric
    output_measure = m0.output_measure
    if not all(m.input_domain == common_domain for m in measurements):
        raise ValueError("all measurements must share the same input domain")
    if not all(m.input_metric == common_metric for m in measurements):
        raise ValueError("all measurements must share the same input metric")
    if not all(m.output_measure == output_measure for m in measurements):
        raise ValueError("all measurements must share the same output measure")

    return _make_measurement(
        input_domain=dp.vector_domain(common_domain, size=len(measurements)),
        input_metric=parallel_distance(common_metric),
        output_measure=output_measure,
        function=lambda data: [m(x) for m, x in zip(measurements, data)],
        privacy_map=lambda d_in: d_in[0] * max(m.map(d_in[1]) for m in measurements),
        TO="ExtrinsicObject",
    )


def parallel_distance(inner_metric):
    return _extrinsic_distance(f"ParallelDistance({inner_metric})", descriptor=inner_metric)


def _run_private_evolution(
    qbl,
    *,
    input_domain,
    input_metric,
    output_measure,
    n_clusters: int,
    n_features: int,
    scale: float,
    config: PEBinaryConfig,
    proposal: PEProposalConfig,
    initial_candidates,
) -> PEBinaryResult:
    np = _np()
    candidates = _ensure_csr_binary(initial_candidates, n_features=n_features)
    rng = np.random.default_rng(config.seed + 1_000_003)
    feature_weights = _feature_weights(proposal.feature_weights, n_features, config.distance)
    elite_count = config.elite_count or max(n_clusters, 2 * n_clusters)
    threshold = float(config.noisy_candidate_weight_threshold_multiplier) * float(scale)
    last_positive = 0
    last_weight_sum = 0.0
    centers = candidates[:n_clusters].copy()

    for iteration in range(config.iterations):
        m_votes = make_private_candidate_votes(
            input_domain,
            input_metric,
            output_measure,
            candidates=candidates,
            scale=scale,
            distance=config.distance,
            feature_weights=feature_weights,
            batch_size=config.batch_size,
        )
        noisy_votes = np.asarray(qbl(m_votes), dtype=float)
        candidate_weights = np.maximum(noisy_votes - threshold, 0.0)
        last_positive = int(np.sum(candidate_weights > 0))
        last_weight_sum = float(candidate_weights.sum())

        centers = _weighted_sparse_modes(
            candidates,
            candidate_weights,
            k=n_clusters,
            center_active_tags=config.center_active_tags,
            distance=config.distance,
            feature_weights=feature_weights,
            max_iter=config.mode_max_iter,
            seed=config.seed + 1009 * iteration,
            batch_size=max(1024, min(config.batch_size, candidates.shape[0])),
        )

        if iteration == config.iterations - 1:
            break

        positive = np.flatnonzero(candidate_weights > 0)
        if positive.size:
            elite_idx = positive[np.argsort(candidate_weights[positive])[::-1]][:elite_count]
            parents = _sp().vstack([centers, candidates[elite_idx]], format="csr")
        else:
            parents = centers

        n_children = max(0, config.population_size - parents.shape[0])
        children = _mutate_candidates(
            parents,
            n_children=n_children,
            rng=rng,
            proposal=proposal,
            config=config,
            n_features=n_features,
        )
        candidates = _sp().vstack([parents, children], format="csr")
        if candidates.shape[0] > config.population_size:
            candidates = candidates[: config.population_size]

    return PEBinaryResult(
        centers=centers,
        iterations=config.iterations,
        population_size=config.population_size,
        distance=config.distance,
        scale=scale,
        last_candidate_positive_noisy_buckets=last_positive,
        last_candidate_weight_sum=last_weight_sum,
        rho_total=None,
        rho_step=None,
        candidate_weight_threshold=threshold,
    )


class PublicProposalModel:
    def __init__(
        self,
        d: int,
        *,
        feature_prior: object = None,
        feature_groups: object = None,
        proposal_graph: object = None,
    ) -> None:
        np = _np()
        sp = _sp()
        self.d = int(d)
        self.feature_prior = _public_positive_vector(feature_prior, d)
        self.feature_prior_probs = _normalize_probs(self.feature_prior)
        self.feature_groups = None if feature_groups is None else np.asarray(feature_groups)
        if self.feature_groups is not None and self.feature_groups.shape != (d,):
            raise ValueError(f"expected feature_groups of length {d}, got {self.feature_groups.shape}")

        self.group_to_indices: dict[int, object] = {}
        self.group_probs: dict[int, object] = {}
        self.group_prior_probs = None
        self.unique_groups = None
        if self.feature_groups is not None:
            unique = np.unique(self.feature_groups)
            self.unique_groups = unique.astype(self.feature_groups.dtype, copy=False)
            group_masses = []
            for g in unique:
                idx = np.flatnonzero(self.feature_groups == g).astype(np.int32)
                self.group_to_indices[int(g)] = idx
                self.group_probs[int(g)] = _normalize_probs(self.feature_prior[idx])
                group_masses.append(float(self.feature_prior[idx].sum()))
            self.group_prior_probs = _normalize_probs(np.asarray(group_masses, dtype=float))

        self.proposal_graph = None
        if proposal_graph is not None:
            graph = sp.csr_matrix(proposal_graph, dtype=float)
            if graph.shape != (d, d):
                raise ValueError(f"expected proposal_graph shape {(d, d)}, got {graph.shape}")
            graph.eliminate_zeros()
            self.proposal_graph = graph

    def sample_seed_tags(self, rng, active: int):
        np = _np()
        tags: set[int] = set()
        while len(tags) < min(active, self.d):
            t = self.sample_from_new_group(rng, tags)
            if t is None:
                t = self.sample_global(rng, tags)
            tags.add(int(t))
        return np.array(sorted(tags), dtype=np.int32)

    def sample_addition(self, rng, tags: set[int], *, neighbor_prob: float, same_group_prob: float) -> int:
        u = float(rng.random())
        if u < neighbor_prob:
            t = self.sample_neighbor(rng, tags, tags)
            if t is not None:
                return t
        if u < neighbor_prob + same_group_prob:
            t = self.sample_from_existing_group(rng, tags, tags)
            if t is not None:
                return t
        t = self.sample_from_new_group(rng, tags)
        if t is not None:
            return t
        return self.sample_global(rng, tags)

    def sample_global(self, rng, exclude: set[int]) -> int:
        for _ in range(32):
            t = int(rng.choice(self.d, p=self.feature_prior_probs))
            if t not in exclude:
                return t
        return int(rng.integers(0, self.d))

    def sample_from_existing_group(self, rng, tags: set[int], exclude: set[int]) -> Optional[int]:
        np = _np()
        if self.feature_groups is None or not tags:
            return None
        tag_arr = np.fromiter(tags, dtype=np.int32)
        anchor = int(rng.choice(tag_arr))
        g = int(self.feature_groups[anchor])
        idx = self.group_to_indices.get(g)
        probs = self.group_probs.get(g)
        if idx is None or probs is None or idx.size == 0:
            return None
        for _ in range(32):
            t = int(idx[rng.choice(idx.size, p=probs)])
            if t not in exclude:
                return t
        return None

    def sample_from_new_group(self, rng, exclude: set[int]) -> Optional[int]:
        if self.unique_groups is None or self.group_prior_probs is None:
            return None
        g = int(self.unique_groups[rng.choice(self.unique_groups.size, p=self.group_prior_probs)])
        idx = self.group_to_indices.get(g)
        probs = self.group_probs.get(g)
        if idx is None or probs is None or idx.size == 0:
            return None
        for _ in range(32):
            t = int(idx[rng.choice(idx.size, p=probs)])
            if t not in exclude:
                return t
        return None

    def sample_neighbor(self, rng, tags: set[int], exclude: set[int]) -> Optional[int]:
        np = _np()
        if self.proposal_graph is None or not tags:
            return None
        tag_arr = np.fromiter(tags, dtype=np.int32)
        for _ in range(16):
            anchor = int(rng.choice(tag_arr))
            row = self.proposal_graph.getrow(anchor)
            if row.indices.size == 0:
                continue
            idx = row.indices.astype(np.int32, copy=False)
            vals = np.asarray(row.data, dtype=float) * self.feature_prior[idx]
            keep = np.array([int(t) not in exclude for t in idx], dtype=bool)
            idx = idx[keep]
            vals = vals[keep]
            if idx.size == 0 or vals.sum() <= 0:
                continue
            probs = _normalize_probs(vals)
            return int(idx[rng.choice(idx.size, p=probs)])
        return None


def nearest_vote_counts(
    x,
    candidates,
    *,
    distance: Distance,
    feature_weights,
    batch_size: int,
):
    np = _np()
    labels = nearest_sparse_labels(
        x,
        candidates,
        distance=distance,
        feature_weights=feature_weights,
        batch_size=batch_size,
    )
    return np.bincount(labels, minlength=candidates.shape[0]).astype(np.int64)


def nearest_sparse_labels(
    x,
    centers,
    *,
    distance: Distance = "weighted_jaccard",
    feature_weights: object = None,
    batch_size: int = 8192,
):
    np = _np()
    x = _ensure_csr_binary(x)
    centers = _ensure_csr_binary(centers)
    d = x.shape[1]
    if centers.shape[1] != d:
        raise ValueError("x and centers have different numbers of columns")
    weights = _feature_weights(feature_weights, d, distance)

    labels = np.empty(x.shape[0], dtype=np.int32)
    x_sizes = _weighted_row_sums(x, weights)
    c_sizes = _weighted_row_sums(centers, weights)
    weighted_centers_t = centers.multiply(weights).T.tocsr()

    for start in range(0, x.shape[0], batch_size):
        stop = min(start + batch_size, x.shape[0])
        inter = x[start:stop].dot(weighted_centers_t)
        inter = inter.toarray() if _sp().issparse(inter) else np.asarray(inter)

        if distance in {"hamming", "weighted_hamming"}:
            dist = x_sizes[start:stop, None] + c_sizes[None, :] - 2.0 * inter
        elif distance in {"jaccard", "weighted_jaccard"}:
            union = x_sizes[start:stop, None] + c_sizes[None, :] - inter
            dist = 1.0 - inter / np.maximum(union, 1e-12)
        elif distance == "weighted_cosine":
            denom = np.sqrt(np.maximum(x_sizes[start:stop, None] * c_sizes[None, :], 1e-12))
            dist = 1.0 - inter / denom
        else:
            raise ValueError(f"unknown distance: {distance}")
        labels[start:stop] = np.argmin(dist, axis=1).astype(np.int32)
    return labels


def _weighted_sparse_modes(
    candidates,
    noisy_candidate_weights,
    *,
    k: int,
    center_active_tags: int,
    distance: Distance,
    feature_weights,
    max_iter: int,
    seed: int,
    batch_size: int,
):
    np = _np()
    sp = _sp()
    rng = np.random.default_rng(seed)
    candidates = _ensure_csr_binary(candidates)
    weights = np.asarray(noisy_candidate_weights, dtype=float)
    d = candidates.shape[1]
    positive = np.flatnonzero(weights > 0)

    if positive.size == 0:
        choice = rng.choice(candidates.shape[0], size=k, replace=candidates.shape[0] < k)
        return candidates[choice].copy()

    order = positive[np.argsort(weights[positive])[::-1]]
    if order.size >= k:
        centers = candidates[order[:k]].copy()
    else:
        extra = rng.choice(candidates.shape[0], size=k - order.size, replace=True)
        centers = sp.vstack([candidates[order], candidates[extra]], format="csr")

    mode_weights = _public_positive_vector(feature_weights, d)
    for _ in range(max_iter):
        labels = nearest_sparse_labels(
            candidates,
            centers,
            distance=distance,
            feature_weights=feature_weights,
            batch_size=batch_size,
        )
        rows = []
        changed = False
        for j in range(k):
            idx = np.flatnonzero(labels == j)
            idx = idx[weights[idx] > 0]
            if idx.size == 0:
                rows.append(centers[j].indices.copy())
                continue
            sub = candidates[idx].astype(float).multiply(weights[idx, None])
            scores = np.asarray(sub.sum(axis=0)).ravel() * mode_weights
            active = min(center_active_tags, int(np.count_nonzero(scores > 0)))
            if active <= 0:
                rows.append(centers[j].indices.copy())
                continue
            top = np.argpartition(scores, -active)[-active:]
            top = np.sort(top[scores[top] > 0].astype(np.int32))
            old = centers[j].indices
            changed |= old.size != top.size or not np.array_equal(old, top)
            rows.append(top)
        centers = _make_sparse_rows_from_sets(rows, d)
        if not changed:
            break
    return centers


def _make_initial_candidates(
    *,
    n_features: int,
    population_size: int,
    init_active_tags: int,
    min_active_tags: int,
    max_active_tags: int,
    seed: int,
    proposal: PEProposalConfig,
):
    np = _np()
    sp = _sp()
    rng = np.random.default_rng(seed)
    if proposal.public_seed_candidates is not None:
        seeds = _ensure_csr_binary(proposal.public_seed_candidates, n_features=n_features)
        if seeds.shape[0] >= population_size:
            idx = rng.choice(seeds.shape[0], size=population_size, replace=False)
            return seeds[idx].copy()
        children = _mutate_candidates(
            seeds,
            n_children=population_size - seeds.shape[0],
            rng=rng,
            proposal=proposal,
            config=PEBinaryConfig(
                population_size=population_size,
                init_active_tags=init_active_tags,
                min_active_tags=min_active_tags,
                max_active_tags=max_active_tags,
                seed=seed,
            ),
            n_features=n_features,
        )
        return sp.vstack([seeds, children], format="csr")

    model = PublicProposalModel(
        n_features,
        feature_prior=proposal.feature_prior,
        feature_groups=proposal.feature_groups,
        proposal_graph=proposal.proposal_graph,
    )
    rows = []
    for _ in range(population_size):
        active = int(np.clip(rng.poisson(init_active_tags), min_active_tags, max_active_tags))
        rows.append(model.sample_seed_tags(rng, active))
    return _make_sparse_rows_from_sets(rows, n_features)


def _mutate_candidates(
    parents,
    *,
    n_children: int,
    rng,
    proposal: PEProposalConfig,
    config: PEBinaryConfig,
    n_features: int,
):
    np = _np()
    if n_children <= 0:
        return _sp().csr_matrix((0, n_features), dtype=np.float32)
    parents = _ensure_csr_binary(parents, n_features=n_features)
    model = PublicProposalModel(
        n_features,
        feature_prior=proposal.feature_prior,
        feature_groups=proposal.feature_groups,
        proposal_graph=proposal.proposal_graph,
    )
    parent_idx = rng.choice(parents.shape[0], size=n_children, replace=True)
    rows = []
    for idx in parent_idx:
        tags = set(map(int, parents[idx].indices))
        if tags:
            drop_n = int(rng.binomial(len(tags), config.mutation_drop_prob))
            if drop_n:
                current = np.fromiter(tags, dtype=np.int32)
                for t in rng.choice(current, size=min(drop_n, current.size), replace=False):
                    tags.discard(int(t))
        add_n = int(max(0, rng.poisson(config.mutation_add_mean)))
        while add_n > 0 and len(tags) < min(config.max_active_tags, n_features):
            tags.add(
                model.sample_addition(
                    rng,
                    tags,
                    neighbor_prob=config.mutation_neighbor_prob,
                    same_group_prob=config.mutation_same_group_prob,
                )
            )
            add_n -= 1
        while len(tags) < min(config.min_active_tags, n_features):
            tags.add(model.sample_addition(rng, tags, neighbor_prob=0.0, same_group_prob=0.0))
        if len(tags) > config.max_active_tags:
            arr = np.fromiter(tags, dtype=np.int32)
            arr = rng.choice(arr, size=config.max_active_tags, replace=False).astype(np.int32)
            tags = set(map(int, arr))
        rows.append(np.array(sorted(tags), dtype=np.int32))
    return _make_sparse_rows_from_sets(rows, n_features)


def _ensure_csr_binary(x, *, n_features: int | None = None, max_active: int | None = None):
    np = _np()
    sp = _sp()
    if sp.issparse(x):
        out = x.tocsr().astype(np.float32)
    else:
        arr = np.asarray(x)
        if arr.ndim == 1:
            if n_features is not None and arr.size == n_features:
                arr = arr.reshape((1, n_features))
            else:
                arr = arr.reshape((-1, 1))
        out = sp.csr_matrix(arr != 0, dtype=np.float32)
    if out.ndim != 2:
        raise ValueError("expected a 2D binary matrix")
    if n_features is not None and out.shape[1] != int(n_features):
        raise ValueError(f"expected {n_features} columns, got {out.shape[1]}")
    out.sum_duplicates()
    if out.data.size and not np.all(np.isfinite(out.data)):
        raise ValueError("data must be finite")
    out.data[:] = 1.0
    if max_active is not None:
        row_sizes = np.diff(out.indptr)
        if np.any(row_sizes > max_active):
            raise ValueError("a row exceeds max_active")
    return out


def _make_sparse_rows_from_sets(rows, d: int):
    np = _np()
    sp = _sp()
    if not rows:
        return sp.csr_matrix((0, d), dtype=np.float32)
    sizes = np.array([len(r) for r in rows], dtype=np.int64)
    indptr = np.zeros(len(rows) + 1, dtype=np.int64)
    indptr[1:] = np.cumsum(sizes)
    indices = np.empty(0, dtype=np.int32) if sizes.sum() == 0 else np.concatenate(
        [np.asarray(r, dtype=np.int32) for r in rows]
    )
    data = np.ones(indices.size, dtype=np.float32)
    out = sp.csr_matrix((data, indices, indptr), shape=(len(rows), d), dtype=np.float32)
    out.sum_duplicates()
    out.data[:] = 1.0
    return out


def _feature_weights(values, d: int, distance: Distance):
    if distance in {"hamming", "jaccard"}:
        return _public_positive_vector(None, d)
    return _public_positive_vector(values, d)


def _public_positive_vector(values, d: int, *, default: float = 1.0):
    np = _np()
    if values is None:
        out = np.full(d, default, dtype=float)
    else:
        out = np.asarray(values, dtype=float)
        if out.shape != (d,):
            raise ValueError(f"expected vector of length {d}, got {out.shape}")
    out = np.where(np.isfinite(out) & (out > 0), out, 0.0)
    if out.sum() <= 0:
        out[:] = default
    return out


def _normalize_probs(values):
    np = _np()
    values = np.asarray(values, dtype=float)
    total = float(values.sum())
    if total <= 0 or not np.isfinite(total):
        return np.full(values.size, 1.0 / max(values.size, 1), dtype=float)
    return values / total


def _weighted_row_sums(x, feature_weights):
    np = _np()
    return np.asarray(x.dot(feature_weights), dtype=float).ravel()


def _count_metric(output_measure):
    if output_measure == dp.zero_concentrated_divergence():
        return dp.l2_distance(T=dp.i32)
    if output_measure == dp.max_divergence():
        return dp.l1_distance(T=dp.i32)
    raise ValueError("output_measure must be MaxDivergence or ZeroConcentratedDivergence")


def _check_symmetric_distance(input_metric) -> None:
    if input_metric != dp.symmetric_distance():
        raise ValueError("PE-means binary currently supports add/remove adjacency only: use dp.symmetric_distance().")


def _validate_config(config: PEBinaryConfig, *, n_clusters: int, n_features: int) -> PEBinaryConfig:
    if n_clusters <= 0:
        raise ValueError("n_clusters must be positive")
    if n_features <= 0:
        raise ValueError("n_features must be positive")
    if config.iterations <= 0:
        raise ValueError("iterations must be positive")
    if config.population_size < n_clusters:
        raise ValueError("population_size must be at least n_clusters")
    if config.center_active_tags <= 0:
        raise ValueError("center_active_tags must be positive")
    if config.min_active_tags <= 0:
        raise ValueError("min_active_tags must be positive")
    if config.max_active_tags < config.min_active_tags:
        raise ValueError("max_active_tags must be at least min_active_tags")
    if config.max_active_tags > n_features:
        raise ValueError("max_active_tags must not exceed n_features")
    if config.init_active_tags <= 0:
        raise ValueError("init_active_tags must be positive")
    if not (0.0 <= config.mutation_drop_prob <= 1.0):
        raise ValueError("mutation_drop_prob must be in [0, 1]")
    if config.mutation_add_mean < 0:
        raise ValueError("mutation_add_mean must be nonnegative")
    if not (0.0 <= config.mutation_neighbor_prob <= 1.0):
        raise ValueError("mutation_neighbor_prob must be in [0, 1]")
    if not (0.0 <= config.mutation_same_group_prob <= 1.0):
        raise ValueError("mutation_same_group_prob must be in [0, 1]")
    if config.mutation_neighbor_prob + config.mutation_same_group_prob > 1.0:
        raise ValueError("mutation_neighbor_prob + mutation_same_group_prob must be <= 1")
    if config.batch_size <= 0:
        raise ValueError("batch_size must be positive")
    if config.mode_max_iter <= 0:
        raise ValueError("mode_max_iter must be positive")
    return config


def _budget_with_slack(budget: float, *, factor: float = 64.0) -> float:
    np = _np()
    eps = np.finfo(float).eps
    return float(np.nextafter(float(budget) * (1.0 + factor * eps), np.inf))


def _scale_with_slack(scale: float) -> float:
    np = _np()
    if scale <= 0:
        raise ValueError("scale must be positive")
    return float(np.nextafter(float(scale), np.inf))


def _enable_opendp_features() -> None:
    try:
        dp.enable_features("contrib", "honest-but-curious")
    except TypeError:  # pragma: no cover - compatibility with older bindings
        dp.enable_features("contrib")
        dp.enable_features("honest-but-curious")


def _delta_from_mu_eps(mu: float, eps: float) -> float:
    np = _np()
    if norm is None:
        raise RuntimeError("scipy is required for automatic epsilon/delta calibration")
    if mu <= 0:
        return 0.0
    return float(norm.cdf(-eps / mu + mu / 2.0) - np.exp(eps) * norm.cdf(-eps / mu - mu / 2.0))


def _mu_for_eps_delta(eps: float, delta: float) -> float:
    if brentq is None:
        raise RuntimeError("scipy is required for automatic epsilon/delta calibration")
    if eps <= 0 or not (0 < delta < 1):
        raise ValueError("epsilon must be positive and delta must be in (0, 1)")

    def f(mu: float) -> float:
        return _delta_from_mu_eps(mu, eps) - delta

    lo = 1e-12
    hi = 1.0
    while f(hi) < 0:
        hi *= 2.0
        if hi > 1e6:
            raise RuntimeError("could not bracket GDP mu")
    return float(brentq(f, lo, hi, xtol=1e-12, rtol=1e-12, maxiter=200))


def _sigma_for_composed_gaussian_histograms_zcdp(eps: float, delta: float, iterations: int) -> float:
    mu_total = _mu_for_eps_delta(eps, delta)
    return float(sqrt(iterations) / mu_total)


def _sigma_for_composed_gaussian_histograms_rho(rho: float, iterations: int) -> float:
    if rho <= 0:
        raise ValueError("rho must be positive")
    if iterations <= 0:
        raise ValueError("iterations must be positive")
    return float(sqrt(iterations / (2.0 * rho)))


def _binary_hamming_loss(x, centers, *, batch_size: int = 8192) -> float:
    np = _np()
    x = _ensure_csr_binary(x)
    centers = _ensure_csr_binary(centers)
    labels = nearest_sparse_labels(x, centers, distance="hamming", batch_size=batch_size)
    x_sizes = np.diff(x.indptr).astype(float)
    c_sizes = np.diff(centers.indptr).astype(float)
    loss = 0.0
    for start in range(0, x.shape[0], batch_size):
        stop = min(start + batch_size, x.shape[0])
        assigned = centers[labels[start:stop]]
        inter = x[start:stop].multiply(assigned).sum(axis=1)
        inter = np.asarray(inter).ravel()
        loss += float(np.sum(x_sizes[start:stop] + c_sizes[labels[start:stop]] - 2.0 * inter))
    return loss


class SparsePrivateEvolutionMeans:
    """Compatibility wrapper around the functional OpenDP PE constructor."""

    name = "sparse_private_evolution_means"

    def __init__(
        self,
        n_features: int,
        n_clusters: int = 8,
        *,
        rho: float | None = None,
        epsilon: float = 1.0,
        delta: float = 1e-6,
        random_state: int | None = None,
        config: PEBinaryConfig | None = None,
    ):
        self.n_features = int(n_features)
        self.n_clusters = int(n_clusters)
        self.rho = rho
        self.epsilon = float(epsilon)
        self.delta = float(delta)
        self.random_state = random_state
        self.config = config or PEBinaryConfig()
        if random_state is not None:
            self.config = replace(self.config, seed=int(random_state))
        self.cluster_centers_ = None
        self.labels_ = None
        self.extra_: dict = {}

    def fit(self, x, y=None):
        max_active = min(self.config.max_active_tags, self.n_features)
        min_active = min(self.config.min_active_tags, max_active)
        if min_active <= 0:
            min_active = 1
        config = replace(self.config, min_active_tags=min_active, max_active_tags=max_active)
        x = _ensure_csr_binary(x, n_features=self.n_features, max_active=None)
        proposal = PEProposalConfig(
            feature_prior=config.feature_prior,
            feature_weights=config.feature_weights,
            feature_groups=config.feature_groups,
            proposal_graph=config.proposal_graph,
            public_seed_candidates=config.public_seed_candidates,
        )
        scale = self.config.noise_sigma
        if scale is None:
            if self.rho is not None:
                scale = _sigma_for_composed_gaussian_histograms_rho(self.rho, config.iterations)
            else:
                scale = _sigma_for_composed_gaussian_histograms_zcdp(
                    self.epsilon,
                    self.delta,
                    config.iterations,
                )
        measure = dp.zero_concentrated_divergence()
        domain = make_sparse_binary_domain(self.n_features, max_active=config.max_active_tags)
        rho_total = self.rho
        rho_step = (rho_total / config.iterations) if rho_total is not None else None
        result = make_private_evolution_binary(
            domain,
            dp.symmetric_distance(),
            measure,
            n_clusters=self.n_clusters,
            n_features=self.n_features,
            scale=scale,
            config=config,
            proposal=proposal,
        )(x)
        backend_used = "opendp"
        self.cluster_centers_ = result.centers
        self.labels_ = nearest_sparse_labels(x, result.centers, distance=config.distance, batch_size=config.batch_size)
        self.extra_ = dict(result.__dict__)
        self.extra_["backend_used"] = backend_used
        self.extra_["accountant"] = config.accountant
        self.extra_["rho_total"] = rho_total
        self.extra_["rho_step"] = rho_step
        self.extra_["iterations"] = config.iterations
        self.extra_["vote_noise_scale"] = float(scale)
        self.extra_["candidate_weight_threshold"] = float(result.candidate_weight_threshold or 0.0)
        return self

    def fit_predict(self, x, y=None):
        self.fit(x, y=y)
        return self.labels_

    def predict(self, x):
        if self.cluster_centers_ is None:
            raise ValueError("model has not been fitted")
        return nearest_sparse_labels(x, self.cluster_centers_, distance=self.config.distance, batch_size=self.config.batch_size)

    def score(self, x, y=None):
        if self.cluster_centers_ is None:
            raise ValueError("model has not been fitted")
        return -_binary_hamming_loss(x, self.cluster_centers_, batch_size=self.config.batch_size)


BinaryPEMeans = SparsePrivateEvolutionMeans
BinaryPEMeansConfig = PEBinaryConfig
SparsePrivateEvolutionConfig = PEBinaryConfig
