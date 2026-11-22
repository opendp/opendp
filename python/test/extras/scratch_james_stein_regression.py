"""Scratch: zero-budget James-Stein denoising of a Gaussian noisy Gram.

Run:
    python python/test/extras/scratch_james_stein_regression.py

For svec Gaussian noise, the m=d(d+1)/2 independent upper-triangular
coordinates form an isotropic Gaussian vector. The trace direction is one
Gaussian coordinate, and the traceless part has dimension m-1. For d>=3,
positive-part James-Stein shrinkage of the traceless part uniformly reduces
expected squared Frobenius error of the Gram matrix.

This does NOT guarantee lower regression coefficient error. This experiment
is deliberately not wired into sklearn's public estimator.
"""

import math

import numpy as np


def shrink_traceless(H, sigma):
    """Denoise a symmetric matrix under iid svec Gaussian noise."""
    d = H.shape[0]
    m = d * (d + 1) // 2
    if d < 3:
        return H.copy()
    avg = np.trace(H) / d
    residual = H - np.eye(d) * avg
    squared = np.sum(residual * residual)
    if squared == 0:
        return H.copy()
    weight = max(0.0, 1.0 - (m - 3) * sigma**2 / squared)
    return np.eye(d) * avg + weight * residual


def coefficient_mse(
    eigenvalues, theta, *, sigma_gram=1.25,
    sigma_cross=1.25 * math.sqrt(2), trials=900, seed=343,
):
    d = len(eigenvalues)
    G = np.diag(eigenvalues)
    b = G @ theta
    rng = np.random.default_rng(seed)
    alpha = 0.05
    B = sigma_gram * min(
        math.sqrt(2 * d) + math.sqrt(-2 * math.log(alpha)),
        math.sqrt((d + 1) * math.log(d / alpha)),
    )
    coeff_loss = np.zeros(2)
    matrix_loss = np.zeros(2)
    for _ in range(trials):
        Z = rng.normal(size=(d, d))
        noisy_gram = G + sigma_gram * (Z + Z.T) / 2
        noisy_cross = b + rng.normal(size=d) * sigma_cross
        for i, M in enumerate((noisy_gram,
                               shrink_traceless(noisy_gram, sigma_gram))):
            eigvals, Q = np.linalg.eigh(M)
            estimate = Q @ (
                (Q.T @ noisy_cross) / np.maximum(eigvals, B)
            )
            coeff_loss[i] += np.linalg.norm(estimate - theta)**2 / trials
            matrix_loss[i] += np.linalg.norm(M - G, "fro")**2 / trials
    return coeff_loss, matrix_loss


def check_scalar_and_trace_properties():
    rng = np.random.default_rng(109)
    for d in (1, 2, 3, 10):
        H = rng.normal(size=(d, d))
        H = (H + H.T) / 2
        out = shrink_traceless(H, 1)
        assert np.allclose(out, out.T)
        assert np.isclose(np.trace(out), np.trace(H))


if __name__ == "__main__":
    check_scalar_and_trace_properties()
    d = 10
    for k in (3, 10, 30, 100):
        for condition in (1, 3, 10, 100):
            spectrum = np.linspace(k, k * condition, d)
            theta = np.ones(d) / math.sqrt(d)
            coeff, gram = coefficient_mse(spectrum, theta)
            print(
                f"k={k:3} condition={condition:3}",
                "coefficient MSE", np.round(coeff, 5),
                "improvement", round(100 * (1 - coeff[1] / coeff[0]), 1),
                "Gram improvement", round(100 * (1 - gram[1] / gram[0]), 1),
            )
