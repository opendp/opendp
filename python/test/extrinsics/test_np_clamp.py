import sys
import opendp.prelude as dp
from opendp.extrinsics.make_np_clamp import then_np_clamp
import pytest

dp.enable_features("honest-but-curious", "contrib", "floating-point")


@pytest.mark.skipif('numpy' not in sys.modules, reason="Numpy needed")
def test_clamp():
    import numpy as np
    space = dp.np_array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, order=2)
    data = np.random.normal(size=(100_000, 10))
    assert trans.output_domain.member(trans(data))

