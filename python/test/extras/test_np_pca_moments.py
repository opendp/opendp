"""Joint Gaussian and Wishart PCA moment calibration."""

import pytest
import opendp.prelude as dp

np = pytest.importorskip("numpy")

from opendp.extras.numpy._make_np_moments import (
    _make_gaussian_pca_moments,
    make_private_np_wishart_moments,
)

@pytest.mark.parametrize("size", [None, 1])
@pytest.mark.parametrize("dimension", [1, 3])
@pytest.mark.parametrize("estimate_mean", [False, True])
def test_trace_aware_moments(size, dimension, estimate_mean):
    dp.enable_features("contrib", "idealized-numerics")
    radius = 2.0
    domain = dp.numpy.array2_domain(
        T=float, size=size, num_columns=dimension, norm=radius, p=2, nan=False
    )
    t, decode = _make_gaussian_pca_moments(
        domain, dp.symmetric_distance(), estimate_mean
    )
    x = np.eye(dimension)[0:1] * radius
    if size is None:
        empty = np.asarray(t(np.empty((0, dimension))))
        for row in (x, np.zeros_like(x)):
            assert np.linalg.norm(np.asarray(t(row)) - empty) == pytest.approx(t.map(1))
            # Negative energy or deficit is clamped before deriving the count.
            _, gram, count = decode(-np.asarray(t(row)))
            assert count == 0
            assert np.trace(gram) == pytest.approx(0.0)
    else:
        # The fixed-size scalar case has its own sharp joint bound.
        neighbors = [np.zeros_like(x), -x]
        if dimension > 1:
            y = np.eye(dimension)[1:2] * radius
            if estimate_mean:
                y = np.sqrt(15) / 4 * y - x / 4
            neighbors.append(y)
        changes = [np.linalg.norm(np.asarray(t(x)) - np.asarray(t(y))) for y in neighbors]
        assert max(changes) <= t.map(2) * (1 + 1e-12)
        if dimension > 1 or not estimate_mean:
            assert max(changes) == pytest.approx(t.map(2))
        assert t.map(1) == 0.0
    step = 1 if size is None else 2
    assert t.map(0) == 0.0
    assert t.map(3 * step) == pytest.approx(3 * t.map(step))
    sums, gram, count = decode(t(x))
    assert count == 1
    np.testing.assert_allclose(gram, x.T @ x, atol=1e-12)
    if estimate_mean:
        np.testing.assert_allclose(sums, x.sum(axis=0))
    else:
        assert sums is None


@pytest.mark.parametrize("fixed", [False, True])
@pytest.mark.parametrize("estimate_mean", [False, True])
def test_wishart_joint_calibration(fixed, estimate_mean):
    dp.enable_features("contrib", "idealized-numerics")
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
