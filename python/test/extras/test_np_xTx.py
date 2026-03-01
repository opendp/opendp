import opendp.prelude as dp
import pytest
from ..helpers import optional_dependency



def test_np_sscp_sym():
    from opendp.extras.numpy._make_np_sscp import then_np_sscp

    with optional_dependency('numpy'):
        space = dp.numpy.array2_domain(num_columns=4, T=float, nan=False), dp.symmetric_distance()
    trans = space >> then_np_sscp()
    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.T @ data)
    assert trans.map(1) == 1
