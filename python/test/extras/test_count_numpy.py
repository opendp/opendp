import opendp.prelude as dp
import pytest
from opendp.extras.numpy._make_np_count import then_np_count, then_private_np_count
from ..helpers import optional_dependency

np = pytest.importorskip("numpy")


def test_np_count():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_count()
    assert trans(np.zeros((1000, 1))) == 1000
    assert trans.map(1) == 1


def test_private_np_count():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    meas = space >> then_private_np_count(dp.zero_concentrated_divergence(), scale=1.0)
    print("meas(np.zeros((1000, 1)))", meas(np.zeros((1000, 1))))
    assert meas.map(1) == 0.5


def test_np_count_supports_int64_rows():
    domain = dp.numpy.array2_domain(T=float, num_columns=0)
    trans = (domain, dp.symmetric_distance()) >> then_np_count()
    assert trans(np.empty((2**31, 0))) == 2**31
