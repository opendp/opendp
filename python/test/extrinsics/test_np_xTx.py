import sys
import opendp.prelude as dp
import pytest

dp.enable_features("honest-but-curious", "contrib", "floating-point")


@pytest.mark.skipif('numpy' not in sys.modules, reason="Numpy needed")
def test_np_sscp_sym():
    import numpy as np
    from opendp._extrinsics._make_np_sscp import then_np_sscp

    space = dp.np_array2_domain(num_columns=4, T=float), dp.symmetric_distance()
    trans = space >> then_np_sscp(dp.symmetric_distance())
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.T @ data)
    assert trans.map(1) == 1


@pytest.mark.skipif('numpy' not in sys.modules, reason="Numpy needed")
def test_np_sscp_l2():
    import numpy as np
    from opendp._extrinsics._make_np_sscp import then_np_sscp

    space = (
        dp.np_array2_domain(num_columns=4, norm=2.0, p=2, T=float),
        dp.symmetric_distance(),
    )
    trans = space >> then_np_sscp(dp.l2_distance(T=float))
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.T @ data)
    assert trans.map(2) == 8

    space = (
        dp.np_array2_domain(num_columns=4, norm=2.0, p=2, size=1000, T=float),
        dp.symmetric_distance(),
    )
    trans = space >> then_np_sscp(dp.l2_distance(T=float))
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.T @ data)
    assert trans.map(3) == 8
