"""Tests for the internal nuclear-distance Wishart release."""

import pytest

import opendp.prelude as dp

np = pytest.importorskip("numpy")
from opendp.extras.numpy import _symmetric_matrix_domain
from opendp.extras.numpy._make_np_wishart import (
    make_private_np_wishart,
    make_private_np_wishart_moments,
)

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


@pytest.mark.parametrize("fixed", [False, True])
@pytest.mark.parametrize("estimate_mean", [False, True])
def test_wishart_joint_calibration(fixed, estimate_mean):
    domain = dp.numpy.array2_domain(
        T=float, size=2 if fixed else None, num_columns=3, norm=2.0, p=2, nan=False
    )
    edits = 2 if fixed else 1
    m, q = make_private_np_wishart_moments(
        domain, dp.symmetric_distance(), edits, 0.3, estimate_mean=estimate_mean
    )
    assert m.map(0) == 0
    assert m.check(edits, 0.3)
    assert m.map(2 * edits) == pytest.approx(0.6)
    if fixed:
        assert m.map(1) == 0
    sums, gram, count = m(np.zeros((2, 3)))
    assert gram.shape == (3, 3)
    assert (sums is None) == (not estimate_mean)
    if fixed:
        assert count == 2
    else:
        assert isinstance(count, int)
    assert np.trace(gram) >= -1e-10
    np.testing.assert_array_equal(gram, gram.T)
    assert q > 0
