import pytest
import opendp.prelude as dp

dp.enable_features('floating-point', 'contrib')

def test_base_gaussian_curve():
    from opendp.measurements import make_base_gaussian
    from opendp.combinators import make_zCDP_to_approxDP
    input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    meas = make_zCDP_to_approxDP(make_base_gaussian(*input_space, 4.))
    curve = meas.map(d_in=1.)
    assert curve.epsilon(delta=0.) == float('inf')
    assert curve.epsilon(delta=1e-3) == 0.6880024554878086
    assert curve.epsilon(delta=1.) == 0.

    curve = make_zCDP_to_approxDP(make_base_gaussian(*input_space, 4.)).map(d_in=0.0)
    assert curve.epsilon(0.0) == 0.0
    with pytest.raises(Exception):
        curve.epsilon(delta=-0.0)

    curve = make_zCDP_to_approxDP(make_base_gaussian(*input_space, 0.)).map(d_in=1.0)
    assert curve.epsilon(delta=0.0) == float('inf')
    assert curve.epsilon(delta=0.1) == float('inf')

    curve = make_zCDP_to_approxDP(make_base_gaussian(*input_space, 0.)).map(d_in=0.0)
    assert curve.epsilon(delta=0.0) == 0.0
    assert curve.epsilon(delta=0.1) == 0.0


def test_base_gaussian_search():
    input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)

    def make_smd_gauss(scale, delta):
        return dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(dp.m.make_base_gaussian(*input_space, scale)), delta)

    fixed_meas = make_smd_gauss(1., 1e-5)
    ideal_dist = fixed_meas.map(1.)
    print("ideal dist", ideal_dist)
    print("check with ideal dist:", fixed_meas.check(1., ideal_dist))

    print("Standard", dp.binary_search_param(
        lambda s: make_smd_gauss(s, 1e-5),
        d_in=1., d_out=(1., 1e-5)))


def test_base_laplace():
    input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    meas = dp.m.make_base_laplace(*input_space, 10.5)
    print("base laplace:", meas(100.))
    print("epsilon", meas.map(1.))
    assert meas.check(1., .096)

def test_base_vector_laplace():
    input_space = dp.vector_domain(dp.atom_domain(T=float)), dp.l1_distance(T=float)
    meas = dp.m.make_base_laplace(*input_space, scale=10.5)
    print("base laplace:", meas([80., 90., 100.]))
    assert meas.check(1., 1.3)


def test_base_gaussian_smoothed_max_divergence():
    input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    meas = dp.c.make_zCDP_to_approxDP(dp.m.make_base_gaussian(*input_space, scale=10.5))
    print("base gaussian:", meas(100.))

    epsilon = meas.map(d_in=1.).epsilon(delta=.000001)
    print("epsilon:", epsilon)
    assert epsilon > .4


def test_base_gaussian_zcdp():
    input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    meas = input_space >> dp.m.then_base_gaussian(scale=1.5, MO=dp.ZeroConcentratedDivergence[float])
    print("base gaussian:", meas(100.))

    rho = meas.map(d_in=1.)
    print("rho:", rho)



def test_base_vector_gaussian():
    delta = .000001
    input_space = dp.vector_domain(dp.atom_domain(T=float)), dp.l2_distance(T=float)
    meas = dp.c.make_fix_delta(
        dp.c.make_zCDP_to_approxDP(
            dp.m.make_base_gaussian(*input_space, scale=10.5)), delta)
    print("base gaussian:", meas([80., 90., 100.]))
    assert meas.check(1., (0.6, delta))


def test_base_geometric():
    input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    meas = input_space >> dp.m.then_base_geometric(scale=2., bounds=(1, 10))
    print("base_geometric in constant time:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    meas = input_space >> dp.m.then_base_geometric(scale=2.)
    print("base_geometric:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_base_discrete_laplace():
    meas = dp.m.make_base_discrete_laplace(dp.atom_domain(T=int), dp.absolute_distance(T=int), scale=2.)
    print("base_discrete_laplace:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

def test_base_discrete_laplace_cks20():
    meas = dp.m.make_base_discrete_laplace_cks20(dp.atom_domain(T=int), dp.absolute_distance(T=int), scale=2.)
    print("base_discrete_laplace:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

def test_base_vector_discrete_laplace_cks20():
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.l1_distance(T=int)
    meas = input_space >> dp.m.then_base_discrete_laplace_cks20(scale=2.)
    print("vector base_dl:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_base_discrete_laplace_linear():
    input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    meas = dp.m.make_base_discrete_laplace_linear(*input_space, scale=2., bounds=(1, 10))
    print("base_discrete_laplace:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_base_vector_discrete_laplace():
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.l1_distance(T=int)
    meas = dp.m.make_base_discrete_laplace(*input_space, scale=2.)
    print("vector base_dl:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

def test_base_discrete_gaussian():
    input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    meas = dp.m.make_base_discrete_gaussian(*input_space, scale=2.)
    print("base_discrete_gaussian:", meas(100))
    assert meas.check(1, 0.5)
    assert meas.check(1, 0.125)

def test_base_vector_discrete_gaussian():
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.l2_distance(T=float)
    meas = dp.m.make_base_discrete_gaussian(*input_space, scale=2.)
    print("vector base_dl:", meas([100, 10, 12]))
    assert meas.check(1., 0.125)
    assert not meas.check(1., 0.124)

def test_make_count_by_ptr():
    input_space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    meas = (
        input_space >>
        dp.t.then_count_by(MO=dp.L1Distance[float], TV=float) >> 
        dp.m.then_base_laplace_threshold(scale=2., threshold=28.)
    )
    print("stability histogram:", meas(["CAT_A"] * 20 + ["CAT_B"] * 10))
    print(meas.map(1))
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

    input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    (input_space >> dp.m.then_gaussian(1.))(1.)

    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.l2_distance(T=float)
    (input_space >> dp.m.then_gaussian(1.))([1, 2, 3])

    input_space = dp.vector_domain(dp.atom_domain(T=float)), dp.l2_distance(T=float)
    (input_space >> dp.m.then_gaussian(1.))([1., 2., 3.])
