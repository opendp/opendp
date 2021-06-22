

def test_base_laplace():
    from opendp.v1.meas import make_base_laplace
    meas = make_base_laplace(scale=10.5)
    print("base laplace:", meas(100.))
    assert meas.check(1., 1.3)


def test_base_vector_laplace():
    from opendp.v1.meas import make_base_vector_laplace
    meas = make_base_vector_laplace(scale=10.5)
    print("base laplace:", meas([80., 90., 100.]))
    assert meas.check(1., 1.3)


def test_base_gaussian():
    from opendp.v1.meas import make_base_gaussian
    meas = make_base_gaussian(scale=10.5)
    print("base gaussian:", meas(100.))
    assert meas.check(1., (1.3, .000001))


def test_base_vector_gaussian():
    from opendp.v1.meas import make_base_vector_gaussian
    meas = make_base_vector_gaussian(scale=10.5)
    print("base gaussian:", meas([80., 90., 100.]))
    assert meas.check(1., (1.3, .000001))


def test_base_geometric():
    from opendp.v1.meas import make_base_geometric
    meas = make_base_geometric(scale=2.)
    print("base_geometric:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    from opendp.v1.meas import make_constant_time_base_geometric
    meas = make_constant_time_base_geometric(scale=2., lower=0, upper=20)
    print("base_geometric:", meas(100))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


def test_base_vector_geometric():
    from opendp.v1.meas import make_base_vector_geometric
    meas = make_base_vector_geometric(scale=2.)
    print("base_geometric:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)

    from opendp.v1.meas import make_constant_time_base_vector_geometric
    meas = make_constant_time_base_vector_geometric(scale=2., lower=0, upper=20)
    print("base_geometric:", meas([100, 10, 12]))
    assert meas.check(1, 0.5)
    assert not meas.check(1, 0.49999)


# TODO: data unloader for hashmaps
# def test_base_stability():
#     from opendp.v1.trans import make_count_by
#     from opendp.v1.meas import make_base_stability
#     meas = (
#         make_count_by(n=10, MI=HammingDistance, MO=L1Distance[float], TI=int) >>
#         make_base_stability(n=10, scale=20., threshold=1., MI=L1Distance[float], TIK=int)
#     )
#     print("base gaussian:", meas([3] * 4 + [5] * 6))
#     assert meas.check(1., (1.3, .000001))
