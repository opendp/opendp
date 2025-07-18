import opendp.prelude as dp
from opendp.extras.numpy import _sscp_domain
import pytest



def test_eigenvalues():
    from opendp.extras.sklearn._make_eigenvalues import (
        then_eigenvalues,
        then_private_eigenvalues,
    )

    pytest.importorskip('numpy')
    print("A")
    space = (
        _sscp_domain(num_features=4, norm=1.0, p=2, size=1000, T=float),
        dp.symmetric_distance(),
    )
    print("B")
    trans = space >> then_eigenvalues()
    print("B.2")
    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(4, 4))
    data += data.T
    print("C")
    assert np.array_equal(trans(data), np.linalg.eigvalsh(data))
    print("D")
    assert trans.map(1) == 1

    # also test the private constructor
    meas = space >> then_private_eigenvalues(dp.max_divergence(), scale=1.0)
    print("meas(data)", meas(data))
    assert meas.map(1) == 1.0

dp.enable_features("contrib", "honest-but-curious", "floating-point")
test_eigenvalues()