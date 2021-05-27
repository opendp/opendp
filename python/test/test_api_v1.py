from opendp.v1.trans import make_count
from opendp.v1.meas import make_base_laplace, make_base_geometric
from opendp.v1.typing import HammingDistance, L1Sensitivity


def test_geometric():
    base_geometric = make_base_geometric(scale=1.5, min=0, max=20)
    # # FIXME
    print("base_geometric:", base_geometric(100))


def test_base():
    data = [True, False, True, True, False]
    count = make_count(MI=HammingDistance, MO=L1Sensitivity[int], TI=bool)
    print("count:", count(data))

    base_laplace = make_base_laplace(scale=1.)
    print("base laplace:", base_laplace(10.))

    base_geometric = make_base_geometric(scale=0.5, min=0, max=20)
    # # FIXME
    # print("base_geometric:", base_geometric(1))

    chain = count >> base_geometric
    # print("chained measurement check:", chain.check(d_in=1, d_out=1000., debug=True))

    print("evaluate chain:", chain(data))


test_base()
test_geometric()
