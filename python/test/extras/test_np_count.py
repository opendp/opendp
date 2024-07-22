import opendp.prelude as dp
import pytest
from ..helpers import optional_dependency

dp.enable_features("honest-but-curious", "contrib", "floating-point")


def test_np_count():
    from opendp.extras.numpy._make_np_count import then_np_count
    with optional_dependency('numpy'):
        space = dp.numpy.np_array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_count()
    np = pytest.importorskip('numpy')
    assert trans(np.zeros(1000)) == 1000
    assert trans.map(1) == 1


def test_private_np_count():
    from opendp.extras.numpy._make_np_count import then_private_np_count
    with optional_dependency('numpy'):
        space = dp.numpy.np_array2_domain(T=float), dp.symmetric_distance()
    meas = space >> then_private_np_count(dp.zero_concentrated_divergence(T=float), scale=1.)
    np = pytest.importorskip('numpy')
    print(meas(np.zeros(1000)))
    assert meas.map(1) == 0.5
