import sys
import opendp.prelude as dp
import pytest

dp.enable_features("honest-but-curious", "contrib", "floating-point")


@pytest.mark.skipif('numpy' not in sys.modules, reason="Numpy needed")
def test_np_count():
    import numpy as np
    from opendp.extras._make_np_count import then_np_count
    space = dp.x.np_array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_count()
    assert trans(np.zeros(1000)) == 1000
    assert trans.map(1) == 1


@pytest.mark.skipif('numpy' not in sys.modules, reason="Numpy needed")
def test_private_np_count():
    import numpy as np
    from opendp.extras._make_np_count import then_private_np_count
    space = dp.x.np_array2_domain(T=float), dp.symmetric_distance()
    meas = space >> then_private_np_count(dp.zero_concentrated_divergence(T=float), scale=1.)
    print(meas(np.zeros(1000)))
    assert meas.map(1) == 0.5
