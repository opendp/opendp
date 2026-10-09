import opendp.prelude as dp
import pytest
from ..helpers import optional_dependency

np = pytest.importorskip("numpy")


def test_private_np_mean():
    from opendp.extras.numpy._make_np_mean import then_private_np_mean

    space = (
        dp.numpy.array2_domain(size=1000, nan=False, T=float),
        dp.symmetric_distance(),
    )
    trans = space >> then_private_np_mean(scale=0.001, norm=1.0)

    print(
        "trans(np.random.normal(size=(1000, 4)))",
        trans(np.random.normal(size=(1000, 4))),
    )
    assert trans.map(2) == 2
