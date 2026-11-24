import opendp.prelude as dp
from opendp.extras.numpy.canonical import (
    BinomialCND,
    _cnd_cdf,
    one_sided_pvalue,
    one_sided_uniformly_most_powerful_tests,
)
import math
import pytest

np = pytest.importorskip("numpy")
stats = pytest.importorskip("scipy.stats")
scipy = pytest.importorskip("scipy")


def approx_trials(n, prob=1, alpha=0):
    # solve a quadratic form for this
    a = prob**2
    b = -((2 * n * prob) + ((stats.norm.ppf(q=alpha) ** 2) * prob * (1 - prob)))
    c = n**2
    n_trials = (-b + math.sqrt(b**2 - (4 * a * c))) / (2 * a)
    return int(round(n_trials))


# generate random samples from Tulap distribution using rejection sampling
# used for more efficient sampling from the Tulap distribution in tests
def sample_tulap_fast(n, shift, d_in, d_out):
    """Fast inexact sampling from the Tulap distribution"""
    epsilon, delta = d_out
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
        unifs = stats.uniform.rvs(
            loc=-1 / 2, scale=1, size=n2
        )  # range = [loc, loc+scale]
        samples = shift + geos1 - geos2 + unifs  # numpy ndarray

        # cut the tails based on the untampered CDF (i.e. no cuts)
        probs = _cnd_cdf(samples, shift=shift, d_in=d_in, d_out=(epsilon, 0.0))
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
            Zs = sample_tulap_fast(n=diff, shift=shift, d_in=d_in, d_out=d_out)
            mids = np.concatenate((mids, Zs), axis=None)
            length = len(mids)
        return mids[:n]

    geos1 = stats.geom.rvs(size=n2, p=(1 - b))
    geos2 = stats.geom.rvs(size=n2, p=(1 - b))
    unifs = stats.uniform.rvs(loc=-1 / 2, scale=1, size=n2)
    samples = shift + (geos1 - geos2 + unifs) * d_in
    return samples


def test__tulap_cdf_positive_input():
    """Test with a positive t, checks basic operation"""
    t = np.array([1])  # Adjusted to array
    result = _cnd_cdf(t, shift=0.0, d_in=1.0, d_out=(0.1, 1e-8))
    assert isinstance(result, np.ndarray)
    assert result[0] > 0, "Result should be positive for positive t"


def test__tulap_cdf_negative_input():
    """Test with a negative t, checks basic operation"""
    t = -1
    result = _cnd_cdf(t, shift=0.0, d_in=1.0, d_out=(0.1, 0.1))
    assert isinstance(result, np.ndarray)
    assert result < 1, "Result should be less than 1 for negative t"


def test__tulap_cdf_array_input():
    """Test with an array of t values"""
    t = np.array([0, 1, -1])
    result = _cnd_cdf(t, shift=0.0, d_in=1.0, d_out=(0.1, 1e-6))
    assert isinstance(result, np.ndarray)
    assert len(result) == 3, "Result should have the same length as input"

    t_values = np.array([-1.0, -0.5, 0.0, 0.5, 1.0])

    ptulap_results = _cnd_cdf(t_values, shift=0.0, d_in=1.0, d_out=(0.5, 1e-7))
    assert np.allclose(
        ptulap_results,
        [
            0.30326526920325075,
            0.37754063104407853,
            0.5,
            0.6224593689559215,
            0.6967347307967493,
        ],
    )


def test__tulap_cdf_inf_handling():
    """Test to ensure infinities are handled correctly"""
    with pytest.warns():
        result = _cnd_cdf(np.array([np.inf]), shift=0.0, d_in=1.0, d_out=(0.1, 1e-8))
    assert not np.isinf(result).any(), "Result should not contain infinities"


def test_confidence_interval():
    Z = 100.0
    d_in = 1.0
    d_out = 0.1, 1e-8

    size = 1000
    tulap = BinomialCND(Z, d_in=d_in, d_out=d_out, size=size)

    alpha = 0.05
    lower, upper = tulap.confidence_interval(alpha=alpha)
    assert lower < Z / size < upper
    assert tulap.p_value(lower) == pytest.approx(alpha, abs=1e-5)
    assert tulap.p_value(upper) == pytest.approx(alpha, abs=1e-5)

    one_sided_lower = tulap.confidence_interval(alpha=alpha, side="lower")
    one_sided_upper = tulap.confidence_interval(alpha=alpha, side="upper")
    assert one_sided_lower < Z / size < one_sided_upper
    assert tulap.p_value(one_sided_lower, tail="right") == pytest.approx(
        alpha, abs=1e-5
    )
    assert tulap.p_value(one_sided_upper, tail="left") == pytest.approx(alpha, abs=1e-5)


def test_confidence_interval_binomial_coverage():
    # At Z=0, theta=0.2 is accepted by the left-tail test.
    # The old inversion incorrectly excluded it from the upper interval.
    size, theta, alpha = 10, 0.2, 0.05
    d_out = (10.0, 0.0)
    for z in (0.0, 0.5):
        cnd = BinomialCND(z, 1.0, d_out, size)
        upper = cnd.confidence_interval(alpha, side="upper")
        assert upper >= theta
        assert cnd.p_value(upper, tail="left") == pytest.approx(alpha, abs=1e-5)

    # Extreme observations may put the decision boundary outside [0, 1].
    for z, endpoint in [(-0.5, 0.0), (size + 0.5, 1.0)]:
        cnd = BinomialCND(z, 1.0, d_out, size)
        assert cnd.confidence_interval(alpha, side="lower") == endpoint
        assert cnd.confidence_interval(alpha, side="upper") == endpoint
        assert cnd.confidence_interval(alpha) == (endpoint, endpoint)


def test_oneside_pvalue():
    tulap = BinomialCND(5.0, d_in=1.0, d_out=(0.1, 1e-6), size=10)
    assert (
        str(tulap) == "BinomialCND(estimate=5.0, d_in=1.0, d_out=(0.1, 1e-06), size=10)"
    )

    assert np.allclose(tulap.p_value(theta=0.5, tail="right"), 0.5)
    assert np.allclose(tulap.p_value(theta=0.5, tail="left"), 0.5)

    pvalue = one_sided_pvalue(
        Z=3, d_in=1.0, d_out=(0.5, 1e-8), size=10, theta=0.5, tail="right"
    )
    assert np.allclose(pvalue, 0.7616207531435369)


def test_twoside_pvalue():
    tulap = BinomialCND(1.0, d_in=1.0, d_out=(0.5, 1e-8), size=10)
    assert np.allclose(tulap.p_value(theta=0.5), 0.1841407488015684)


def test_1s_ump_basic():
    pytest.importorskip("scipy")
    result = one_sided_uniformly_most_powerful_tests(
        d_in=1.0, d_out=(0.1, 1e-7), size=10, theta=0.5, alpha=0.05, tail="left"
    )
    # tail should be monotonically increasing
    assert (np.diff(result) > 0).sum() == len(result) - 1

    result = one_sided_uniformly_most_powerful_tests(
        d_in=1.0, d_out=(0.1, 1e-7), size=10, theta=0.5, alpha=0.05, tail="right"
    )
    # tail should be monotonically increasing
    assert (np.diff(result) < 0).sum() == len(result) - 1


def test_canonical_context_no_transformation():
    context = dp.Context.compositor(
        data=1.0,
        privacy_unit=dp.unit_of(absolute=1.0),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=2,
        domain=dp.atom_domain(nan=False, T=float),
    )

    assert isinstance(context.query().canonical_noise().release(), float)
    assert isinstance(
        context.query().canonical_noise(binomial_size=1000).release(), BinomialCND
    )


def test_canonical_context_with_transformation():
    context = dp.Context.compositor(
        data=[1.0, 2.0, 3.0, 4.0],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=2,
    )

    query = context.query().impute_constant(0.0).clamp((0.0, 1.0)).sum()

    assert isinstance(query.canonical_noise().release(), float)
    assert isinstance(query.canonical_noise(binomial_size=1000).release(), BinomialCND)


def test_canonical_context_with_partial_transformation():
    context = dp.Context.compositor(
        data=[1.0, 2.0, 3.0, 4.0],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=2,
    )

    with pytest.raises(
        ValueError,
        match="Canonical noise requires all arguments in the input query to be specified.",
    ):
        context.query().resize(constant=1.0).clamp((0.0, 1.0)).sum().canonical_noise()
