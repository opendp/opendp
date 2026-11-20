import opendp.prelude as dp
from opendp.extras.numpy import _nuclear_norm_distance, _symmetric_matrix_domain
import pytest

np = pytest.importorskip("numpy")


def test_eigenvalues():
    from opendp.extras.sklearn._make_eigenvalues import (
        then_eigenvalues,
        then_private_eigenvalues,
    )

    space = (
        _symmetric_matrix_domain(num_features=4, T=float),
        _nuclear_norm_distance(),
    )
    trans = space >> then_eigenvalues()
    data = np.random.normal(size=(4, 4))
    data += data.T
    assert np.array_equal(trans(data), np.linalg.eigvalsh(data))
    assert trans.map(1) == 1

    # also test the private constructor
    meas = space >> then_private_eigenvalues(dp.max_divergence(), scale=1.0)
    print("meas(data)", meas(data))
    assert meas.map(1) == 1.0


def test_eigenvalues_nuclear_lipschitz():
    from opendp.extras.sklearn._make_eigenvalues import make_eigenvalues

    domain = _symmetric_matrix_domain(num_features=2)
    trans = make_eigenvalues(domain, _nuclear_norm_distance())
    a = np.diag([2.0, -3.0])
    b = np.diag([1.0, 1.0])
    nuclear_distance = np.linalg.svd(a - b, compute_uv=False).sum()
    assert np.linalg.norm(trans(a) - trans(b), ord=1) <= nuclear_distance
    assert trans.map(nuclear_distance) == nuclear_distance
