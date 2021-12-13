from opendp.mod import enable_features

enable_features('floating-point', 'contrib')


def test_base_gaussian():
    from opendp.meas import make_base_gaussian, make_base_analytic_gaussian
    from opendp.mod import binary_search_param
    print("Analytic", binary_search_param(
        make_base_analytic_gaussian,
        d_in=1., d_out=(1., 1e-5)))
    print("Standard", binary_search_param(
        make_base_gaussian,
        d_in=1., d_out=(1., 1e-5)))


def test_base_laplace():
    from opendp.meas import make_base_laplace
    meas = make_base_laplace(scale=10.5)
    print("base laplace:", meas(100.))
    assert meas.check(1., 1.3)


def test_base_vector_laplace():
    from opendp.meas import make_base_laplace
    meas = make_base_laplace(scale=10.5, D="VectorDomain<AllDomain<f64>>")
    print("base laplace:", meas([80., 90., 100.]))
    assert meas.check(1., 1.3)


def test_base_analytic_gaussian():
    from opendp.meas import make_base_gaussian, make_base_analytic_gaussian
    meas = make_base_gaussian(scale=10.5)
    print("base gaussian:", meas(100.))
    assert meas.check(1., (1.3, .000001))

    meas = make_base_analytic_gaussian(scale=10.5)
    print("base analytic gaussian:", meas(100.))
    assert meas.check(1., (1.3, .000001))


def test_base_vector_gaussian():
    from opendp.meas import make_base_gaussian, make_base_analytic_gaussian
    meas = make_base_gaussian(scale=10.5, D="VectorDomain<AllDomain<f64>>")
    print("base gaussian:", meas([80., 90., 100.]))
    assert meas.check(1., (1.3, .000001))

    meas = make_base_analytic_gaussian(scale=10.5, D="VectorDomain<AllDomain<f64>>")
    print("base analytic gaussian:", meas([80., 90., 100.]))
    assert meas.check(1., (1.3, .000001))


def test_base_discrete_laplace():
    from opendp.meas import make_base_discrete_laplace
    meas = make_base_discrete_laplace(scale=2., bounds=(1, 10))
    print("base_discrete_laplace in constant time:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    meas = make_base_discrete_laplace(scale=2.)
    print("base_discrete_laplace:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_base_vector_discrete_laplace():
    from opendp.meas import make_base_discrete_laplace
    meas = make_base_discrete_laplace(scale=2., D="VectorDomain<AllDomain<i32>>")
    print("vector base_discrete_laplace:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    meas = make_base_discrete_laplace(scale=2., bounds=(10, 100), D="VectorDomain<AllDomain<i32>>")
    print("constant time vector base_discrete_laplace:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_make_count_by_ptr():
    from opendp.trans import make_count_by
    from opendp.meas import make_base_ptr
    from opendp.typing import L1Distance

    meas = make_count_by(MO=L1Distance[float], TK=str, TV=float) \
           >> make_base_ptr(scale=2., threshold=16., TK=str)
    print("stability histogram:", meas(["CAT_A"] * 20 + ["CAT_B"] * 10))
    assert meas.check(1, (1.0, 1e-6))


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
