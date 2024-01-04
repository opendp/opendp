import opendp.prelude as dp
from opendp.extrinsics.domains import _np_xTx_domain

dp.enable_features("honest-but-curious", "contrib", "floating-point")


def test_np_eigenvalues():
    import numpy as np
    from opendp.extrinsics._make_np_eigenvalues import then_np_eigenvalues, then_private_np_eigenvalues
    space = _np_xTx_domain(num_features=4, norm=1., ord=2, T=float), dp.symmetric_distance()
    trans = space >> then_np_eigenvalues()
    data = np.random.normal(size=(4, 4))
    data += data.T
    assert np.array_equal(trans(data), np.linalg.eigvalsh(data))
    assert trans.map(1) == 1


    # also test the private constructor
    trans = space >> then_private_np_eigenvalues(dp.zero_concentrated_divergence(T=float), scale=1.)
    print(trans(data))
    assert trans.map(1) == 0.5
