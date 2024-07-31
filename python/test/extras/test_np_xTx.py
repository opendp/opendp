import opendp.prelude as dp
import pytest
from ..helpers import optional_dependency



def test_np_sscp_sym():
    from opendp.extras.numpy._make_np_sscp import then_np_sscp

    with optional_dependency('numpy'):
        space = dp.numpy.array2_domain(num_columns=4, T=float), dp.symmetric_distance()
    trans = space >> then_np_sscp(dp.symmetric_distance())
    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.T @ data)
    assert trans.map(1) == 1


def test_np_sscp_l2():
    from opendp.extras.numpy._make_np_sscp import then_np_sscp

    with optional_dependency('numpy'):
        space = (
            dp.numpy.array2_domain(num_columns=4, norm=2.0, p=2, T=float),
            dp.symmetric_distance(),
        )
    trans = space >> then_np_sscp(dp.l2_distance(T=float))
    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.T @ data)
    assert trans.map(2) == 8

    space = (
        dp.numpy.array2_domain(num_columns=4, norm=2.0, p=2, size=1000, T=float),
        dp.symmetric_distance(),
    )
    trans = space >> then_np_sscp(dp.l2_distance(T=float))
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.T @ data)
    assert trans.map(3) == 8
