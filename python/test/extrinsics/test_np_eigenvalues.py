import sys
import opendp.prelude as dp
from opendp.extras.domains import _np_sscp_domain
import pytest

dp.enable_features("honest-but-curious", "contrib", "floating-point")


@pytest.mark.skipif("numpy" not in sys.modules, reason="Numpy needed")
def test_np_eigenvalues():
    import numpy as np
    from opendp.extras._make_np_eigenvalues import (
        then_np_eigenvalues,
        then_private_np_eigenvalues,
    )

    space = (
        _np_sscp_domain(num_features=4, norm=1.0, p=2, size=1000, T=float),
        dp.symmetric_distance(),
    )
    trans = space >> then_np_eigenvalues()
    data = np.random.normal(size=(4, 4))
    data += data.T
    assert np.array_equal(trans(data), np.linalg.eigvalsh(data))
    assert trans.map(1) == 1

    # also test the private constructor
    meas = space >> then_private_np_eigenvalues(dp.max_divergence(T=float), scale=1.0)
    print(meas(data))
    assert meas.map(1) == 1.0
