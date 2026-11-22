"""Scratch utility experiments for two-release private least squares.

Standalone: python python/test/extras/scratch_linear_regression_utility.py
These are *not* correctness proofs or published risk guarantees. The sample
calibration uses add/remove adjacency, R=Y=1, and no intercept unless stated.
No OpenDP
dependency is needed: only NumPy. Values depend on public reference parameters.
"""

import math

import numpy as np


def spectral_margin(d, alpha, scale, *, gaussian=True, weight=math.sqrt(2)):
    """One-sided matrix-noise tail, for a weighted upper-triangle release."""
    t = math.log(d) - math.log(alpha)
    if gaussian:
        assert weight == math.sqrt(2)
        return scale * min(
            math.sqrt(2 * d) + math.sqrt(-2 * math.log(alpha)),
            math.sqrt((d + 1) * t),
        )
    # For off-diagonal weight w, the log-mgf is
    # -log(1-u^2) -(d-1)log(1-u^2/w^2), with u < min(1,w).
    # An analytic public choice of u is close to the optimal Chernoff
    # parameter for the tested dimensions.
    variance_factor = 1 + (d - 1) / weight**2
    u = min(1, weight) * math.sqrt(
        2 * t / (2 * variance_factor - 1 + 2 * t)
    )
    u = min(u, math.nextafter(min(1, weight), 0))
    matrix = scale * (
        t - math.log1p(-u*u)
        - (d - 1) * math.log1p(-(u/weight)**2)
    ) / u
    entries = d * (d + 1) // 2
    entrywise = d * scale * max(1, 1 / weight) * (
        math.log(entries) - math.log(alpha)
    )
    return min(matrix, entrywise)


def analytic_gram_fraction(d, *, gaussian, reference_theta=1.0, R=1.0, Y=1.0):
    """First-order isotropic coefficient-MSE optimum, not a universal optimum.

    reference_theta is a PUBLIC reference for ||theta||, never estimated from
    private data. Do not use as a data-dependent budget split.
    """
    if gaussian:
        a = (d + 1) / 2 * reference_theta**2 * R**4
        b = d * R**2 * Y**2
        return math.sqrt(a) / (math.sqrt(a) + math.sqrt(b))
    m = d * (d + 1) // 2
    a = (d + 1) * reference_theta**2 * (R**4 * m)
    b = 2 * d * (R**2 * Y**2 * d)
    return a**(1 / 3) / (a**(1 / 3) + b**(1 / 3))


def tentative_joint_substitution_cost(*, gram_sd, cross_sd, R=1, Y=1):
    """Candidate exact Gaussian RDP sensitivity for one fixed-size replacement.

    Assuming the maxima occur at ||x||=||x'||=R and |y|=|y'|=Y, optimize
    over |cos(x,x')|. Needs proof over arbitrary row norms before certification.
    Returns rho for the pair (svec(Delta G), Delta b).
    """
    a = R**4 / (2 * gram_sd**2)
    b = R**2 * Y**2 / (2 * cross_sd**2)
    inner_product = min(b / (2 * a), 1.0)
    return 2 * a * (1 - inner_product**2) + 2 * b * (1 + inner_product)


def slack_column(X, radius):
    """Max out every row L2 norm without altering original Gram entries."""
    X = np.asarray(X, dtype=float)
    remaining = np.maximum(radius**2 - np.sum(X**2, axis=1), 0)
    return np.column_stack([X, np.sqrt(remaining)])


def sample_noise(rng, d, *, gaussian, privacy, fraction, weight=math.sqrt(2)):
    """Sample the same joint noise geometry as two independent DP releases."""
    if gaussian:
        sg = 1 / math.sqrt(2 * privacy * fraction)
        sb = 1 / math.sqrt(2 * privacy * (1 - fraction))
        Z = rng.normal(0, sg, size=(d, d))
        E = (Z + Z.T) / 2
        eta = rng.normal(0, sb, size=d)
    else:
        m = d * (d + 1) // 2
        sensitivity_gram = math.sqrt(d + d * (d - 1) * weight**2 / 4)
        sg = sensitivity_gram / (privacy * fraction)
        sb = math.sqrt(d) / (privacy * (1 - fraction))
        E = np.zeros((d, d))
        upper = np.triu_indices(d)
        values = rng.laplace(0, sg, m)
        values[upper[0] != upper[1]] /= weight
        E[upper] = values
        E[(upper[1], upper[0])] = values
        eta = rng.laplace(0, sb, d)
    return E, eta, sg


def coefficient_error(rng, eigvals, theta, *, gaussian, privacy, fraction,
                      floor, weight=math.sqrt(2), alpha=0.05):
    G = np.diag(eigvals)
    E, eta, sg = sample_noise(rng, len(theta), gaussian=gaussian,
                               privacy=privacy, fraction=fraction, weight=weight)
    noisy_gram = G + E
    eigenvalues, Q = np.linalg.eigh(noisy_gram)
    margin = spectral_margin(len(theta), alpha, sg,
                             gaussian=gaussian, weight=weight)
    adjusted = np.maximum(eigenvalues, margin) if floor else (
        eigenvalues + max(margin - eigenvalues[0], 0)
    )
    pred = Q @ ((Q.T @ (G @ theta + eta)) / adjusted)
    return float(np.sum((pred - theta)**2))


def benchmark(*, d=10, min_eigen=30, max_eigen=30, gaussian=True,
              privacy=1.0, trials=400, seed=20261121):
    """Isotropic/ill-conditioned ablations with matched privacy allowances."""
    rng = np.random.default_rng(seed)
    seeds = rng.integers(0, 2**32, size=trials)
    eigvals = np.linspace(min_eigen, max_eigen, d)
    theta = np.ones(d) / math.sqrt(d)
    analytic = analytic_gram_fraction(d, gaussian=gaussian)
    configs = [
        ("ridge/equal", False, 0.5, math.sqrt(2)),
        ("floor/equal", True, 0.5, math.sqrt(2)),
        ("ridge/analytic", False, analytic, math.sqrt(2)),
        ("floor/analytic", True, analytic, math.sqrt(2)),
    ]
    if not gaussian:
        configs += [("floor/triu", True, analytic, 1.0)]
    out = {}
    for name, floor, frac, weight in configs:
        out[name] = float(np.mean([
            coefficient_error(np.random.default_rng(int(s)), eigvals, theta,
                              gaussian=gaussian, privacy=privacy, fraction=frac,
                              floor=floor, weight=weight)
            for s in seeds
        ]))
    return out



def benchmark_joint_substitution(*, d=10, k=30, rho=1.0,
                                 trials=500, seed=20261121):
    """Compare normalized joint Gaussian 5/4 map with composition."""
    theta = np.ones(d) / math.sqrt(d)
    G = np.eye(d) * k
    margin_multiplier = (
        math.sqrt(2 * d) + math.sqrt(-2 * math.log(.05))
    )
    result = np.zeros(2)
    rng = np.random.default_rng(seed)
    for _ in range(trials):
        Z = rng.normal(size=(d, d))
        eta = rng.normal(size=d)
        for i, reduction in enumerate((1.0, 5.0 / (4.0 * math.sqrt(2)))):
            # Normalized per-replacement sensitivities are sqrt(2) for
            # svec(Gram) and 2 for cross-product.
            sigma_g = math.sqrt(2 / rho) * reduction
            sigma_b = 2 / math.sqrt(rho) * reduction
            noisy_gram = G + (Z + Z.T) * sigma_g / 2
            noisy_cross = G @ theta + sigma_b * eta
            eigenvalues, Q = np.linalg.eigh(noisy_gram)
            adjusted = np.maximum(eigenvalues, sigma_g * margin_multiplier)
            estimate = Q @ ((Q.T @ noisy_cross) / adjusted)
            result[i] += np.linalg.norm(estimate - theta)**2 / trials
    return result.tolist()

def coefficient_estimates_with_intercept(noisy_gram, noisy_cross, n, margin):
    """Compare floor-all versus unpenalized-intercept block elimination."""
    eigvals, Q = np.linalg.eigh(noisy_gram)
    floor_all = Q @ (
        (Q.T @ noisy_cross) / np.maximum(eigvals, margin)
    )
    # Schur complement with public n: intercept is unpenalized,
    # but the noisy centered Gram may amplify slope estimation error.
    cross = noisy_gram[:-1, -1]
    center = noisy_gram[:-1, :-1] - np.outer(cross, cross) / n
    values, V = np.linalg.eigh(center)
    rhs = noisy_cross[:-1] - cross * noisy_cross[-1] / n
    slopes = V @ ((V.T @ rhs) / np.maximum(values, margin))
    intercept = (noisy_cross[-1] - cross @ slopes) / n
    return floor_all, np.r_[slopes, intercept]


def benchmark_public_intercept(*, p=10, n=30, trials=300, seed=147):
    """L2 coefficient error; centered features, true intercept 1."""
    rng = np.random.default_rng(seed)
    d = p + 1
    truth = np.r_[np.zeros(p), 1.0]
    gram = np.diag([n / p] * p + [n])
    cross = gram @ truth
    # Add/remove with ||[x,1]||_2 <= sqrt(2), |y| <= 1.
    gram_scale, cross_scale = 2.0, math.sqrt(2.0)
    margin = spectral_margin(d, .05, gram_scale)
    errors = np.zeros(2)
    intercept_errors = np.zeros(2)
    for _ in range(trials):
        Z = rng.normal(0, gram_scale, (d, d))
        noisy_gram = gram + (Z + Z.T) / 2
        noisy_gram[-1, -1] = n
        noisy_cross = cross + rng.normal(0, cross_scale, d)
        estimates = coefficient_estimates_with_intercept(
            noisy_gram, noisy_cross, n, margin
        )
        for i, theta in enumerate(estimates):
            errors[i] += np.linalg.norm(theta - truth)**2 / trials
            intercept_errors[i] += (theta[-1] - 1)**2 / trials
    return errors.tolist(), intercept_errors.tolist()


def augmented_y_ratio(d, *, samples=3000):
    """Linearized Gaussian error from a joint SSCP of [x, r*y].

    Compare optimal r to optimally divided two-release Gaussian budget.
    Assumes R=Y=||theta||=1 and add/remove adjacency.
    """
    A, B = (d + 1) / 2, d
    separate = (math.sqrt(A) + math.sqrt(B))**2 / 2
    ratios = np.linspace(0.2, 3.0, samples)
    augmented = (1 + ratios**2)**2 * (A / 2 + B / (4 * ratios**2))
    return float(np.min(augmented) / separate)


if __name__ == "__main__":
    for gaussian in (True, False):
        for d in (2, 10, 30):
            for k in (5, 30, 100):
                scores = benchmark(
                    d=d, min_eigen=k, gaussian=gaussian,
                    privacy=1 if gaussian else 2,
                )
                kind = "zCDP" if gaussian else "pureDP"
                print(kind, "d=", d, "eigen=", k,
                      {name: round(loss, 5) for name, loss in scores.items()})
    print("Substitution joint-cost candidate, equal budgets (rho=1):",
          tentative_joint_substitution_cost(
              gram_sd=math.sqrt(2), cross_sd=2))
    print("Joint augmented-[x,y] Gram relative MSE (d=10):",
          augmented_y_ratio(10))
    print("Fixed-size Gaussian composition vs joint, d=10, k=30:",
          benchmark_joint_substitution())
    for n in (10, 30, 100):
        total, intercept = benchmark_public_intercept(n=n)
        print("public intercept n=", n,
              "total MSE", total, "intercept MSE", intercept)
