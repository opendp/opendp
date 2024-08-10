import opendp.prelude as dp
from opendp.extras.numpy import _sscp_domain
import pytest
from ..helpers import optional_dependency



def test_eigenvalues():
    from opendp.extras.sklearn._make_eigenvalues import (
        then_eigenvalues,
        then_private_eigenvalues,
    )

    with optional_dependency('numpy'):
        space = (
            _sscp_domain(num_features=4, norm=1.0, p=2, size=1000, T=float),
            dp.symmetric_distance(),
        )
    trans = space >> then_eigenvalues()
    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(4, 4))
    data += data.T
    assert np.array_equal(trans(data), np.linalg.eigvalsh(data))
    assert trans.map(1) == 1

    # also test the private constructor
    meas = space >> then_private_eigenvalues(dp.max_divergence(T=float), scale=1.0)
    print(meas(data))
    assert meas.map(1) == 1.0
