"""
This module requires extra installs: ``pip install 'opendp[numpy]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.numpy.canonical``.
"""

from __future__ import annotations

import math
from typing import Literal
from opendp._lib import import_optional_dependency
from opendp.mod import binary_search


class BinomialCND:
    """
    Utilities to conduct statistical inference on the output of the canonical noise mechanism on binomially-distributed data.

    Use :func:`opendp.measurements.make_canonical_noise` to instantiate the canonical noise mechanism.

    The mechanism outputs a sample from X + N,
    where X ~ Binomial(n=size, p=theta), N ~ CND(0, d_in, d_out).

    A counting query is a realization of X ~ Binomial(n=size, p=theta) when,
    for each row in data with ``size`` rows, a counter is incremented with probability theta.

    :param estimate: the differentially private outcome of the canonical noise mechanism on binomial data
    :param d_in: sensitivity of the input to the mechanism
    :param d_out: privacy parameters (ε, δ)
    :param size: (approximate) total number of records in the sensitive dataset
    """

    def __init__(
        self, estimate: float, d_in: float, d_out: tuple[float, float], size: int
    ) -> None:
        self.estimate = estimate
        self.d_in = d_in
        self.d_out = d_out
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
                self.estimate, self.d_in, self.d_out, self.size, alpha
            )

        if side in {"lower", "upper"}:
            return one_sided_confidence_interval(
                self.estimate, self.d_in, self.d_out, self.size, alpha, side
            )

        raise ValueError(f"tail must be None, 'lower' or 'upper', not '{side}'")  # pragma: no cover

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
                self.estimate, self.d_in, self.d_out, self.size, theta
            )

        if tail in {"left", "right"}:
            return one_sided_pvalue(
                self.estimate, self.d_in, self.d_out, self.size, theta, tail
            )

        raise ValueError(f"tail must be None, 'left' or 'right', not '{tail}'")  # pragma: no cover

    def __repr__(self) -> str:
        return f"BinomialCND(estimate={self.estimate}, d_in={self.d_in}, d_out={self.d_out}, size={self.size})"


# If the canonical noise mechanism is generalized beyond (ε, δ)-DP, 
# then this needs to be updated, as it only considers the Tulap noise generated from this definition.
def _cnd_cdf(t, shift, d_in: float, d_out: tuple[float, float]):
    """Computes the value of the cumulative density function Pr[T <= t] where T ~ Tulap(shift, b, q),
    where b and q are calibrated to satisfy d_out-DP when sensitivity is d_in"""
    np = import_optional_dependency("numpy")
    epsilon, delta = d_out

    t = (np.atleast_1d(t) - shift) / d_in
    b = math.exp(-epsilon)
    q = (2 * delta * b) / (1 - b + 2 * delta * b)
    lcut = q / 2
    rcut = q / 2
    r = np.rint(t)
    g = -math.log(b)
    l = math.log(1 + b)  # noqa
    k = 1 - b
    
    with np.errstate(over="ignore"):
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
    Z: float, d_in: float, d_out: tuple[float, float], size: int, alpha: float
) -> tuple[float, float]:
    """Compute a two-sided confidence interval centered on ``Z``.

    :param Z: realization of cnd random variable
    :param d_in: sensitivity of the input to the mechanism
    :param d_out: privacy parameters (ε, δ)
    :param size: (approximate) total number of records in the sensitive dataset
    :param alpha: statistical significance level
    :return: confidence interval bounds
    """
    mle = max(min(Z / size, 1.0), 0.0)
    predicate = lambda B: two_sided_pvalue(B * size, d_in, d_out, size, mle) > alpha

    L = binary_search(predicate, bounds=(0.0, mle)) if mle > 0 else 0.0
    U = binary_search(predicate, bounds=(mle, 1.0)) if mle < 1 else 1.0

    return float(L), float(U)


def one_sided_confidence_interval(
    Z: float,
    d_in: float,
    d_out: tuple[float, float],
    size: int,
    alpha: float,
    side: Literal["lower", "upper"],
) -> float:
    """Compute a one-sided confidence interval for ``self.estimate``.

    * tail="lower": return a one-sided confidence interval lower bound
    * tail="upper": return a one-sided confidence interval upper bound

    :param Z: realization of cnd random variable
    :param d_in: sensitivity of the input to the mechanism
    :param d_out: privacy parameters (ε, δ)
    :param size: (approximate) total number of records in the sensitive dataset
    :param alpha: statistical significance level
    :param side: configure interval type
    :return: lower or upper confidence interval bound
    """
    mle = max(min(Z / size, 1.0), 0.0)
    tail = {"lower": "left", "upper": "right"}.get(side)
    if tail is None:
        raise ValueError(f"tail must be 'lower' or 'upper', not {tail}")  # pragma: no cover
    pred = lambda B: one_sided_pvalue(B * size, d_in, d_out, size, mle, tail) > alpha  # type: ignore[arg-type]
    return float(binary_search(pred, bounds=(0.0, 1.0)))


def two_sided_pvalue(
    Z: float, d_in: float, d_out: tuple[float, float], size: int, theta: float
) -> float:
    """Computes a p-value for a hypothesis test on the probability of success.

    :param Z: realization of cnd random variable
    :param d_in: sensitivity of the input to the mechanism
    :param d_out: privacy parameters (ε, δ)
    :param size: (approximate) number of trials
    :param theta: true probability of binomial distribution
    :return: Probability of observing Z when the true success rate is theta
    """
    T = abs(size * theta - Z)
    left_tail = one_sided_pvalue(
        Z=size * theta + T,
        d_in=d_in,
        d_out=d_out,
        size=size,
        theta=theta,
        tail="right",
    )
    right_tail = one_sided_pvalue(
        Z=size * theta - T,
        d_in=d_in,
        d_out=d_out,
        size=size,
        theta=theta,
        tail="right",
    )

    return float(left_tail + (1 - right_tail))


def one_sided_pvalue(
    Z: float,
    d_in: float,
    d_out: tuple[float, float],
    size: int,
    theta: float,
    tail: Literal["left", "right"],
) -> float:
    """Computes a p-value for a hypothesis test on the probability of success.

    * tail="left": null hypothesis states that true proportion is no less than theta
    * tail="right": null hypothesis states that true proportion is no greater than theta

    :param Z: realization of cnd random variable
    :param d_in: sensitivity of the input to the mechanism
    :param d_out: privacy parameters (ε, δ)
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
        F = _cnd_cdf(values, Z, d_in=d_in, d_out=d_out)
    elif tail == "left":
        F = 1 - _cnd_cdf(values, Z, d_in=d_in, d_out=d_out)
    else:
        raise ValueError(f"tail must be 'left' or 'right', not '{tail}'")  # pragma: no cover
    return float(np.dot(F.T, B))


def one_sided_uniformly_most_powerful_tests(
    d_in: float, 
    d_out: tuple[float, float],
    size: int,
    theta: float,
    alpha: float,
    tail: Literal["left", "right"],
) -> list[float]:
    """Compute one-sided UMP tests for each choice of i in [size].

    * When tail="left", then the null hypothesis is: the estimate is at most ``i``
    * When tail="right", then the null hypothesis is: the estimate is at least ``i``

    :param d_in: sensitivity of the input to the mechanism
    :param d_out: privacy parameters (ε, δ)
    :param size: (approximate) number of trials
    :param theta: true probability of binomial distribution
    :param alpha: statistical significance level
    :param tail: either "left" or "right"
    :return: The probability of accepting the null hypothesis for each choice of i in [size].
    """
    stats = import_optional_dependency("scipy.stats")
    np = import_optional_dependency("numpy")

    values = np.arange(0, size + 1)
    B = stats.binom.pmf(k=values, n=size, p=theta)

    predicate = lambda s: np.dot(B, _cnd_cdf(values, s, d_in, d_out)) > alpha

    root = binary_search(predicate)
    phi = _cnd_cdf(values, root, d_in, d_out)

    if tail == "left":
        return phi

    if tail == "right":
        return 1 - phi

    raise ValueError(f"tail must be either 'left' or 'right', not '{tail}'")  # pragma: no cover
