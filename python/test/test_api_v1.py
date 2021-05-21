from opendp.v1.trans import make_count
from opendp.v1.meas import make_base_laplace, make_base_geometric
from opendp.v1.mod import FfiMeasurementPtr
from opendp.v1.typing import HammingDistance, L1Sensitivity


def test_laplace():
    data = [True, False, True, True, False]
    count = make_count(MI=HammingDistance, MO=L1Sensitivity[int], TI=bool)
    print(count(data))

    # base_laplace = make_base_laplace(scale=1.)
    # print(base_laplace(1.))

    base_geometric = make_base_geometric(scale=0.5, min=0, max=20)
    print(base_geometric(1))

    chain = count >> base_geometric
    print(chain.check(1, .5))
    print(chain(data))


test_laplace()
