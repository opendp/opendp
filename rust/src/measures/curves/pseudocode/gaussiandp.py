"""Reference formulas for the Gaussian differential privacy curve."""

from math import exp, sqrt


def delta_via_gaussianDP(mu: float, epsilon: float, normal_cdf) -> float:
    if mu == 0.0:
        return 0.0
    return normal_cdf(-epsilon / mu + mu / 2) - exp(epsilon) * normal_cdf(
        -epsilon / mu - mu / 2
    )


def beta_via_gaussianDP(mu: float, alpha: float, normal_cdf, normal_ppf) -> float:
    if alpha == 0.0:
        return 1.0
    if alpha == 1.0:
        return 0.0
    return normal_cdf(normal_ppf(1.0 - alpha) - mu / sqrt(1.0))


def compose_gaussianDP(mus: list[float]) -> float:
    return sqrt(sum(mu * mu for mu in mus))
