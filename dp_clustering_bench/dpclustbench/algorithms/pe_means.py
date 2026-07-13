from __future__ import annotations

from dataclasses import dataclass
from math import gamma, pi, sin, sqrt
from typing import Optional

import numpy as np
from scipy.optimize import brentq
from scipy.stats import norm

from .base import ClusterAlgorithm, FitContext
from ..common import FitResult, Timer


def delta_from_mu_eps(mu: float, eps: float) -> float:
    """delta(eps) for mu-GDP using the standard Gaussian tradeoff formula."""
    if mu <= 0:
        return 0.0
    return float(norm.cdf(-eps / mu + mu / 2.0) - np.exp(eps) * norm.cdf(-eps / mu - mu / 2.0))


def mu_for_eps_delta(eps: float, delta: float) -> float:
    """Largest mu such that mu-GDP implies (eps, delta)-DP."""
    if eps <= 0 or not (0 < delta < 1):
        raise ValueError("epsilon must be positive and delta must be in (0, 1)")

    def f(mu: float) -> float:
        return delta_from_mu_eps(mu, eps) - delta

    lo = 1e-12
    hi = 1.0
    while f(hi) < 0:
        hi *= 2.0
        if hi > 1e6:
            raise RuntimeError("could not bracket GDP mu")
    return float(brentq(f, lo, hi, xtol=1e-12, rtol=1e-12, maxiter=200))


def gaussian_sigma_for_composed_histograms(eps: float, delta: float, iterations: int) -> float:
    """Gaussian stddev for T sensitivity-one histograms under GDP composition."""
    mu_total = mu_for_eps_delta(eps, delta)
    return float(sqrt(iterations) / mu_total)


def gaussian_sigma_for_composed_histograms_rho(rho: float, iterations: int) -> float:
    if rho <= 0:
        raise ValueError("rho must be positive")
    if iterations <= 0:
        raise ValueError("iterations must be positive")
    return float(sqrt(iterations / (2.0 * rho)))


def sample_ball(rng: np.random.Generator, n: int, d: int, radius: float) -> np.ndarray:
    z = rng.normal(size=(n, d))
    z /= np.maximum(np.linalg.norm(z, axis=1, keepdims=True), 1e-300)
    r = radius * rng.random(size=(n, 1)) ** (1.0 / d)
    return z * r


def clip_l2(x: np.ndarray, radius: float) -> np.ndarray:
    norms = np.linalg.norm(x, axis=1, keepdims=True)
    return x * np.minimum(1.0, radius / np.maximum(norms, 1e-300))


def levy_stable_mantegna(
    rng: np.random.Generator,
    shape,
    alpha: float = 1.5,
) -> np.ndarray:
    """Symmetric Levy-stable samples using Mantegna's algorithm."""
    if not (0 < alpha <= 2):
        raise ValueError("alpha must be in (0, 2]")
    sigma_u = (
        gamma(1 + alpha)
        * sin(pi * alpha / 2)
        / (gamma((1 + alpha) / 2) * alpha * 2 ** ((alpha - 1) / 2))
    ) ** (1 / alpha)
    u = rng.normal(0.0, sigma_u, size=shape)
    v = rng.normal(0.0, 1.0, size=shape)
    return u / np.maximum(np.abs(v), 1e-300) ** (1.0 / alpha)


def nearest_votes(x: np.ndarray, candidates: np.ndarray, *, batch_size: int = 4096) -> np.ndarray:
    votes = np.zeros(candidates.shape[0], dtype=float)
    for start in range(0, x.shape[0], batch_size):
        stop = min(start + batch_size, x.shape[0])
        dsq = ((x[start:stop, None, :] - candidates[None, :, :]) ** 2).sum(axis=2)
        labels = np.argmin(dsq, axis=1)
        votes += np.bincount(labels, minlength=candidates.shape[0])
    return votes


def truncate_histogram_to_public_n(noisy: np.ndarray, n: int) -> np.ndarray:
    """Approximate MLE-style postprocessing from PE-means.

    Keep the smallest prefix of high-count buckets whose positive noisy mass
    reaches the public dataset size, and zero out the rest. Negative retained
    counts are clipped to zero.
    """
    y = np.asarray(noisy, dtype=float).copy()
    order = np.argsort(y)[::-1]
    pos = np.maximum(y[order], 0.0)
    cumsum = np.cumsum(pos)
    if cumsum.size == 0 or cumsum[-1] <= 0:
        return np.zeros_like(y)
    keep_count = int(np.searchsorted(cumsum, n, side="left") + 1)
    keep_count = max(1, min(keep_count, len(order)))
    out = np.zeros_like(y)
    keep = order[:keep_count]
    out[keep] = np.maximum(y[keep], 0.0)
    return out


def weighted_kmeans(points: np.ndarray, weights: np.ndarray, k: int, seed: int) -> np.ndarray:
    from sklearn.cluster import KMeans

    weights = np.asarray(weights, dtype=float)
    points = np.asarray(points, dtype=float)
    mask = weights > 0
    if np.sum(mask) == 0:
        rng = np.random.default_rng(seed)
        return points[rng.choice(points.shape[0], size=k, replace=points.shape[0] < k)].copy()
    pts = points[mask]
    w = weights[mask]
    k_eff = min(k, pts.shape[0])
    if k_eff == 1:
        centers = np.average(pts, axis=0, weights=w)[None, :]
    else:
        centers = KMeans(n_clusters=k_eff, n_init=10, random_state=seed).fit(pts, sample_weight=w).cluster_centers_
    if k_eff < k:
        rng = np.random.default_rng(seed + 17)
        extra = pts[rng.choice(pts.shape[0], size=k - k_eff, replace=True)]
        centers = np.vstack([centers, extra])
    return centers


def sphere_packing_initialization(
    rng: np.random.Generator,
    *,
    count: int,
    d: int,
    radius: float,
    min_distance: Optional[float] = None,
    retries_per_radius: int = 2000,
) -> np.ndarray:
    """Simple randomized sphere-packing initialization."""
    if min_distance is None:
        min_distance = radius / 2.0
    selected = []
    current = float(min_distance)
    attempts = 0
    while len(selected) < count:
        point = sample_ball(rng, 1, d, max(radius - current, 1e-12))[0]
        ok = True
        if selected:
            prev = np.vstack(selected)
            ok = bool(np.all(np.linalg.norm(prev - point, axis=1) >= current))
        if ok:
            selected.append(point)
            attempts = 0
            continue
        attempts += 1
        if attempts >= retries_per_radius:
            current *= 0.5
            attempts = 0
    return np.vstack(selected)


@dataclass
class PEMeansConfig:
    iterations: int = 20
    population_size: int = 256
    variations_per_center: Optional[int] = None
    noise_sigma: Optional[float] = None
    rho: Optional[float] = None
    levy_alpha: float = 1.5
    mutation_scale: float = 0.08
    adaptive_population: bool = True
    min_signal_to_noise: float = 3.0
    batch_size: int = 4096


class PEMeans(ClusterAlgorithm):
    """Benchmark-oriented PE-means implementation.

    This is intentionally written as plain numpy/sklearn code so that it can be
    audited and modified. It is not official author code.
    """

    name = "pe_means"

    def __init__(self, config: Optional[PEMeansConfig] = None):
        self.config = config or PEMeansConfig()

    def _mutate(self, rng: np.random.Generator, centers: np.ndarray, radius: float, n_children: int) -> np.ndarray:
        c = self.config
        parent_idx = rng.choice(centers.shape[0], size=n_children, replace=True)
        parents = centers[parent_idx]
        noise = levy_stable_mantegna(rng, parents.shape, alpha=c.levy_alpha)
        children = parents + c.mutation_scale * radius * noise
        return clip_l2(children, radius)

    def fit(self, x: np.ndarray, ctx: FitContext) -> FitResult:
        x = clip_l2(np.asarray(x, dtype=float), ctx.bounds.radius)
        c = self.config
        if c.iterations <= 0:
            raise ValueError("PE-means needs at least one iteration")
        rng = np.random.default_rng(ctx.seed)
        sigma = c.noise_sigma
        if sigma is None:
            if c.rho is not None:
                sigma = gaussian_sigma_for_composed_histograms_rho(c.rho, c.iterations)
            elif ctx.rho is not None:
                sigma = gaussian_sigma_for_composed_histograms_rho(ctx.rho, c.iterations)
            else:
                sigma = gaussian_sigma_for_composed_histograms(ctx.epsilon, ctx.delta, c.iterations)

        population_size = max(c.population_size, ctx.k)
        candidates = sphere_packing_initialization(
            rng,
            count=population_size,
            d=x.shape[1],
            radius=ctx.bounds.radius,
        )
        best_centers = None
        last_weight_sum = 0.0
        effective_population = population_size

        with Timer() as timer:
            for t in range(c.iterations):
                votes = nearest_votes(x, candidates, batch_size=c.batch_size)
                noisy = votes + rng.normal(0.0, sigma, size=votes.shape)
                weights = truncate_histogram_to_public_n(noisy, x.shape[0])
                last_weight_sum = float(np.sum(weights))
                centers = weighted_kmeans(candidates, weights, ctx.k, seed=ctx.seed + 1009 * t)
                best_centers = centers

                if c.adaptive_population:
                    positive = max(1, int(np.sum(weights > 0)))
                    # Keep the candidate histogram from becoming too sparse as
                    # noise starts dominating. This is a benchmark heuristic, not
                    # an additional private access.
                    if last_weight_sum < c.min_signal_to_noise * sigma * max(positive, 1):
                        effective_population = max(ctx.k, effective_population // 2)
                    else:
                        effective_population = min(population_size, max(ctx.k, int(effective_population * 1.2)))

                if t == c.iterations - 1:
                    break
                if c.variations_per_center is None:
                    n_children = max(ctx.k, effective_population - ctx.k)
                else:
                    n_children = max(ctx.k, int(c.variations_per_center) * ctx.k)
                children = self._mutate(rng, centers, ctx.bounds.radius, n_children)
                candidates = np.vstack([centers, children])
                if candidates.shape[0] > population_size:
                    candidates = candidates[:population_size]

        extra = {
            "iterations": c.iterations,
            "population_size": population_size,
            "noise_sigma": float(sigma),
            "levy_alpha": c.levy_alpha,
            "mutation_scale": c.mutation_scale,
            "last_weight_sum": last_weight_sum,
        }
        return self._result(np.asarray(best_centers, dtype=float), timer.elapsed, extra)
