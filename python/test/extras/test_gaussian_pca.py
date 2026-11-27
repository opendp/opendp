from math import sqrt

import pytest
import opendp.prelude as dp

np = pytest.importorskip("numpy")
pytest.importorskip("sklearn")

from opendp.extras.numpy import _symmetric_matrix_domain
from opendp.extras.numpy._make_np_svec import make_np_svec, unsvec
from opendp.extras.sklearn.decomposition import make_private_pca, PCA


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
        assert transform.map(2.) == pytest.approx(2. if p == 2 else 2. * sqrt(6))


@pytest.mark.parametrize("center", ["origin", None])
def test_gaussian_pca_coherent_spectrum(center):
    dp.enable_features("contrib", "idealized-numerics")
    data = np.array([[1., 0., 0.], [-1., 0., 0.], [0., .5, 0.], [0., -.5, 0.]])
    domain = dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=3, norm=1., p=2, nan=False
    )
    measurement = make_private_pca(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(),
        2, 1e12, center=center, num_components=2
    )
    assert measurement.check(2, 1e12)
    release = measurement(data)
    model = PCA(n_components=2, center=center)
    model._ingest_release(release)
    np.testing.assert_allclose(model.components_ @ model.components_.T, np.eye(2), atol=1e-12)
    np.testing.assert_allclose(model.mean_, np.zeros(3), atol=1e-4)
    np.testing.assert_allclose(model.singular_values_, [sqrt(2), sqrt(.5)], atol=1e-4)
    np.testing.assert_allclose(np.abs(model.components_), [[1, 0, 0], [0, 1, 0]], atol=1e-4)
