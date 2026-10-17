import opendp.prelude as dp
from opendp.extras.numpy import then_np_clamp
import pytest

np = pytest.importorskip("numpy")


def test_clamp():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    data = np.random.normal(size=(100_000, 10))
    assert trans.output_domain.member(trans(data))


def test_clamp_nan_inf():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    data = np.array([[np.inf, 0.0], [np.inf, np.inf], [1.0, 1.0], [np.nan, 0.0]])
    assert trans.output_domain.member(trans(data))


@pytest.mark.parametrize("num_columns", [None, 2])
def test_clamp_zero_length(num_columns):
    domain = dp.numpy.array2_domain(num_columns=num_columns, T=float)
    space = domain, dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    empty = np.zeros((0, 2))
    assert domain.member(empty)
    assert trans(empty).shape == (0, 2)
