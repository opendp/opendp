"""Tests for the internal nuclear-distance Wishart release."""

import pytest

import opendp.prelude as dp

np = pytest.importorskip("numpy")
from opendp.extras.numpy import _symmetric_matrix_domain
from opendp.extras.numpy._make_np_wishart import make_private_np_wishart

dp.enable_features("contrib", "idealized-numerics")


@pytest.mark.parametrize("d", [1, 3])
def test_wishart_matrix(d):
    m = make_private_np_wishart(
        _symmetric_matrix_domain(num_features=d), dp.nuclear_distance(), 0.5
    )
    assert m.map(0.0) == 0
    assert m.map(2.0) == 2
    result = m(np.eye(d))
    assert result.shape == (d, d)
    np.testing.assert_array_equal(result, result.T)
    assert np.isfinite(result).all()

