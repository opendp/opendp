from __future__ import annotations

from typing import Dict, Optional
import numpy as np


def _min_distances(x: np.ndarray, centers: np.ndarray, *, squared: bool) -> np.ndarray:
    x = np.asarray(x, dtype=float)
    centers = np.asarray(centers, dtype=float)
    if centers.ndim != 2 or centers.shape[0] == 0:
        raise ValueError("centers must have shape (k, d) with k > 0")
    # Chunk to avoid building huge n*k*d arrays on larger runs.
    out = np.empty(x.shape[0], dtype=float)
    chunk = max(1, int(5_000_000 / max(centers.shape[0], 1)))
    for start in range(0, x.shape[0], chunk):
        stop = min(start + chunk, x.shape[0])
        diff = x[start:stop, None, :] - centers[None, :, :]
        dsq = np.sum(diff * diff, axis=2)
        best = np.min(dsq, axis=1)
        out[start:stop] = best if squared else np.sqrt(best)
    return out


def labels_from_centers(x: np.ndarray, centers: np.ndarray) -> np.ndarray:
    x = np.asarray(x, dtype=float)
    centers = np.asarray(centers, dtype=float)
    labels = np.empty(x.shape[0], dtype=int)
    chunk = max(1, int(5_000_000 / max(centers.shape[0], 1)))
    for start in range(0, x.shape[0], chunk):
        stop = min(start + chunk, x.shape[0])
        diff = x[start:stop, None, :] - centers[None, :, :]
        dsq = np.sum(diff * diff, axis=2)
        labels[start:stop] = np.argmin(dsq, axis=1)
    return labels


def kmeans_loss(x: np.ndarray, centers: np.ndarray) -> float:
    return float(np.sum(_min_distances(x, centers, squared=True)))


def kmedians_loss(x: np.ndarray, centers: np.ndarray) -> float:
    return float(np.sum(_min_distances(x, centers, squared=False)))


def optional_label_metrics(true_labels: Optional[np.ndarray], predicted_labels: np.ndarray) -> Dict[str, Optional[float]]:
    if true_labels is None:
        return {"ari": None, "nmi": None}
    try:
        from sklearn.metrics import adjusted_rand_score, normalized_mutual_info_score
    except Exception:
        return {"ari": None, "nmi": None}
    return {
        "ari": float(adjusted_rand_score(true_labels, predicted_labels)),
        "nmi": float(normalized_mutual_info_score(true_labels, predicted_labels)),
    }
