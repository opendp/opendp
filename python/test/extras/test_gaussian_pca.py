import pytest
import opendp.prelude as dp

np = pytest.importorskip("numpy")
pytest.importorskip("sklearn")

from opendp.extras.sklearn import decomposition
from opendp.extras.numpy import _make_np_moments as moments


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
    original = moments.make_np_svec

    def record(*args, **kwargs):
        seen.append(kwargs["p"])
        return original(*args, **kwargs)

    monkeypatch.setattr(moments, "make_np_svec", record)
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
