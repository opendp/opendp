"""Compare postprocessing and noise allocation on clipped synthetic datasets.

Run from the repository root:
    OPENBLAS_NUM_THREADS=1 python \
        python/test/extras/scratch_bounded_regression_utility.py

No OpenDP dependency. This models one fixed-size row substitution, L2 row
radius R=1, label radius Y=1, and Gaussian/zCDP allowance rho=1.
Noise draws are paired between variants. No private hyperparameter selection
is performed. Both OLS coefficient MSE and empirical excess prediction loss
are reported, since the latter discounts nearly unidentifiable directions.
"""

import math

import numpy as np


def make_dataset(n, d, shape, theta_norm, seed=11):
    rng = np.random.default_rng(seed)
    if shape == "isotropic":
        scales = np.ones(d)
    elif shape == "moderate":
        scales = np.geomspace(1, 0.2, d)
    elif shape == "collinear":
        scales = np.geomspace(1, 0.01, d)
    elif shape == "low_rank":
        scales = np.array(
            [1.0] * max(1, d // 4)
            + [0.02] * (d - max(1, d // 4))
        )
    else:
        raise ValueError(shape)
    orientation, _ = np.linalg.qr(rng.normal(size=(d, d)))
    X = rng.normal(size=(n, d)) @ (orientation * scales).T
    X /= np.maximum(np.linalg.norm(X, axis=1, keepdims=True), 1.0)
    theta = rng.normal(size=d)
    theta *= theta_norm / np.linalg.norm(theta)
    y = np.clip(X @ theta + 0.1 * rng.normal(size=n), -1.0, 1.0)
    return X, y


def james_stein(H, gram_sigma):
    """Positive-part shrinkage of the *traceless* Gaussian Gram noise."""
    count, d, _ = H.shape
    mean = np.trace(H, axis1=1, axis2=2) / d
    residual = H - mean[:, None, None] * np.eye(d)
    lengths = np.sum(residual * residual, axis=(1, 2))
    m = d * (d + 1) // 2
    factor = np.maximum(
        0.0, 1.0 - (m - 3) * gram_sigma**2 / np.maximum(lengths, 1e-12)
    )
    return mean[:, None, None] * np.eye(d) + factor[:, None, None] * residual


def evaluate(X, y, trials=500, seed=71):
    rng = np.random.default_rng(seed)
    n, d = X.shape
    G, cross = X.T @ X, X.T @ y
    reference = np.linalg.lstsq(X, y, rcond=None)[0]
    Z = rng.normal(size=(trials, d, d))
    Z = (Z + Z.transpose(0, 2, 1)) / 2.0
    eta = rng.normal(size=(trials, d))
    alpha = 0.05
    choices = {
        "equal": (1.0, False, 1.0),
        "cross_1.1": (1.1, False, 1.0),
        "cross_1.2": (1.2, False, 1.0),
        "cross_sqrt2": (math.sqrt(2.0), False, 1.0),
        "james_stein": (1.0, True, 1.0),
        "cross_1.1_js": (1.1, True, 1.0),
        "floor_1.5": (1.0, False, 1.5),
    }
    results = {}
    for name, (weight, shrink, floor_factor) in choices.items():
        # Exact normalized joint substitution sensitivity for d >= 2.
        S = 1.0 + weight * weight / 4.0
        gram_sigma = S  # R=Y=rho=1
        cross_sigma = math.sqrt(2.0) * S / weight
        H = G[None, :, :] + gram_sigma * Z
        if shrink:
            H = james_stein(H, gram_sigma)
        noisy_cross = cross[None, :] + cross_sigma * eta
        eigvals, Q = np.linalg.eigh(H)
        margin = gram_sigma * min(
            math.sqrt(2.0 * d) + math.sqrt(-2.0 * math.log(alpha)),
            math.sqrt((d + 1) * math.log(d / alpha)),
        )
        projected = np.einsum("nij,ni->nj", Q, noisy_cross)
        fitted = np.einsum(
            "nij,nj->ni",
            Q,
            projected / np.maximum(eigvals, floor_factor * margin),
        )
        error = fitted - reference
        coef_mse = np.mean(np.sum(error * error, axis=1))
        excess = np.mean(np.einsum("ni,ij,nj->n", error, G, error)) / n
        results[name] = (coef_mse, excess)
    return results


if __name__ == "__main__":
    scenarios = []
    for d in (5, 10):
        for n in (100, 300, 1000):
            for shape in ("isotropic", "moderate", "collinear", "low_rank"):
                for theta_norm in (0.5, 1.0, 2.0):
                    scenarios.append(
                        evaluate(*make_dataset(n, d, shape, theta_norm))
                    )

    for metric, label in enumerate(("coefficient MSE", "prediction loss")):
        print(label, "from", len(scenarios), "matched-noise scenarios")
        for name in scenarios[0]:
            if name == "equal":
                continue
            improvements = np.array([
                100.0 * (1.0 - scores[name][metric] / scores["equal"][metric])
                for scores in scenarios
            ])
            print(
                name,
                "median", round(float(np.median(improvements)), 1),
                "mean", round(float(np.mean(improvements)), 1),
                "worst", round(float(np.min(improvements)), 1),
                "wins", int(np.sum(improvements > 0)),
            )
