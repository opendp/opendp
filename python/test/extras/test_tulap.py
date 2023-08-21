import opendp.prelude as dp
import numpy as np  # type: ignore[import]
from opendp.extras.numpy.tulap import (
    BinomialTulap,
    _tulap_cdf,
    one_sided_pvalue,
    one_sided_uniformly_most_powerful_tests,
)
import math
import pytest


def approx_trials(n, prob=1, alpha=0):
    stats = pytest.importorskip("scipy.stats")
    # solve a quadratic form for this
    a = prob**2
    b = -((2 * n * prob) + ((stats.norm.ppf(q=alpha) ** 2) * prob * (1 - prob)))
    c = n**2
    n_trials = (-b + math.sqrt(b**2 - (4 * a * c))) / (2 * a)
    return int(round(n_trials))


# generate random samples from Tulap distribution using rejection sampling
# used for more efficient sampling from the Tulap distribution in tests
def sample_tulap_fast(n, m=0, epsilon=0, delta=0):
    """
    Fast, non-floating-safe sampling from the Tulap distribution
    m - real number
    b - (0, 1)
    q - [0, 1)

    Note:
    Tulap random variable Tulap(m, b, q) is continuous and symmetric of m
    """
    np = pytest.importorskip("numpy")
    stats = pytest.importorskip("scipy.stats")
    b = math.exp(-epsilon)
    q = (2 * delta * b) / (1 - b + 2 * delta * b)

    # q represents truncation
    if q >= 0:
        alpha = 0.95
        lcut = q / 2
        rcut = q / 2

        # calculate actual amount needed
        q = lcut + rcut
        n2 = approx_trials(n=n, prob=(1 - q), alpha=alpha)

        # sample from the original Tulambda distribution
        geos1 = stats.geom.rvs(size=n2, p=(1 - b))
        geos2 = stats.geom.rvs(size=n2, p=(1 - b))
        unifs = stats.uniform.rvs(loc=-1 / 2, scale=1, size=n2)  # range = [loc, loc+scale]
        samples = m + geos1 - geos2 + unifs  # numpy ndarray

        # cut the tails based on the untampered CDF (i.e. no cuts)
        probs = _tulap_cdf(samples, m=m, epsilon=epsilon)
        is_mid_bool = np.logical_and(
            np.less_equal(lcut, probs), np.less_equal(probs, (1 - rcut))
        ).astype(int)
        is_mid = []
        for i in range(len(is_mid_bool)):
            if is_mid_bool[i] == 1:
                is_mid.append(i)

        mids = samples[is_mid]
        length = len(mids)
        while length < n:
            diff = n - length
            Zs = sample_tulap_fast(n=diff, m=m, epsilon=epsilon, delta=delta)
            mids = np.concatenate((mids, Zs), axis=None)
            length = len(mids)
        return mids[:n]

    geos1 = stats.geom.rvs(size=n2, p=(1 - b))
    geos2 = stats.geom.rvs(size=n2, p=(1 - b))
    unifs = stats.uniform.rvs(loc=-1 / 2, scale=1, size=n2)
    samples = m + geos1 - geos2 + unifs
    return samples




dp.enable_features("contrib")


def test__tulap_cdf_positive_input():
    """Test with a positive t, checks basic operation"""
    t = np.array([1])  # Adjusted to array
    result = _tulap_cdf(t, epsilon=0.1, delta=1e-8)
    assert isinstance(result, np.ndarray)
    assert result[0] > 0, "Result should be positive for positive t"


def test__tulap_cdf_negative_input():
    """Test with a negative t, checks basic operation"""
    t = -1
    result = _tulap_cdf(t, epsilon=0.1, delta=0.1)
    assert isinstance(result, np.ndarray)
    assert result < 1, "Result should be less than 1 for negative t"


def test__tulap_cdf_array_input():
    """Test with an array of t values"""
    t = np.array([0, 1, -1])
    result = _tulap_cdf(t, epsilon=0.1, delta=1e-6)
    assert isinstance(result, np.ndarray)
    assert len(result) == 3, "Result should have the same length as input"

    t_values = np.array([-1.0, -0.5, 0.0, 0.5, 1.0])

    ptulap_results = _tulap_cdf(t_values, epsilon=0.5, delta=1e-7)
    assert list(ptulap_results) == [
        0.30326526920325075,
        0.37754063104407853,
        0.5,
        0.6224593689559215,
        0.6967347307967493,
    ]


def test__tulap_cdf_inf_handling():
    """Test to ensure infinities are handled correctly"""
    pytest.importorskip("numpy")
    with pytest.warns():
        result = _tulap_cdf(np.array([np.inf]), epsilon=0.1, delta=1e-8)
    assert not np.isinf(result).any(), "Result should not contain infinities"



def test_confidence_interval():
    pytest.importorskip("scipy")
    Z = 100.0
    epsilon, delta = 0.1, 1e-8
    
    size = 1000
    samples = sample_tulap_fast(n=10_000, m=Z, epsilon=epsilon, delta=delta) / size
    tulap = BinomialTulap(Z, epsilon=epsilon, delta=delta, size=size)

    alpha = 0.05
    ci = tulap.confidence_interval(alpha=alpha)
    # scipy functions give slightly different numbers on 1.13 vs 1.14
    assert np.allclose(ci, [
        0.06555402885319662,
        0.13444597114680337,
    ])

    # sampling from the tulap is somewhat slow -->
    #    so a small number of samples is taken --> 
    #       so empirical alpha varies somewhat -->
    #        so empirical alpha is not checked
    empirical_alpha = (np.less(samples, ci[0]) | np.greater(samples, ci[1])).mean()  # type: ignore[index]
    assert empirical_alpha < alpha

    lower = tulap.confidence_interval(alpha=alpha, side="lower")
    assert np.allclose(lower, 0.07261474401027863)

    empirical_alpha = np.less(samples, lower).mean()
    assert empirical_alpha < alpha


def test_oneside_pvalue():
    pytest.importorskip("scipy")
    tulap = BinomialTulap(5.0, epsilon=0.1, delta=1e-6, size=10)
    print(tulap)
    # should be approximately equal to 0.5
    assert np.allclose(tulap.p_value(theta=0.5, tail="right"), 0.4993195913951362)
    assert np.allclose(tulap.p_value(theta=0.5, tail="left"), 0.49970384610486424)

    pvalue = one_sided_pvalue(
        Z=3, epsilon=0.5, delta=1e-8, size=10, theta=0.5, tail="right"
    )
    assert np.allclose(pvalue, 0.7606589354450621)


def test_twoside_pvalue():
    pytest.importorskip("scipy")
    tulap = BinomialTulap(1.0, epsilon=0.5, delta=1e-8, size=10)
    assert np.allclose(tulap.p_value(theta=0.5), 0.18443148204450355)


def test_1s_ump_basic():
    pytest.importorskip("scipy")
    result = one_sided_uniformly_most_powerful_tests(
        theta=0.5, epsilon=0.1, delta=1e-7, size=10, alpha=0.05, tail="left"
    )
    # tail should be monotonically increasing
    assert (np.diff(result) > 0).sum() == len(result) - 1

    result = one_sided_uniformly_most_powerful_tests(
        theta=0.5, epsilon=0.1, delta=1e-7, size=10, alpha=0.05, tail="right"
    )
    # tail should be monotonically increasing
    assert (np.diff(result) < 0).sum() == len(result) - 1
