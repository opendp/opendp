from __future__ import annotations

from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any, Dict, Optional
import time

import numpy as np


Array = np.ndarray


@dataclass
class Bounds:
    lower: Array
    upper: Array
    radius: float

    @classmethod
    def symmetric(cls, dim: int, radius: float) -> "Bounds":
        lower = np.full(dim, -float(radius), dtype=float)
        upper = np.full(dim, float(radius), dtype=float)
        return cls(lower=lower, upper=upper, radius=float(radius))

    @classmethod
    def from_data(cls, x: Array, pad: float = 0.0) -> "Bounds":
        lower = np.min(x, axis=0) - pad
        upper = np.max(x, axis=0) + pad
        radius = float(np.max(np.linalg.norm(x, axis=1)))
        return cls(lower=lower.astype(float), upper=upper.astype(float), radius=radius)


@dataclass
class FitResult:
    algorithm: str
    centers: Array
    elapsed_s: float
    extra: Dict[str, Any]


@dataclass
class BenchmarkRow:
    algorithm: str
    run: int
    n: int
    d: int
    k: int
    epsilon: float
    delta: float
    radius: float
    seed: int
    elapsed_s: float
    kmeans_loss_sum: float
    kmeans_loss_avg: float
    kmedians_loss_sum: float
    kmedians_loss_avg: float
    ari: Optional[float] = None
    nmi: Optional[float] = None
    status: str = "ok"
    error: str = ""
    extra: str = ""
    rho: Optional[float] = None

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


class Timer:
    def __enter__(self) -> "Timer":
        self.start = time.perf_counter()
        self.elapsed = 0.0
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.elapsed = time.perf_counter() - self.start


def ensure_dir(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path


def as_2d_float(x: Array) -> Array:
    x = np.asarray(x, dtype=float)
    if x.ndim != 2:
        raise ValueError(f"expected a 2D array, got shape {x.shape}")
    if not np.all(np.isfinite(x)):
        raise ValueError("input contains non-finite values")
    return x


def resolve_repo_path(root: Path, name: str, env_name: str) -> Path:
    import os

    if os.environ.get(env_name):
        return Path(os.environ[env_name]).expanduser().resolve()
    return (root / "third_party" / name).resolve()
