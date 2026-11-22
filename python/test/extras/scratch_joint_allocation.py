"""Scratch: privacy-aware allocation between joint Gram and cross releases.

Run:
    python python/test/extras/scratch_joint_allocation.py

Only NumPy is required. This is a statistical ablation, NOT an OpenDP
measurement. The default public estimator remains the equal-weight mechanism.
All Gaussian experiments use fixed-size substitution (two row edits), bounds
R=Y=1, and total privacy rho=1 unless explicitly overridden.
"""

import math

import numpy as np


def normalized_gaussian_sensitivity(cross_weight):
    """Exact one-row replacement sensitivity of the weighted joint vector.

    Statistics are svec(Gram)/(sqrt(2)*R**2) and
    cross_weight*(X.T @ y)/(2*R*Y). For d >= 2, this bound is sharp.
    """
    w = float(cross_weight)
    if w <= 0 or not math.isfinite(w):
        raise ValueError("cross_weight must be finite and positive")
    return 1 + w * w / 4 if w <= 2 else w


def gaussian_scales(cross_weight, rho=1.0, R=1.0, Y=1.0):
    """Smallest common-isotropic Gaussian noise scale at the zCDP budget."""
    w = float(cross_weight)
    sensitivity = normalized_gaussian_sensitivity(w)
    sigma = sensitivity / math.sqrt(2 * rho)
    return sigma * math.sqrt(2) * R**2, sigma * 2 * R * Y / w


def linearized_optimal_weight(d, reference_theta_norm, R=1.0, Y=1.0):
    """Optimal fixed-size Gaussian allocation under isotropic linearization.

    The reference coefficient norm is PUBLIC, not derived from private data.
    The result is not uniformly optimal over all data sets.
    """
    A = (d + 1) / 2 * R**4 * reference_theta_norm**2
    B = d * R**2 * Y**2
    if A == 0:
        return 2.0
    r = 2 * B / (math.sqrt(B * B / 4 + 4 * A * B) + B / 2)
    # r must solve A*r*r+B*r/2-B=0. The stable form above gives
    # 2B/(sqrt(B^2/4+4AB)+B/2), equivalent to the positive root.
    return math.sqrt(2 * min(r, 2.0))


def laplace_cross_weight(d, reference_theta_norm, R=1.0, Y=1.0):
    """Optimal add/remove Laplace split under isotropic linearized MSE."""
    if reference_theta_norm == 0:
        return math.inf
    m = d * (d + 1) / 2
    gram_unit = R**2 * math.sqrt(m)
    cross_unit = R * Y * math.sqrt(d)
    A = (d + 1) * gram_unit**2 * reference_theta_norm**2
    B = 2 * d * cross_unit**2
    return (B / A) ** (1.0 / 3.0)


def gaussian_spectral_bound(d, alpha, sigma):
    return sigma * min(
        math.sqrt(2 * d) + math.sqrt(-2 * math.log(alpha)),
        math.sqrt((d + 1) * (math.log(d) - math.log(alpha))),
    )


def benchmark_joint_allocation(
    *, d=10, k=30, theta_norm=1.0, rho=1.0, trials=1500,
    condition_number=1.0, seed=20261121,
):
    """Compare allocations using the same Gaussian draws across candidates.

    A diagonal population Gram has eigenvalues from k to k*condition_number.
    The fitted coefficient vector is theta_norm/sqrt(d) in each coordinate.
    No clipping, y residuals, or private hyperparameter fitting is modeled.
    """
    rng = np.random.default_rng(seed)
    Z = rng.standard_normal((trials, d, d))
    symmetric_noise = (Z + np.swapaxes(Z, 1, 2)) / 2
    cross_noise = rng.standard_normal((trials, d))
    eigvals = np.linspace(k, k * condition_number, d)
    G = np.diag(eigvals)
    true_theta = np.full(d, theta_norm / math.sqrt(d))
    true_b = G @ true_theta
    weights = {
        "current": 1.0,
        "equal_cross_to_gram_sd": math.sqrt(2),
        "public_reference_optimal": linearized_optimal_weight(
            d, theta_norm
        ),
    }
    loss = {}
    for name, weight in weights.items():
        sg, sb = gaussian_scales(weight, rho)
        H = G[None, :, :] + sg * symmetric_noise
        eigenvalues, Q = np.linalg.eigh(H)
        margin = gaussian_spectral_bound(d, 0.05, sg)
        noisy_b = true_b[None, :] + sb * cross_noise
        rotated = np.einsum("nij,ni->nj", Q, noisy_b)
        fitted = np.einsum(
            "nij,nj->ni", Q, rotated / np.maximum(eigenvalues, margin)
        )
        loss[name] = float(
            np.mean(np.sum((fitted - true_theta)**2, axis=1))
        )
    return weights, loss


if __name__ == "__main__":
    for k in (3, 10, 30, 100):
        for theta_norm in (0, 0.5, 1, 2):
            weights, losses = benchmark_joint_allocation(
                k=k, theta_norm=theta_norm
            )
            print(
                f"k={k:3} ||theta||={theta_norm:3.1f}",
                "cross weights",
                {name: round(w, 3) for name, w in weights.items()},
                "MSE",
                {name: round(v, 5) for name, v in losses.items()},
            )
