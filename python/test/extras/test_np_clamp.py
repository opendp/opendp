import opendp.prelude as dp
from opendp.extras.numpy import then_np_clamp
import pytest
from ..helpers import optional_dependency


def test_clamp():
    with optional_dependency("numpy"):
        space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    np = pytest.importorskip("numpy")
    data = np.random.normal(size=(100_000, 10))
    assert trans.output_domain.member(trans(data))


def test_clamp_nan_inf():
    with optional_dependency("numpy"):
        space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    np = pytest.importorskip("numpy")
    data = np.array([[np.inf, 0.0], [np.inf, np.inf], [1.0, 1.0], [np.nan, 0.0]])
    assert trans.output_domain.member(trans(data))
