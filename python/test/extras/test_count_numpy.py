import opendp.prelude as dp
import pytest
from opendp.extras.numpy._make_np_count import then_np_count, then_private_np_count
from ..helpers import optional_dependency

np = pytest.importorskip("numpy")


def test_np_count():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_count()
    assert trans(np.zeros(1000)) == 1000
    assert trans.map(1) == 1


def test_private_np_count():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    meas = space >> then_private_np_count(dp.zero_concentrated_divergence(), scale=1.0)
    print("meas(np.zeros(1000))", meas(np.zeros(1000)))
    assert meas.map(1) == 0.5
