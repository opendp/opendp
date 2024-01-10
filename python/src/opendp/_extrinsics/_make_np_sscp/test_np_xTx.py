import opendp.prelude as dp

dp.enable_features("honest-but-curious", "contrib", "floating-point")


def test_np_cov():
    import numpy as np
    from opendp.extrinsics._make_np_xTx import then_np_xTx
    space = dp.np_array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_xTx()
    assert trans(np.zeros(1000)) == 1000
    assert trans.map(1) == 1
