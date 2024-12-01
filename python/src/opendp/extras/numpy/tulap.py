from __future__ import annotations

import math
from typing import Literal
from opendp._lib import import_optional_dependency
from opendp.mod import binary_search


class BinomialTulap:
    """
    Utilities to conduct statistical inference on a realization of the random variable Z = M + T,
    where M ~ Binomial(n=size, p=theta), T ~ Tulap(0, b, q), and b and q are calibrated to satisfy (ε, δ)-DP.

    Equivalently, Z ~ Tulap(m, b, q), where m is a realization of M ~ Binomial(n=size, p=theta),
    and b and q are calibrated to satisfy (ε, δ)-DP.

    A counting query is a realization of M ~ Binomial(n=size, p=theta) when,
    for each row in data with ``size`` rows, the counter is incremented with probability theta.
    This quantity is then privatized by the Tulap mechanism :py:func:`opendp.measurements.make_canonical_noise`,
    a noise perturbation mechanism that privatizes the count by adding a sample from T ~ Tulap(0, b, q).

    :param estimate: a draw from Z, the outcome of the Tulap mechanism on binomial data
    :param epsilon: noise parameter ε for the Tulap distribution
    :param delta: noise parameter δ for the Tulap distribution
    :param size: (approximate) total number of records in the sensitive dataset
    """

    def __init__(
        self, estimate: float, epsilon: float, delta: float, size: int
    ) -> None:
        self.estimate = estimate
        self.epsilon = epsilon
        self.delta = delta
        self.size = size

    def confidence_interval(
        self, alpha: float, side: Literal["lower", "upper"] | None = None
    ) -> float | tuple[float, float]:
        """Compute a confidence interval for ``self.estimate``.

        * side=None: return the two bounds of a confidence interval
        * side="lower": return a one-sided confidence interval lower bound
        * side="upper": return a one-sided confidence interval upper bound

        :param alpha: statistical significance level
        :param side: configure interval type
        :return: confidence interval bound(s)
        """
        if side is None:
            return two_sided_confidence_interval(
                self.estimate, self.epsilon, self.delta, self.size, alpha
            )

        if side in {"lower", "upper"}:
            return one_sided_confidence_interval(
                self.estimate, self.epsilon, self.delta, self.size, alpha, side
            )

        raise ValueError("tail must be None, lower or upper")

    def p_value(
        self, theta: float, tail: Literal["left", "right"] | None = None
    ) -> float:
        """
        Computes a p-value based on ``self.estimate``.

        * tail=None: null hypothesis states that true proportion is theta
        * tail="left": null hypothesis states that true proportion is no less than theta
        * tail="right": null hypothesis states that true proportion is no greater than theta

        :param theta: success rate
        :param tail: configure the null hypothesis
        :return: the probability of observing ``self.estimate``, given the null hypothesis
        """
        if tail is None:
            return two_sided_pvalue(
                self.estimate, self.epsilon, self.delta, self.size, theta
            )

        if tail in {"left", "right"}:
            return one_sided_pvalue(
                self.estimate, self.epsilon, self.delta, self.size, theta, tail
            )

        raise ValueError("tail must be None, left or right")

    def __repr__(self) -> str:
        return f"Tulap(estimate={self.estimate}, epsilon={self.epsilon}, delta={self.delta}, size={self.size})"


def _tulap_cdf(t, m: float = 0, epsilon: float = 0, delta: float = 0):
    """Computes the value of the cumulative density function Pr[T <= t] where T ~ Tulap(m, b, q),
    where b and q are calibrated to satisfy (ε, δ)-DP"""
    np = import_optional_dependency("numpy")

    t = np.atleast_1d(t)
    b = math.exp(-epsilon)
    q = (2 * delta * b) / (1 - b + 2 * delta * b)
    lcut = q / 2
    rcut = q / 2
    t = t - m  # normalize
    r = np.rint(t)
    g = -math.log(b)
    l = math.log(1 + b)  # noqa
    k = 1 - b
    
    negs = np.exp((r * g) - l + np.log(b + ((t - r + (1 / 2)) * k)))
    poss = 1 - np.exp((r * -g) - l + np.log(b + ((r - t + (1 / 2)) * k)))

    # check for infinities
    negs[np.isinf(negs)] = 0
    poss[np.isinf(poss)] = 0
    # truncate w.r.t. the indicator on t's positivity
    is_leq0 = np.less_equal(t, 0).astype(int)
    trunc = (is_leq0 * negs) + ((1 - is_leq0) * poss)

    # handle the cut adjustment and scaling
    q = lcut + rcut
    is_mid = np.logical_and(
        np.less_equal(lcut, trunc), np.less_equal(trunc, 1 - rcut)
    ).astype(int)
    is_rhs = np.less(1 - rcut, trunc).astype(int)
    return ((trunc - lcut) / (1 - q)) * is_mid + is_rhs


def two_sided_confidence_interval(
    Z: float, epsilon: float, delta: float, size: int, alpha: float
) -> tuple[float, float]:
    """Compute a two-sided confidence interval centered on ``Z``.

    :param Z: tulap random variable
    :param epsilon: noise parameter ε for the Tulap distribution
    :param delta: noise parameter δ for the Tulap distribution
    :param size: (approximate) total number of records in the sensitive dataset
    :param alpha: statistical significance level
    :return: confidence interval bounds
    """
    mle = max(min(Z / size, 1.0), 0.0)
    predicate = lambda B: two_sided_pvalue(B * size, epsilon, delta, size, mle) > alpha

    L = binary_search(predicate, bounds=(0.0, mle)) if mle > 0 else 0.0
    U = binary_search(predicate, bounds=(mle, 1.0)) if mle < 1 else 1.0

    return L, U


def one_sided_confidence_interval(
    Z: float,
    epsilon: float,
    delta: float,
    size: int,
    alpha: float,
    side: Literal["lower", "upper"],
) -> float:
    """Compute a one-sided confidence interval for ``self.estimate``.

    * tail="lower": return a one-sided confidence interval lower bound
    * tail="upper": return a one-sided confidence interval upper bound

    :param Z: tulap random variable
    :param epsilon: noise parameter ε for the Tulap distribution
    :param delta: noise parameter δ for the Tulap distribution
    :param size: (approximate) total number of records in the sensitive dataset
    :param alpha: statistical significance level
    :param side: configure interval type
    :return: lower or upper confidence interval bound
    """
    mle = max(min(Z / size, 1.0), 0.0)
    tail = {"lower": "left", "upper": "right"}.get(side)
    if tail is None:
        raise ValueError("tail must be 'lower' or 'upper'")
    pred = lambda B: one_sided_pvalue(B * size, epsilon, delta, size, mle, tail) > alpha  # type: ignore[arg-type]
    return binary_search(pred, bounds=(0.0, 1.0))


def two_sided_pvalue(
    Z: float, epsilon: float, delta: float, size: int, theta: float
) -> float:
    """Computes a p-value for a hypothesis test on the probability of success.

    :param Z: tulap random variable
    :param epsilon: noise parameter ε for the Tulap distribution
    :param delta: noise parameter δ for the Tulap distribution
    :param size: (approximate) number of trials
    :param theta: true probability of binomial distribution
    :return: Probability of observing Z when the true success rate is theta
    """
    T = abs(size * theta - Z)
    left_tail = one_sided_pvalue(
        Z=size * theta + T,
        epsilon=epsilon,
        delta=delta,
        size=size,
        theta=theta,
        tail="right",
    )
    right_tail = one_sided_pvalue(
        Z=size * theta - T,
        epsilon=epsilon,
        delta=delta,
        size=size,
        theta=theta,
        tail="right",
    )

    return left_tail + (1 - right_tail)


def one_sided_pvalue(
    Z: float,
    epsilon: float,
    delta: float,
    size: int,
    theta: float,
    tail: Literal["left", "right"],
) -> float:
    """Computes a p-value for a hypothesis test on the probability of success.

    * tail="left": null hypothesis states that true proportion is no less than theta
    * tail="right": null hypothesis states that true proportion is no greater than theta

    :param Z: tulap random variable
    :param epsilon: noise parameter ε for the Tulap distribution
    :param delta: noise parameter δ for the Tulap distribution
    :param size: (approximate) number of trials
    :param theta: true probability of binomial distribution
    :param tail: either "left" or "right"
    :return: Probability of observing Z when the true success rate is theta
    """
    stats = import_optional_dependency("scipy.stats")
    np = import_optional_dependency("numpy")

    values = np.arange(size)
    B = stats.binom.pmf(k=values, n=size, p=theta)
    if tail == "right":
        F = _tulap_cdf(t=values - Z, epsilon=epsilon, delta=delta)
    elif tail == "left":
        F = 1 - _tulap_cdf(t=values - Z, epsilon=epsilon, delta=delta)
    else:
        raise ValueError("tail must be 'left' or 'right'")
    return np.dot(F.T, B)


def one_sided_uniformly_most_powerful_tests(
    theta: float,
    epsilon: float,
    delta: float,
    size: int,
    alpha: float,
    tail: Literal["left", "right"],
) -> list[float]:
    """Compute one-sided UMP tests for each choice of i in [size].

    * When tail="left", then the null hypothesis is: the estimate is at most ``i``
    * When tail="right", then the null hypothesis is: the estimate is at least ``i``

    :param Z: tulap random variable
    :param epsilon: noise parameter ε for the Tulap distribution
    :param delta: noise parameter δ for the Tulap distribution
    :param size: (approximate) number of trials
    :param theta: true probability of binomial distribution
    :param tail: either "left" or "right"
    :return: The probability of accepting the null hypothesis for each choice of i in [size].
    """
    stats = import_optional_dependency("scipy.stats")
    np = import_optional_dependency("numpy")

    values = np.arange(0, size + 1)
    B = stats.binom.pmf(k=values, n=size, p=theta)

    predicate = lambda s: np.dot(B, _tulap_cdf(values - s, 0.0, epsilon, delta)) > alpha

    root = binary_search(predicate)
    phi = _tulap_cdf(t=values - root, epsilon=epsilon, delta=delta)

    if tail == "left":
        return phi

    if tail == "right":
        return 1 - phi

    raise ValueError("tail must be either 'left' or 'right'")
