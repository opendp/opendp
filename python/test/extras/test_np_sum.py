import opendp.prelude as dp
import pytest
from ..helpers import optional_dependency



def test_np_sum():
    from opendp.extras.numpy._make_np_sum import then_np_sum

    # unsized data
    with optional_dependency('numpy'):
        space = dp.numpy.array2_domain(norm=1., p=2, T=float), dp.symmetric_distance()
    trans = space >> then_np_sum()
    assert trans.map(1) == 1

    # sized data
    space = dp.numpy.array2_domain(norm=1., p=2, size=1000, T=float), dp.symmetric_distance()
    trans = space >> then_np_sum()
    assert trans.map(2) == 2.

    # function
    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.sum(axis=0))


def test_private_np_sum():
    from opendp.extras.numpy._make_np_sum import then_private_np_sum
    with optional_dependency('numpy'):
        space = dp.numpy.array2_domain(norm=1., p=2, T=float), dp.symmetric_distance()
    meas = space >> then_private_np_sum(dp.zero_concentrated_divergence(), scale=1.)
    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(1000, 4))
    print(meas(data))
    assert meas.map(1) == 0.5
