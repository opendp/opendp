"""Numerical core for objective-perturbed logistic regression (idealized arithmetic).

A first-order residual <= tol and strong convexity ridge imply distance to the
exact objective-perturbed minimizer <= tol/ridge. Independent spherical Laplace
repair noise of scale 2*tol/(ridge*repair_epsilon) covers two density-ratio
comparisons with that exact mechanism. The Gaussian/Laplace RNG and optimizer
here are NOT hardened against finite-precision or side-channel attacks.
"""
import numpy as np
from scipy.optimize import minimize
from scipy.special import expit


def clip_training_data(data, feature_norm, fit_intercept):
    x = np.asarray(data.X, dtype=float).copy()
    if x.ndim != 2:
        raise ValueError("X must be a 2D array")
    x = np.nan_to_num(x, nan=0.0, posinf=0.0, neginf=0.0)
    lengths = np.linalg.norm(x, axis=1)
    x *= np.minimum(1, feature_norm / np.maximum(lengths, 1e-300))[:, None]
    y = (np.asarray(data.y).reshape(-1) >= 0.5).astype(float)
    if fit_intercept:
        x = np.column_stack((x, np.ones(len(x))))
    return x, y


def sample_spherical_laplace(dim, scale, rng=None):
    if rng is None:
        rng = np.random.default_rng()
    direction = rng.standard_normal(dim)
    direction /= np.linalg.norm(direction)
    return direction * rng.gamma(shape=dim, scale=scale)


def logistic_objective_and_gradient(theta, x, y, ridge, linear):
    margins = x @ theta
    objective = (
        np.logaddexp(0.0, margins).sum()
        - np.dot(y, margins)
        + ridge * np.dot(theta, theta) / 2
        + np.dot(linear, theta)
    )
    gradient = x.T @ (expit(margins) - y) + ridge * theta + linear
    return float(objective), gradient


def optimize_perturbed_logistic(x, y, ridge, linear, tolerance, max_iter):
    """Return a certified residual, not merely an optimizer success flag.

    Exact-arithmetic smooth strongly-convex gradient descent eventually
    meets the certificate. No dataset-dependent failure status is released;
    execution time and finite-precision behavior are outside this prototype's
    formal privacy model.
    """
    dim = x.shape[1]
    f = lambda w: logistic_objective_and_gradient(w, x, y, ridge, linear)
    result = minimize(
        f, np.zeros(dim), method="L-BFGS-B", jac=True,
        options={"maxiter": max_iter, "ftol": 1e-15, "gtol": tolerance / 10},
    )
    theta = np.asarray(result.x, dtype=float)
    if np.linalg.norm(f(theta)[1]) <= tolerance:
        return theta

    # Fast Newton continuation. A final globally convergent gradient fallback
    # ensures the release is never conditioned on a data-dependent solver flag.
    for _ in range(40):
        cost, grad = f(theta)
        if np.linalg.norm(grad) <= tolerance:
            return theta
        p = expit(x @ theta)
        h = (x.T * (p * (1 - p))) @ x
        h.flat[::dim + 1] += ridge
        step = np.linalg.solve(h, grad)
        amount = 1.0
        while amount > 2**-40 and f(theta - amount * step)[0] > cost:
            amount *= 0.5
        theta -= amount * step

    smoothness = ridge + np.sum(np.einsum("ij,ij->i", x, x)) / 4
    while True:
        grad = f(theta)[1]
        if np.linalg.norm(grad) <= tolerance:
            return theta
        theta -= grad / smoothness


def perturb_and_fit(data, *, feature_norm, fit_intercept, ridge,
                    noise_scale, repair_scale, tolerance, max_iter):
    x, y = clip_training_data(data, feature_norm, fit_intercept)
    b = sample_spherical_laplace(x.shape[1], noise_scale)
    theta = optimize_perturbed_logistic(x, y, ridge, b, tolerance, max_iter)
    return theta + sample_spherical_laplace(x.shape[1], repair_scale)
