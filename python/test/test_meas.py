from opendp.mod import enable_features
from opendp.typing import *

enable_features('floating-point', 'contrib')

def test_base_gaussian_curve():
    from opendp.meas import make_base_gaussian
    curve = make_base_gaussian(4.).map(1.)
    print(curve.epsilon(1e-3))


def test_base_gaussian_search():
    from opendp.meas import make_base_gaussian, make_base_analytic_gaussian
    from opendp.comb import make_fix_delta
    from opendp.mod import binary_search_param

    fixed_meas = make_fix_delta(make_base_analytic_gaussian(1.), 1e-5)
    ideal_dist = fixed_meas.map(1.)
    print("ideal dist", ideal_dist)
    print("check with ideal dist:", fixed_meas.check(1., ideal_dist))

    print("Analytic", binary_search_param(
        lambda s: make_fix_delta(make_base_analytic_gaussian(s), 1e-5),
        d_in=1., d_out=(1., 1e-5)))
    print("Standard", binary_search_param(
        lambda s: make_fix_delta(make_base_gaussian(s), 1e-5),
        d_in=1., d_out=(1., 1e-5)))


def test_base_laplace():
    from opendp.meas import make_base_laplace
    meas = make_base_laplace(scale=10.5)
    print("base laplace:", meas(100.))
    print("epsilon", meas.map(1.))
    assert meas.check(1., .096)


def test_base_vector_laplace():
    from opendp.meas import make_base_laplace
    meas = make_base_laplace(scale=10.5, D="VectorDomain<AllDomain<f64>>")
    print("base laplace:", meas([80., 90., 100.]))
    assert meas.check(1., 1.3)


def test_base_gaussian_smoothed_max_divergence():
    from opendp.meas import make_base_gaussian

    meas = make_base_gaussian(scale=10.5)
    print("base gaussian:", meas(100.))

    epsilon = meas.map(d_in=1.).epsilon(delta=.000001)
    print("epsilon:", epsilon)
    assert epsilon > .5


def test_base_gaussian_zcdp():
    from opendp.meas import make_base_gaussian

    meas = make_base_gaussian(scale=1.5, MO=ZeroConcentratedDivergence[float])
    print("base gaussian:", meas(100.))

    rho = meas.map(d_in=1.)
    print("rho:", rho)


def test_base_analytic_gaussian():
    from opendp.meas import make_base_analytic_gaussian
    meas = make_base_analytic_gaussian(scale=1.5)
    print("base analytic gaussian:", meas(100.))

    # analytic gaussian allows epsilon > 1
    epsilon = meas.map(1.).epsilon(delta=.000001)
    print("epsilon:", epsilon)
    assert epsilon > 3.


def test_base_vector_gaussian():
    from opendp.meas import make_base_gaussian, make_base_analytic_gaussian
    from opendp.comb import make_fix_delta
    delta = .000001
    meas = make_fix_delta(make_base_gaussian(scale=10.5, D="VectorDomain<AllDomain<f64>>"), delta)
    print("base gaussian:", meas([80., 90., 100.]))
    assert meas.check(1., (0.6, delta))

    meas = make_fix_delta(make_base_analytic_gaussian(scale=10.5, D="VectorDomain<AllDomain<f64>>"), delta)
    print("base analytic gaussian:", meas([80., 90., 100.]))
    assert meas.check(1., (0.5, delta))


def test_base_geometric():
    from opendp.meas import make_base_geometric
    meas = make_base_geometric(scale=2., bounds=(1, 10), D=AllDomain[i32])
    print("base_geometric in constant time:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    meas = make_base_geometric(scale=2., D=AllDomain[i32])
    print("base_geometric:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_base_vector_geometric():
    from opendp.meas import make_base_geometric
    meas = make_base_geometric(scale=2., D=VectorDomain[AllDomain[i32]])
    print("vector base_geometric:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    meas = make_base_geometric(scale=2., bounds=(10, 100), D=VectorDomain[AllDomain[i32]])
    print("constant time vector base_geometric:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_make_count_by_ptr():
    from opendp.trans import make_count_by
    from opendp.meas import make_base_ptr
    from opendp.comb import make_fix_delta
    from opendp.typing import L1Distance

    meas = make_count_by(MO=L1Distance[float], TK=str, TV=float) \
           >> make_base_ptr(scale=2., threshold=28., TK=str)
    fixed_meas = make_fix_delta(meas, 1e-6)
    print("stability histogram:", fixed_meas(["CAT_A"] * 20 + ["CAT_B"] * 10))

    print(meas.map(1).epsilon(1e-6))
    assert fixed_meas.check(1, (1.0, 1e-6))

def test_randomized_response():
    from opendp.meas import make_randomized_response
    meas = make_randomized_response(categories=["A", "B", "C", "D"], prob=0.75)
    print("randomized response:", meas("A"))
    import math
    assert meas.check(1, math.log(9.))
    assert not meas.check(1, math.log(8.999))


def test_randomized_response_bool():
    from opendp.meas import make_randomized_response_bool
    meas = make_randomized_response_bool(prob=0.75)
    print("randomized response:", meas(True))
    import math
    assert meas.check(1, math.log(3.))
    assert not meas.check(1, math.log(2.999))
