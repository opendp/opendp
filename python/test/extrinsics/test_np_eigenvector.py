import sys
import opendp.prelude as dp
from opendp._extrinsics.domains import _np_SSCP_domain
import pytest

dp.enable_features("honest-but-curious", "contrib", "floating-point")


@pytest.mark.skipif('numpy' not in sys.modules, reason="Numpy needed")
def test_private_np_eigenvector():
    import numpy as np
    from opendp._extrinsics._make_np_eigenvector import then_private_eigenvector

    space = (
        _np_SSCP_domain(num_features=4, norm=1.0, p=2, T=float),
        dp.symmetric_distance(),
    )
    meas = space >> then_private_eigenvector(unit_epsilon=100_000.0)
    data = np.random.normal(size=(4, 4))
    data += data.T
    noisy = meas(data)
    exact = np.linalg.eigh(data)[1][:, -1]

    # normalize sign
    noisy = np.copysign(noisy, exact)

    assert np.allclose(noisy, exact, atol=0.3)
    assert meas.map(2) == 100_000.0


@pytest.mark.skipif('numpy' not in sys.modules, reason="Numpy needed")
def test_eigenvector_integration():
    import numpy as np
    from opendp._extrinsics.make_np_clamp import then_np_clamp
    from opendp._extrinsics._make_np_xTx import then_np_xTx
    from opendp._extrinsics._make_np_eigenvector import then_private_eigenvector

    num_columns = 4
    space = (
        dp.np_array2_domain(num_columns=num_columns, T=float),
        dp.symmetric_distance(),
    )
    meas = (
        space
        >> then_np_clamp(norm=4.0, p=2)
        >> then_np_xTx(dp.symmetric_distance())
        >> then_private_eigenvector(1.0)
    )

    data = np.random.normal(size=(1000, num_columns))
    print(meas(data))


@pytest.mark.skipif('scipy' not in sys.modules, reason="Scipy needed")
def test_eigenvectors():
    import numpy as np
    from opendp._extrinsics.make_np_clamp import then_np_clamp
    from opendp._extrinsics._make_np_xTx import then_np_xTx
    from opendp._extrinsics._make_np_eigenvector import then_private_np_eigenvectors

    num_columns = 4
    space = (
        dp.np_array2_domain(num_columns=num_columns, T=float),
        dp.symmetric_distance(),
    )
    meas = (
        space
        >> then_np_clamp(norm=4.0, p=2)
        >> then_np_xTx(dp.symmetric_distance())
        >> then_private_np_eigenvectors([1.0] * 3)
    )
    data = np.random.normal(size=(1000, num_columns))
    print(meas(data))
