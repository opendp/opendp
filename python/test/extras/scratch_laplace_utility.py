"""Scratch: fixed-size pure-DP Laplace sensitivity before versus after.

Run directly:
    OPENBLAS_NUM_THREADS=1 python python/test/extras/scratch_laplace_utility.py

Needs only NumPy and the local scratch_bounded_regression_utility.py.
Both variants use the same independent Laplace draws and the same
epsilon and clipping assumptions. They differ only in the proven joint
L1 sensitivity factor 2 versus 5*sqrt(2)/4.
"""

import math

import numpy as np

from scratch_bounded_regression_utility import make_dataset


def evaluate(X, y, *, epsilon=8.0, trials=800, seed=124):
    n, d = X.shape
    G, cross = X.T @ X, X.T @ y
    reference = np.linalg.lstsq(X, y, rcond=None)[0]
    rng = np.random.default_rng(seed)
    upper = np.triu_indices(d)
    offdiag = upper[0] != upper[1]
    m = d * (d + 1) // 2
    gram_noise = rng.laplace(size=(trials, m))
    cross_noise = rng.laplace(size=(trials, d))
    result = {}
    for name, step in (
        ("previous", 2.0),
        ("improved", 5.0 * math.sqrt(2.0) / 4.0),
    ):
        # R=Y=1, fixed-size substitution; Gram is an svec release.
        gram_scale = step * math.sqrt(2.0 * m) / epsilon
        cross_scale = step * 2.0 * math.sqrt(d) / epsilon
        noise = gram_noise * gram_scale
        noise[:, offdiag] /= math.sqrt(2.0)
        E = np.zeros((trials, d, d))
        E[:, upper[0], upper[1]] = noise
        E[:, upper[1], upper[0]] = noise
        noisy_gram = G[None, :, :] + E
        t = math.log(d / 0.05)
        u = math.sqrt(2.0 * t / (d + 2.0 * t))
        gram_tail = gram_scale * min(
            (
                t - math.log1p(-u * u)
                - (d - 1) * math.log1p(-u * u / 2.0)
            ) / u,
            d * math.log(m / 0.05),
        )
        eigvals, Q = np.linalg.eigh(noisy_gram)
        noisy_cross = cross[None, :] + cross_scale * cross_noise
        rotated = np.einsum("nij,ni->nj", Q, noisy_cross)
        fitted = np.einsum(
            "nij,nj->ni", Q, rotated / np.maximum(eigvals, gram_tail)
        )
        error = fitted - reference
        result[name] = (
            np.mean(np.sum(error * error, axis=1)),
            np.mean(np.einsum("ni,ij,nj->n", error, G, error)) / n,
        )
    return result


if __name__ == "__main__":
    for epsilon in (4.0, 8.0, 16.0):
        experiments = []
        for d in (5, 10):
            for n in (100, 300, 1000):
                for shape in (
                    "isotropic", "moderate", "collinear", "low_rank"
                ):
                    for theta_norm in (0.5, 1.0, 2.0):
                        X, y = make_dataset(n, d, shape, theta_norm)
                        scores = evaluate(X, y, epsilon=epsilon)
                        experiments.append([
                            100.0 * (
                                1.0 - scores["improved"][j] /
                                scores["previous"][j]
                            )
                            for j in (0, 1)
                        ])
        experiments = np.asarray(experiments)
        for j, label in enumerate(("coefficient MSE", "prediction loss")):
            gain = experiments[:, j]
            print(
                "epsilon", epsilon, label,
                "median improvement", round(float(np.median(gain)), 1),
                "worst", round(float(np.min(gain)), 1),
                "wins", int(np.sum(gain > 0)), "/", len(gain),
            )
