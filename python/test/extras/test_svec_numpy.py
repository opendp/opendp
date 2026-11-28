import pytest
import opendp.prelude as dp

np = pytest.importorskip("numpy")

from opendp.extras.numpy import _symmetric_matrix_domain
from opendp.extras.numpy._make_np_svec import make_np_svec, unsvec


def test_symmetric_matrix_coordinates():
    dp.enable_features("contrib", "idealized-numerics")
    matrix = np.array([[2.0, -3.0, 4.0], [-3.0, 5.0, 6.0], [4.0, 6.0, -7.0]])
    domain = _symmetric_matrix_domain(num_features=3)
    metric = dp.l2_distance(T=float)
    for p in (1, 2):
        transform = make_np_svec(domain, metric, p=p)
        values = np.asarray(transform(matrix))
        assert transform.output_domain.member(values)
        np.testing.assert_allclose(unsvec(values, 3), matrix)
        assert np.linalg.norm(values) == pytest.approx(np.linalg.norm(matrix, "fro"))
        assert transform.map(2.0) == pytest.approx(2.0 if p == 2 else 2.0 * np.sqrt(6))
