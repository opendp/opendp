import opendp.prelude as dp
from opendp.extras.numpy import _symmetric_matrix_domain
import pytest

np = pytest.importorskip("numpy")


def test_eigenvalues():
    from opendp.extras.sklearn._make_eigenvalues import (
        then_eigenvalues,
        then_private_eigenvalues,
    )

    space = (
        _symmetric_matrix_domain(num_features=4, T=float),
        dp.nuclear_distance(),
    )
    trans = space >> then_eigenvalues()
    data = np.random.normal(size=(4, 4))
    data += data.T
    assert np.array_equal(trans(data), np.linalg.eigvalsh(data))
    assert trans.map(1.0) == 1

    # also test the private constructor
    meas = space >> then_private_eigenvalues(dp.max_divergence(), scale=1.0)
    print("meas(data)", meas(data))
    assert meas.map(1.0) == 1.0


def test_eigenvalues_nuclear_lipschitz():
    from opendp.extras.sklearn._make_eigenvalues import make_eigenvalues

    domain = _symmetric_matrix_domain(num_features=2)
    trans = make_eigenvalues(domain, dp.nuclear_distance())
    a = np.diag([2.0, -3.0])
    b = np.diag([1.0, 1.0])
    nuclear_distance = np.linalg.svd(a - b, compute_uv=False).sum()
    eig_a = np.asarray(trans(a))
    eig_b = np.asarray(trans(b))
    assert np.linalg.norm(eig_a - eig_b, ord=1) <= nuclear_distance
    assert trans.map(nuclear_distance) == nuclear_distance


def test_eigenvalues_rejects_invalid_space():
    from opendp.extras.sklearn._make_eigenvalues import make_eigenvalues

    metric = dp.nuclear_distance()
    invalid = dp.numpy.array2_domain(num_columns=2, T=float)
    valid = _symmetric_matrix_domain(num_features=2)
    with pytest.raises(ValueError, match="NPSymmetricMatrixDomain"):
        make_eigenvalues(invalid, metric)
    with pytest.raises(ValueError, match="nuclear"):
        make_eigenvalues(valid, dp.l2_distance(T=float))
