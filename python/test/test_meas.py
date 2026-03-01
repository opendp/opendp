import pytest
import opendp.prelude as dp


def test_gaussian_curve():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)
    meas = dp.c.make_zCDP_to_approxDP(dp.m.make_gaussian(*input_space, 4.))
    profile = meas.map(d_in=1.)
    assert profile.epsilon(delta=0.) == float('inf')
    # see cdp_delta for formula of 0.688 and 0.151

    # cdp_epsilon(rho=(1/4)^2 / 2, delta=1e-3)
    assert profile.epsilon(delta=1e-3) == 0.6880024554878085
    assert profile.epsilon(delta=1.) == 0.
    # cdp_delta(rho=(1/4)^2 / 2, epsilon=0.0)
    assert profile.delta(epsilon=0.) == 0.1508457845622862
    # reuse the constant above
    assert profile.delta(epsilon=0.6880024554878085) == 1e-3

    profile = dp.c.make_zCDP_to_approxDP(dp.m.make_gaussian(*input_space, 4.)).map(d_in=0.0)
    assert profile.epsilon(0.0) == 0.0
    with pytest.raises(dp.OpenDPException):
        profile.epsilon(delta=-0.0)
    with pytest.raises(dp.OpenDPException):
        profile.delta(epsilon=-0.0)
        
    profile = dp.c.make_zCDP_to_approxDP(dp.m.make_gaussian(*input_space, 0.)).map(d_in=1.0)
    assert profile.epsilon(delta=0.0) == float('inf')
    assert profile.epsilon(delta=0.1) == float('inf')
    assert profile.delta(epsilon=0.0) == 1.0
    assert profile.delta(epsilon=0.1) == 1.0

    profile = dp.c.make_zCDP_to_approxDP(dp.m.make_gaussian(*input_space, 0.)).map(d_in=0.0)
    assert profile.epsilon(delta=0.0) == 0.0
    assert profile.epsilon(delta=0.1) == 0.0
    assert profile.delta(epsilon=0.0) == 0.0
    assert profile.delta(epsilon=0.1) == 0.0



def test_gaussian_search():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)

    def make_approx_gauss(scale, delta):
        return dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(dp.m.make_gaussian(*input_space, scale)), delta)

    fixed_meas = make_approx_gauss(1., 1e-5)
    ideal_dist = fixed_meas.map(1.)
    print("ideal dist", ideal_dist)
    print("check with ideal dist:", fixed_meas.check(1., ideal_dist))

    print("Standard", dp.binary_search_param(
        lambda s: make_approx_gauss(s, 1e-5),
        d_in=1., d_out=(1., 1e-5)))

def new_make_noise(measure):
    def make_noise(domain, metric, scale):
        return dp.m.make_noise(domain, metric, measure, scale)
    return make_noise

@pytest.mark.parametrize("constructor", [
    dp.m.make_laplace,
    new_make_noise(dp.max_divergence())
])
def test_laplace(constructor):
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)
    meas = constructor(*input_space, 1)
    assert -50 < meas(0.) < 50
    assert meas.map(1.0) == 1.0


@pytest.mark.parametrize("constructor", [
    dp.m.make_laplace,
    new_make_noise(dp.max_divergence())
])
def test_vector_laplace(constructor):
    input_space = dp.vector_domain(dp.atom_domain(T=float, nan=False)), dp.l1_distance(T=float)
    meas = constructor(*input_space, scale=1.)
    release = meas([0., 0., 0.])
    assert -50 < min(release)
    assert max(release) < 50
    assert meas.map(1.0) == 1.0


def test_gaussian_smoothed_max_divergence():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)
    meas = dp.c.make_zCDP_to_approxDP(dp.m.make_gaussian(*input_space, scale=10.5))
    print("base gaussian:", meas(100.))

    epsilon = meas.map(d_in=1.).epsilon(delta=.000001)
    print("epsilon:", epsilon)
    assert epsilon > .4


def test_gaussian_zcdp():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)
    meas = input_space >> dp.m.then_gaussian(scale=1.5, MO=dp.ZeroConcentratedDivergence)
    print("base gaussian:", meas(100.))

    rho = meas.map(d_in=1.)
    print("rho:", rho)



def test_vector_gaussian():
    delta = .000001
    input_space = dp.vector_domain(dp.atom_domain(T=float, nan=False)), dp.l2_distance(T=float)
    meas = dp.c.make_fix_delta(
        dp.c.make_zCDP_to_approxDP(
            dp.m.make_gaussian(*input_space, scale=10.5)), delta)
    print("base gaussian:", meas([80., 90., 100.]))
    assert meas.check(1., (0.6, delta))


def test_geometric():
    input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    meas = input_space >> dp.m.then_geometric(scale=2., bounds=(1, 10))
    print("base_geometric in constant time:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    meas = input_space >> dp.m.then_geometric(scale=2.)
    print("base_geometric:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_discrete_laplace():
    meas = dp.m.make_laplace(dp.atom_domain(T=int), dp.absolute_distance(T=int), scale=2.)
    print("base_discrete_laplace:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

def test_discrete_laplace_cks20():
    meas = dp.m.make_laplace(dp.atom_domain(T=int), dp.absolute_distance(T=int), scale=2.)
    print("base_discrete_laplace:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

def test_vector_discrete_laplace_cks20():
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.l1_distance(T=int)
    meas = input_space >> dp.m.then_laplace(scale=2.)
    print("vector base_dl:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_discrete_laplace_linear():
    input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    meas = dp.m.make_geometric(*input_space, scale=2., bounds=(1, 10))
    print("base_discrete_laplace:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_vector_discrete_laplace():
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.l1_distance(T=int)
    meas = dp.m.make_laplace(*input_space, scale=2.)
    print("vector base_dl:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

def test_discrete_gaussian():
    input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    meas = dp.m.make_gaussian(*input_space, scale=2.)
    print("base_discrete_gaussian:", meas(100))
    assert meas.check(1, 0.5)
    assert meas.check(1, 0.125)

def test_vector_discrete_gaussian():
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.l2_distance(T=float)
    meas = dp.m.make_gaussian(*input_space, scale=2.)
    print("vector base_dl:", meas([100, 10, 12]))
    assert meas.check(1., 0.125)
    assert not meas.check(1., 0.124)

def test_make_count_by_ptr():
    meas = (
        dp.space_of(list[str]) >>
        dp.t.then_count_by() >> 
        dp.m.then_laplace_threshold(scale=2., threshold=28)
    )
    print("stability histogram:", meas(["CAT_A"] * 20 + ["CAT_B"] * 10))
    assert meas.map(1) == (0.5, 5.175928103895444e-07)
    assert meas.check(1, (1.0, 1e-6))

def test_randomized_response():
    meas = dp.m.make_randomized_response(categories=["A", "B", "C", "D"], prob=0.75)
    print("randomized response:", meas("A"))
    import math
    assert meas.check(1, math.log(9.))
    assert not meas.check(1, math.log(8.999))


def test_randomized_response_bool():
    meas = dp.m.make_randomized_response_bool(prob=0.75)
    print("randomized response:", meas(True))
    import math
    assert meas.check(1, math.log(3.))
    assert not meas.check(1, math.log(2.999))


def test_gaussian():
    input_space = dp.atom_domain(T=int), dp.absolute_distance(T=float)
    (input_space >> dp.m.then_gaussian(1.))(1)

    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)
    (input_space >> dp.m.then_gaussian(1.))(1.)

    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.l2_distance(T=float)
    (input_space >> dp.m.then_gaussian(1.))([1, 2, 3])

    input_space = dp.vector_domain(dp.atom_domain(T=float, nan=False)), dp.l2_distance(T=float)
    (input_space >> dp.m.then_gaussian(1.))([1., 2., 3.])


@pytest.mark.parametrize("measure,d_out", [
    # d_in * 2 / scale = 2
    (dp.max_divergence(), 2), 
    # (d_in * 2 / scale)^2 / 8
    (dp.zero_concentrated_divergence(), 1 / 2)
])
def test_noisy_max(measure, d_out):
    input_domain = dp.vector_domain(dp.atom_domain(T=dp.usize))
    input_metric = dp.linf_distance(T=dp.usize)

    meas = (input_domain, input_metric) >> dp.m.then_noisy_max(measure, 1.)
    # fails with very small probability
    assert meas([0, 0, 20, 40]) == 3  # because score 3 is by far the greatest
    assert meas.map(1) == d_out


@pytest.mark.parametrize("measure,d_out", [
    # (d_in * 2) / scale * 2 = 4
    (dp.max_divergence(), 4), 
    # ((d_in * 2) / scale)^2 / 8 * 2 = 1
    (dp.zero_concentrated_divergence(), 1)
])
def test_noisy_top_k(measure, d_out):
    input_domain = dp.vector_domain(dp.atom_domain(T=dp.usize))
    input_metric = dp.linf_distance(T=dp.usize)

    meas = (input_domain, input_metric) >> dp.m.then_noisy_top_k(measure, 2, 1.)
    # fails with very small probability
    assert meas([0, 0, 20, 40]) == [3, 2]  # because score 3 and then 2 are by far the greatest

    assert meas.map(1) == d_out


def test_alp_histogram():
    counter = dp.t.make_count_by(
        dp.vector_domain(dp.atom_domain(T=str)),
        dp.symmetric_distance())

    alp_meas = counter >> dp.m.then_alp_queryable(
        scale=1.,
        total_limit=24,
        value_limit=24,
    )

    alp_qbl = alp_meas(["A"] * 20 + ["B"] * 10)

    print("Should be high-ish", alp_qbl("A"))
    print("Should be mid-ish", alp_qbl("B"))
    print("Should be low-ish", alp_qbl("C"))
    assert alp_meas.map(1) == 1

def test_randomized_response_bitvec():
    np = pytest.importorskip('numpy')
    f = 1e-20
    m = 3
    m_rr = dp.m.make_randomized_response_bitvec(
        dp.bitvector_domain(max_weight=m), dp.discrete_distance(), f=f
    )

    # the postprocessor expects little endian data
    data = np.packbits(
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0], 
        bitorder='little'
    )

    # roundtrip: bytes -> mech -> numpy
    release = np.frombuffer(m_rr(data), dtype=np.uint8)
    assert np.array_equal(data, release)
    # epsilon is 2 * m * ln((2 - f) / f)
    assert m_rr.map(1) == 280.4690942426452

    sums = dp.m.debias_randomized_response_bitvec(
        [m_rr(data)] * 40,
        f=f
    )
    signs = np.packbits((np.array(sums) > 0).astype(int), bitorder='little')
    assert np.array_equal(signs, release)


def test_laplace_threshold_int():
    domain = dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=int))
    metric = dp.l01inf_distance(dp.absolute_distance(T=int))
    meas = dp.m.make_laplace_threshold(domain, metric, scale=2., threshold=28)
    release = meas({str(i): i * 10 for i in range(10)})
    # 0 + noise is likely not over 28
    assert "0" not in release
    # 90 + noise is likely over 28
    assert "9" in release
    # delta is the mass of the right tail of integer laplace greater than 28 with conservative arithmetic
    # 5 = 10 / 2 = Δ / σ
    assert meas.map((1, 10, 10)) == (5.0, 4.659221997116436e-05)

def test_laplace_threshold_float():
    domain = dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=float, nan=False))
    metric = dp.l01inf_distance(dp.absolute_distance(T=float))
    meas = dp.m.make_laplace_threshold(domain, metric, scale=2., threshold=28.)
    release = meas({str(i): i * 10.0 for i in range(10)})
    # 0 + noise is likely not over 28
    assert "0" not in release, release
    # 90 + noise is likely over 28
    assert "9" in release, release
    print(meas.map((1, 10, 10)))
    # delta is the mass of the right tail of continuous laplace greater than 28 with conservative arithmetic
    # the looser continuous laplace is used because discrete laplace tail bound is not computable with large constants
    assert meas.map((1, 10, 10)) == (5.0, 6.17049020433802e-05)

def test_gaussian_threshold_int():
    domain = dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=int))
    metric = dp.l02inf_distance(dp.absolute_distance(T=int))
    meas = dp.m.make_gaussian_threshold(domain, metric, scale=2., threshold=28)
    release = meas({str(i): i * 10 for i in range(10)})
    # 0 + noise is likely not over 28
    assert "0" not in release, release
    # 90 + noise is likely over 28
    assert "9" in release, release
    # delta is the mass of the right tail of continuous gaussian greater than 28 with conservative arithmetic
    # the continuous gaussian is used because discrete gaussian tail bound is not analytic
    # 12.5 = (10 / 2)^2 / 2 = (Δ / σ)^2 / 2
    assert meas.map((1, 10, 10)) == (12.5, 1.1102230246251565e-16)

def make_noise_threshold_zCDP(domain, metric, scale, threshold):
    measure = dp.approximate(dp.zero_concentrated_divergence())
    return dp.m.make_noise_threshold(domain, metric, measure, scale, threshold)

@pytest.mark.parametrize("constructor", [
    dp.m.make_gaussian_threshold,
    make_noise_threshold_zCDP
])
def test_gaussian_threshold_float(constructor):
    domain = dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=float, nan=False))
    metric = dp.l02inf_distance(dp.absolute_distance(T=float))
    meas = constructor(domain, metric, scale=2., threshold=28.)
    release = meas({str(i): i * 10.0 for i in range(10)})
    # 0 + noise is likely not over 28
    assert "0" not in release, release
    # 90 + noise is likely over 28
    assert "9" in release, release
    # delta is the mass of the right tail of continuous gaussian greater than 28 with conservative arithmetic
    # the continuous gaussian is used because discrete gaussian tail bound is not analytic
    # 12.5 = (10 / 2)^2 / 2 = (Δ / σ)^2 / 2
    assert meas.map((1, 10, 10)) == (12.5, 1.1102230246251565e-16)


def test_canonical_noise():
    space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)
    m_cnd = space >> dp.m.then_canonical_noise(d_in=1., d_out=(1., 1e-6))

    assert m_cnd.map(1.) == (1.0, 1e-6)
    # just check that it runs
    assert isinstance(m_cnd(0.), float)
