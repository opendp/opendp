import pytest
from opendp import prelude as dp

np = pytest.importorskip("numpy")


def test_np_array_postprocessor():
    fun = dp.as_array()

    result = fun(np.array([1, 2, 3], dtype=np.int32))

    assert isinstance(result, np.ndarray)
    assert np.array_equal(result, np.array([1, 2, 3], dtype=np.int32))
