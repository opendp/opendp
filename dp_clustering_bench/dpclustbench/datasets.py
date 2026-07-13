from __future__ import annotations

from pathlib import Path
from typing import Optional, Tuple
import json
import numpy as np

from .common import Bounds


def sample_ball(rng: np.random.Generator, n: int, d: int, radius: float) -> np.ndarray:
    z = rng.normal(size=(n, d))
    norm = np.linalg.norm(z, axis=1, keepdims=True)
    norm = np.maximum(norm, 1e-300)
    direction = z / norm
    # Uniform in volume: radius * U^(1/d).
    radii = radius * rng.random(size=(n, 1)) ** (1.0 / d)
    return direction * radii


def clip_l2(x: np.ndarray, radius: float) -> np.ndarray:
    x = np.asarray(x, dtype=float)
    norms = np.linalg.norm(x, axis=1, keepdims=True)
    scale = np.minimum(1.0, radius / np.maximum(norms, 1e-300))
    return x * scale


def make_gaussian_mixture(
    *,
    n: int,
    d: int,
    k: int,
    radius: float,
    cluster_std: float = 0.05,
    center_radius: float = 0.75,
    imbalance: float = 1.0,
    seed: int = 0,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, Bounds]:
    """Generate an oracle-bounded Gaussian mixture inside an L2 ball.

    Args:
        n: number of records.
        d: dimension.
        k: number of mixture components.
        radius: public L2 radius used by DP algorithms.
        cluster_std: per-coordinate Gaussian standard deviation as a fraction of radius.
        center_radius: centers are sampled in a ball of radius center_radius * radius.
        imbalance: if 1, component weights are uniform. Larger values use a Zipf-like skew.
        seed: RNG seed.

    Returns:
        x, labels, true_centers, bounds.
    """
    if n <= 0 or d <= 0 or k <= 0:
        raise ValueError("n, d, and k must be positive")
    rng = np.random.default_rng(seed)
    centers = sample_ball(rng, k, d, center_radius * radius)

    if imbalance <= 1.0:
        probs = np.full(k, 1.0 / k)
    else:
        weights = 1.0 / np.arange(1, k + 1, dtype=float) ** imbalance
        probs = weights / np.sum(weights)

    labels = rng.choice(k, size=n, p=probs)
    x = centers[labels] + rng.normal(scale=cluster_std * radius, size=(n, d))
    x = clip_l2(x, radius)
    return x.astype(float), labels.astype(int), centers.astype(float), Bounds.symmetric(d, radius)


def make_google_lsh_synthetic(*, seed: int = 0):
    """Synthetic dataset described in the Google DP clustering README.

    100,000 points, 100 dimensions, 64 Gaussian components, per-coordinate
    standard deviation 0.0125, centers sampled uniformly from the radius 0.875
    L2 ball, and all points projected into the unit ball.
    """
    return make_gaussian_mixture(
        n=100_000,
        d=100,
        k=64,
        radius=1.0,
        cluster_std=0.0125,
        center_radius=0.875,
        seed=seed,
    )


def make_sklearn_blobs_paper_shape(*, n: int, d: int, k: int, radius: float = 1.0, seed: int = 0):
    """Synthetic PE-means-style sklearn `make_blobs` dataset.

    The PE-means paper says these datasets were generated with sklearn's
    `make_blobs`, and Table 1 reports the (n, d, k) grid. The exact paper
    seed/standard deviation are not specified in the arXiv text, so these
    settings are transparent and easy to change.
    """
    from sklearn.datasets import make_blobs

    x, labels = make_blobs(
        n_samples=n,
        n_features=d,
        centers=k,
        cluster_std=1.0,
        center_box=(-8.0, 8.0),
        random_state=seed,
    )
    x = clip_l2(x - x.mean(axis=0, keepdims=True), radius)
    return x.astype(float), labels.astype(int), None, Bounds.symmetric(d, radius)


def make_scale_surrogate(*, n: int, d: int, k: int, radius: float = 1.0, seed: int = 0):
    """Surrogate for FastLloyd/PE-means scale datasets.

    The paper says these are generated with R's clusterGeneration package and
    random overlap. In the absence of the original generated files, this creates
    anisotropic blobs with moderate overlap, then projects to an L2 ball.
    """
    rng = np.random.default_rng(seed)
    centers = sample_ball(rng, k, d, 0.75 * radius)
    labels = rng.integers(0, k, size=n)
    x = np.empty((n, d), dtype=float)
    for j in range(k):
        idx = np.flatnonzero(labels == j)
        if idx.size == 0:
            continue
        scales = rng.uniform(0.015, 0.09, size=d) * radius
        x[idx] = centers[j] + rng.normal(size=(idx.size, d)) * scales
    x = clip_l2(x, radius)
    return x, labels.astype(int), centers, Bounds.symmetric(d, radius)


def load_npz(path: str | Path, *, x_key: str = "X", y_key: Optional[str] = "y"):
    data = np.load(path, allow_pickle=True)
    x = np.asarray(data[x_key], dtype=float)
    y = None if y_key is None or y_key not in data else np.asarray(data[y_key])
    centers = None if "centers" not in data else np.asarray(data["centers"], dtype=float)
    radius = None
    if "metadata_json" in data:
        try:
            meta = json.loads(str(data["metadata_json"]))
            radius = meta.get("radius")
        except Exception:
            radius = None
    bounds = Bounds.symmetric(x.shape[1], float(radius)) if radius is not None else Bounds.from_data(x)
    return x, y, centers, bounds
