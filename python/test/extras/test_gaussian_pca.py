import pytest
import opendp.prelude as dp

np = pytest.importorskip("numpy")
pytest.importorskip("sklearn")

from opendp.extras.sklearn import decomposition2 as decomposition


@pytest.mark.parametrize("is_zcdp", [False, True])
@pytest.mark.parametrize("k", [1, 2])
@pytest.mark.parametrize("center", ["origin", None])
def test_sscp_coherent_spectrum(monkeypatch, is_zcdp, k, center):
    dp.enable_features("contrib", "idealized-numerics")
    data = np.array(
        [
            [1.0, 0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, 0.5, 0.0, 0.0],
            [0.0, -0.5, 0.0, 0.0],
        ]
    )
    domain = dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=4, norm=1.0, p=2, nan=False
    )
    seen = []
    original = decomposition.make_np_svec

    def record(*args, **kwargs):
        seen.append(kwargs["p"])
        return original(*args, **kwargs)

    monkeypatch.setattr(decomposition, "make_np_svec", record)
    measure = dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence()
    budget = 1e12 if is_zcdp else 1e6
    measurement = decomposition.make_private_pca(
        domain,
        dp.symmetric_distance(),
        measure,
        2,
        budget,
        center=center,
        num_components=k,
    )
    assert measurement.check(2, budget)
    assert measurement.map(1) == 0.0
    release = measurement(data)
    assert seen == ([2] if is_zcdp else [])
    assert release.components.shape == (k, 4)
    np.testing.assert_allclose(
        release.components @ release.components.T, np.eye(k), atol=1e-12
    )
    np.testing.assert_allclose(release.mean, np.zeros(4), atol=1e-3)
    np.testing.assert_allclose(
        release.singular_values[:2], [np.sqrt(2), np.sqrt(0.5)], atol=1e-3
    )
    np.testing.assert_allclose(np.abs(release.components), np.eye(4)[:k], atol=1e-3)


@pytest.mark.parametrize("size", [None, 1])
@pytest.mark.parametrize("dimension", [1, 3])
@pytest.mark.parametrize("estimate_mean", [False, True])
def test_trace_aware_moments(size, dimension, estimate_mean):
    dp.enable_features("contrib", "idealized-numerics")
    radius = 2.0
    domain = dp.numpy.array2_domain(
        T=float, size=size, num_columns=dimension, norm=radius, p=2, nan=False
    )
    t, decode = decomposition._make_gaussian_pca_moments(
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


@pytest.mark.parametrize("size", [None, 4])
@pytest.mark.parametrize("is_zcdp", [False, True])
def test_joint_moments_centered_pca(size, is_zcdp):
    dp.enable_features("contrib", "idealized-numerics")
    data = np.array([[0.6, 0.4], [0.6, -0.4], [0.6, 0.4], [0.6, -0.4]])
    domain = dp.numpy.array2_domain(
        T=float, size=size, num_columns=2, norm=1.0, p=2, nan=False
    )
    distance = 1 if size is None else 2
    measure = dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence()
    m = decomposition.make_private_pca(
        domain,
        dp.symmetric_distance(),
        measure,
        distance,
        1e12,
        num_components=2,
    )
    assert m.check(distance, 1e12)
    release = m(data)
    assert release.n_samples == 4
    np.testing.assert_allclose(release.mean, [0.6, 0.0], atol=1e-4)
    np.testing.assert_allclose(release.singular_values[0], 0.8, atol=1e-4)
    np.testing.assert_allclose(np.abs(release.components[0]), [0.0, 1.0], atol=1e-4)


def test_trace_bounded_spectrum():
    # Known count bounds total clipped scatter, not individual directions.
    raw = np.array([-4.0, 1.0, 3.0, 8.0])
    projected = decomposition._project_spectrum(raw, 5.0)
    np.testing.assert_allclose(projected, [0.0, 0.0, 0.0, 5.0])
    np.testing.assert_allclose(
        decomposition._project_spectrum(raw, None), [0.0, 1.0, 3.0, 8.0]
    )
    np.testing.assert_allclose(
        decomposition._project_spectrum(raw, 20.0), [0.0, 1.0, 3.0, 8.0]
    )
    # Exercise a face with more than one retained eigenvalue.
    np.testing.assert_allclose(
        decomposition._project_spectrum(raw, 7.0), [0.0, 0.0, 1.0, 6.0]
    )


@pytest.mark.parametrize("is_zcdp", [False, True])
def test_matrix_pca_respects_public_trace_bound(is_zcdp):
    dp.enable_features("contrib", "idealized-numerics")
    data = np.eye(4)
    domain = dp.numpy.array2_domain(
        T=float, size=4, num_columns=4, norm=1.0, p=2, nan=False
    )
    measure = dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence()
    m = decomposition.make_private_pca(
        domain, dp.symmetric_distance(), measure, 2, 0.01, center="origin"
    )
    release = m(data)
    assert m.check(2, 0.01)
    assert np.sum(release.singular_values**2) <= 4.0 + 1e-10


@pytest.mark.parametrize("size", [None, 4])
@pytest.mark.parametrize("is_zcdp", [False, True])
def test_joint_pca_anchors_spectrum_to_released_mean_and_count(size, is_zcdp):
    dp.enable_features("contrib", "idealized-numerics")

    data = np.tile([0.5, 0.0], (4, 1))
    domain = dp.numpy.array2_domain(
        T=float, size=size, num_columns=2, norm=1.0, p=2, nan=False
    )
    distance = 1 if size is None else 2
    measure = dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence()
    m = decomposition.make_private_pca(
        domain,
        dp.symmetric_distance(),
        measure,
        distance,
        0.1,
        num_components=2,
    )
    assert m.check(distance, 0.1)
    release = m(data)
    assert np.linalg.norm(release.mean) <= 1.0 + 1e-12
    bound = release.n_samples * max(0.0, 1.0 - release.mean @ release.mean)
    assert np.sum(release.singular_values**2) <= bound + 1e-10
