import opendp.prelude as dp
import pytest
from ..helpers import optional_dependency

np = pytest.importorskip("numpy")


def test_np_sum():
    from opendp.extras.numpy._make_np_sum import then_np_sum

    # unsized data
    space = (
        dp.numpy.array2_domain(norm=1.0, p=2, nan=False, T=float),
        dp.symmetric_distance(),
    )
    trans = space >> then_np_sum()
    assert trans.map(1) == 1

    # sized data
    space = (
        dp.numpy.array2_domain(norm=1.0, p=2, size=1000, nan=False, T=float),
        dp.symmetric_distance(),
    )
    trans = space >> then_np_sum()
    assert trans.map(2) == 2.0

    # function
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.sum(axis=0))


def test_private_np_sum():
    from opendp.extras.numpy._make_np_sum import then_private_np_sum

    space = (
        dp.numpy.array2_domain(norm=1.0, p=2, nan=False, T=float),
        dp.symmetric_distance(),
    )
    meas = space >> then_private_np_sum(dp.zero_concentrated_divergence(), scale=1.0)
    data = np.random.normal(size=(1000, 4))
    print("meas(data)", meas(data))
    assert meas.map(1) == 0.5
