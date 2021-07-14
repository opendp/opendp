

def test_base_laplace():
    from opendp.v1.meas import make_base_laplace
    meas = make_base_laplace(scale=10.5)
    print("base laplace:", meas(100.))
    assert meas.check(1., 1.3)


def test_base_vector_laplace():
    from opendp.v1.meas import make_base_laplace
    meas = make_base_laplace(scale=10.5, D="VectorDomain<AllDomain<f64>>")
    print("base laplace:", meas([80., 90., 100.]))
    assert meas.check(1., 1.3)


def test_base_gaussian():
    from opendp.v1.meas import make_base_gaussian
    meas = make_base_gaussian(scale=10.5)
    print("base gaussian:", meas(100.))
    assert meas.check(1., (1.3, .000001))


def test_base_vector_gaussian():
    from opendp.v1.meas import make_base_gaussian
    meas = make_base_gaussian(scale=10.5, D="VectorDomain<AllDomain<f64>>")
    print("base gaussian:", meas([80., 90., 100.]))
    assert meas.check(1., (1.3, .000001))


def test_base_geometric():
    from opendp.v1.meas import make_base_geometric
    meas = make_base_geometric(scale=2.)
    print("base_geometric in constant time:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    meas = make_base_geometric(scale=2.)
    print("base_geometric:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_base_vector_geometric():
    from opendp.v1.meas import make_base_geometric
    meas = make_base_geometric(scale=2., D="VectorDomain<AllDomain<i32>>")
    print("vector base_geometric:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    meas = make_base_geometric(scale=2., D="VectorDomain<AllDomain<i32>>")
    print("constant time vector base_geometric:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_basic_composition_multi():
    from opendp.v1.core import make_basic_composition_multi
    from opendp.v1.meas import make_base_geometric
    composed = make_basic_composition_multi([
        make_base_geometric(scale=2.),
        make_base_geometric(scale=2.)
    ])

    print(composed.check(1, 2.))

test_basic_composition_multi()

# TODO: data unloader for hashmaps
# def test_base_stability():
#     from opendp.v1.trans import make_count_by
#     from opendp.v1.meas import make_base_stability
#     meas = (
#         make_count_by(n=10, MI=SubstituteDistance, MO=L1Distance[float], TI=int) >>
#         make_base_stability(n=10, scale=20., threshold=1., MI=L1Distance[float], TIK=int)
#     )
#     print("base gaussian:", meas([3] * 4 + [5] * 6))
#     assert meas.check(1., (1.3, .000001))
