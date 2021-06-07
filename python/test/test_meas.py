

def test_geometric():
    from opendp.v1.meas import make_base_geometric
    base_geometric = make_base_geometric(scale=2., lower=0, upper=20)
    print("base_geometric:", base_geometric(100))
    assert base_geometric.check(1, 0.5)
    assert not base_geometric.check(1, 0.49999)


def test_gaussian():
    from opendp.v1.meas import make_base_gaussian
    base_gaussian = make_base_gaussian(scale=10.5)
    print("base gaussian:", base_gaussian(100.))
    assert base_gaussian.check(1., (1.3, .000001))


def test_laplace():
    from opendp.v1.meas import make_base_laplace
    base_laplace = make_base_laplace(scale=10.5)
    print("base laplace:", base_laplace(100.))
    assert base_laplace.check(1., 1.3)
