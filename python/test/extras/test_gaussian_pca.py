import pytest
import opendp.prelude as dp

np = pytest.importorskip("numpy")
pytest.importorskip("sklearn")

from opendp.extras.numpy import _symmetric_matrix_domain
from opendp.extras.numpy._make_np_svec import make_np_svec, unsvec
from opendp.extras.sklearn import decomposition


def test_symmetric_matrix_coordinates():
    dp.enable_features("contrib", "idealized-numerics")
    matrix = np.array([[2., -3., 4.], [-3., 5., 6.], [4., 6., -7.]])
    domain = _symmetric_matrix_domain(num_features=3)
    metric = dp.l2_distance(T=float)
    for p in (1, 2):
        transform = make_np_svec(domain, metric, p=p)
        values = np.asarray(transform(matrix))
        assert transform.output_domain.member(values)
        np.testing.assert_allclose(unsvec(values, 3), matrix)
        assert np.linalg.norm(values) == pytest.approx(np.linalg.norm(matrix, "fro"))
        assert transform.map(2.) == pytest.approx(2. if p == 2 else 2. * np.sqrt(6))


@pytest.mark.parametrize("selection,size,width,expected", [
    (1, 10, 4, True),
    (2, 10, 4, False),
    (None, 2, 8, True),
    (None, None, 8, False),
    (None, 10, 8, False),
    (.5, 10, 8, False),
    ("mle", 10, 8, False),
    (1, 10, 1, False),
])
def test_public_solver_policy(selection, size, width, expected):
    assert decomposition._use_bingham(selection, size, width) == expected


@pytest.mark.parametrize("is_zcdp", [False, True])
@pytest.mark.parametrize("k", [1, 2])
@pytest.mark.parametrize("center", ["origin", None])
def test_solver_dispatch_and_coherent_spectrum(monkeypatch, is_zcdp, k, center):
    dp.enable_features("contrib", "idealized-numerics")
    data = np.array([[1., 0., 0., 0.], [-1., 0., 0., 0.],
                     [0., .5, 0., 0.], [0., -.5, 0., 0.]])
    domain = dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=4, norm=1., p=2, nan=False
    )
    seen = []
    original = decomposition.make_np_svec

    def record(*args, **kwargs):
        seen.append(kwargs["p"])
        return original(*args, **kwargs)

    monkeypatch.setattr(decomposition, "make_np_svec", record)
    measure = dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence()
    # Large signal relative to noise for SSCP; Bingham is only tested for
    # dispatch and orthogonality, at an ordinary budget.
    budget = 1e12 if is_zcdp else 1e6 if k == 2 else 1.
    measurement = decomposition.make_private_pca(
        domain, dp.symmetric_distance(), measure,
        2, budget, center=center, num_components=k
    )
    assert measurement.check(2, budget)
    assert measurement.map(1) == 0.
    release = measurement(data)
    assert seen == ([2] if is_zcdp and center == "origin" else [1] if not is_zcdp and k == 2 else [])
    assert release.components.shape == (k, 4)
    np.testing.assert_allclose(release.components @ release.components.T, np.eye(k), atol=1e-12)
    if is_zcdp or k == 2:
        np.testing.assert_allclose(release.mean, np.zeros(4), atol=1e-3)
        np.testing.assert_allclose(release.singular_values[:2], [np.sqrt(2), np.sqrt(.5)], atol=1e-3)
        np.testing.assert_allclose(np.abs(release.components), np.eye(4)[:k], atol=1e-3)

def test_weighted_moment_sensitivity():
    from opendp.extras.numpy._make_np_moments import _moment_sensitivity_sq

    for ratio in (1., np.sqrt(2.), 3.):
        # Attain the bound with unit rows at the maximizing angle.
        t = 1 / (2 * ratio**2)
        x = np.array([1., 0.])
        y = np.array([-t, np.sqrt(1 - t*t)])
        diff = np.linalg.norm(np.outer(x, x) - np.outer(y, y), "fro")**2
        diff += np.linalg.norm(x-y)**2 / ratio**2
        assert diff == pytest.approx(_moment_sensitivity_sq(ratio, True))


@pytest.mark.parametrize("size", [None, 4])
def test_joint_moments_centered_pca(size):
    dp.enable_features("contrib", "idealized-numerics")
    data = np.array([[.6, .4], [.6, -.4], [.6, .4], [.6, -.4]])
    domain = dp.numpy.array2_domain(
        T=float, size=size, num_columns=2, norm=1., p=2, nan=False
    )
    distance = 1 if size is None else 2
    m = decomposition.make_private_pca(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(),
        distance, 1e12, num_components=2,
    )
    assert m.check(distance, 1e12)
    release = m(data)
    assert release.n_samples == 4
    np.testing.assert_allclose(release.mean, [.6, 0.], atol=1e-4)
    np.testing.assert_allclose(release.singular_values[0], .8, atol=1e-4)
    np.testing.assert_allclose(np.abs(release.components[0]), [0., 1.], atol=1e-4)

def test_trace_bounded_spectrum():
    # Known count bounds total clipped scatter, not individual directions.
    raw = np.array([-4., 1., 3., 8.])
    projected = decomposition._project_spectrum(raw, 5.)
    np.testing.assert_allclose(projected, [0., 0., 0., 5.])
    np.testing.assert_allclose(
        decomposition._project_spectrum(raw, None), [0., 1., 3., 8.]
    )
    np.testing.assert_allclose(
        decomposition._project_spectrum(raw, 20.), [0., 1., 3., 8.]
    )
    # Exercise a face with more than one retained eigenvalue.
    np.testing.assert_allclose(
        decomposition._project_spectrum(raw, 7.), [0., 0., 1., 6.]
    )


@pytest.mark.parametrize("is_zcdp", [False, True])
def test_matrix_pca_respects_public_trace_bound(is_zcdp):
    dp.enable_features("contrib", "idealized-numerics")
    data = np.eye(4)
    domain = dp.numpy.array2_domain(
        T=float, size=4, num_columns=4, norm=1., p=2, nan=False
    )
    measure = dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence()
    m = decomposition.make_private_pca(
        domain, dp.symmetric_distance(), measure, 2, 0.01, center="origin"
    )
    release = m(data)
    assert m.check(2, 0.01)
    assert np.sum(release.singular_values**2) <= 4. + 1e-10

@pytest.mark.parametrize("size", [None, 4])
def test_joint_pca_anchors_spectrum_to_released_mean_and_count(size):
    dp.enable_features("contrib", "idealized-numerics")

    data = np.tile([.5, .0], (4, 1))
    domain = dp.numpy.array2_domain(
        T=float, size=size, num_columns=2, norm=1., p=2, nan=False
    )
    distance = 1 if size is None else 2
    m = decomposition.make_private_pca(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(),
        distance, .1, num_components=2,
    )
    assert m.check(distance, .1)
    release = m(data)
    assert np.linalg.norm(release.mean) <= 1. + 1e-12
    bound = release.n_samples * max(0., 1.-release.mean @ release.mean)
    assert np.sum(release.singular_values**2) <= bound + 1e-10
