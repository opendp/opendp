"""Paper Algorithm 3 versus OpenDP's two-statistic Gaussian variant.

Standalone: OPENBLAS_NUM_THREADS=1 python benchmark_vs_adassp_revised.py
Requires NumPy and scikit-learn, but not a compiled OpenDP runtime.

The preprocessing below is a *public benchmark fixture*, NOT a
differentially private operation on confidential data. Both mechanisms
receive the same bounded data (R=Y=1), add/remove adjacency, no intercept.
OpenDP uses native zCDP converted to the same (epsilon, delta) budget.
"""
import math

import numpy as np
from sklearn.datasets import load_diabetes, load_linnerud


def normalize(X, y):
    X = np.asarray(X, dtype=float).copy()
    y = np.asarray(y, dtype=float).copy()
    X -= X.mean(axis=0)
    y -= y.mean()
    X /= max(np.linalg.norm(X, axis=1).max(), 1e-12)
    y /= max(np.abs(y).max(), 1e-12)
    return X, y


def experiment(X, y, *, eps=2.0, delta=1e-6, alpha=0.05,
               reps=300, seed=10):
    X, y = normalize(X, y)
    n, d = X.shape
    gram, cross = X.T @ X, X.T @ y
    reference = np.linalg.lstsq(X, y, rcond=None)[0]
    rng = np.random.default_rng(seed)

    L = math.log(1 / delta)
    rho = (math.sqrt(L + eps) - math.sqrt(L))**2
    sigma = 1 / math.sqrt(rho)
    margin = sigma * min(
        math.sqrt(2 * d) + math.sqrt(-2 * math.log(alpha)),
        math.sqrt((d + 1) * math.log(d / alpha)),
    )

    paper_log = math.log(3.75 / delta)
    paper_sigma = math.sqrt(2 * paper_log) / (eps / 3)
    paper_offset = 2 * paper_log / (eps / 3)
    paper_margin = (
        (math.sqrt(d) + math.sqrt(math.log(1 / alpha)))
        * math.sqrt(32 * paper_log) / (eps / 3)
    )
    losses = {"paper": [], "joint": []}
    smallest_eigen = np.linalg.eigvalsh(gram)[0]
    for _ in range(reps):
        Z = rng.standard_normal((d, d))
        cross_noise = rng.standard_normal(d)

        # Algorithm 3 (Aryanfard et al., 2026): three independent releases.
        Ep = paper_sigma * np.triu(Z)
        Ep = Ep + np.triu(Ep, 1).T
        noisy_min = max(
            smallest_eigen + paper_sigma * rng.standard_normal()
            - paper_offset, 0,
        )
        ridge = max(paper_margin - noisy_min, 0)
        paper_matrix = gram + Ep + ridge * np.eye(d)
        try:
            theta_p = np.linalg.solve(
                paper_matrix, cross + paper_sigma * cross_noise
            )
        except np.linalg.LinAlgError:
            theta_p = np.linalg.lstsq(
                paper_matrix, cross + paper_sigma * cross_noise, rcond=None
            )[0]

        # Current two-statistic joint add/remove Gaussian release.
        Ej = sigma * (Z + Z.T) / 2
        eigenvalues, Q = np.linalg.eigh(gram + Ej)
        theta_j = Q @ (
            (Q.T @ (cross + sigma * cross_noise))
            / np.maximum(eigenvalues, margin)
        )
        for name, theta in (("paper", theta_p), ("joint", theta_j)):
            error = theta - reference
            losses[name].append(float(error @ gram @ error / n))
    return n, d, {name: float(np.mean(v)) for name, v in losses.items()}


def synthetic(n, d, shape):
    rng = np.random.default_rng(53 + d + n)
    V, _ = np.linalg.qr(rng.normal(size=(d, d)))
    scales = {
        "isotropic": np.ones(d),
        "moderate": np.geomspace(1, 0.3, d),
        "collinear": np.geomspace(1, 0.025, d),
    }[shape]
    X = rng.normal(size=(n, d)) @ (V * scales).T
    theta = rng.normal(size=d) / math.sqrt(d)
    return X, X @ theta + 0.3 * rng.normal(size=n)


if __name__ == "__main__":
    diabetes = load_diabetes()
    linnerud = load_linnerud()
    datasets = [
        ("diabetes", diabetes.data, diabetes.target),
        ("linnerud", linnerud.data, linnerud.target[:, 0]),
    ]
    datasets += [
        (f"{shape}-n{n}-d{d}", *synthetic(n, d, shape))
        for shape in ("isotropic", "moderate", "collinear")
        for n in (100, 500) for d in (5, 10)
    ]
    for eps in (1.0, 4.0, 16.0):
        print(f"epsilon={eps}")
        for name, X, y in datasets:
            n, d, result = experiment(X, y, eps=eps)
            print(name, n, d, result,
                  "ratio", result["paper"] / max(result["joint"], 1e-15))
