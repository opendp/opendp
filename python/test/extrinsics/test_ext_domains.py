import sys
from opendp.extrinsics.domains import _np_xTx_domain
import opendp.prelude as dp
import pytest

dp.enable_features("honest-but-curious", "contrib", "floating-point")


@pytest.mark.skipif('numpy' not in sys.modules, reason="Numpy needed")
def test_np_array2_domain():
    import numpy as np
    print(dp.np_array2_domain(norm=1., order=2, origin=np.array([1, 2, 3]), T=float))
    print(dp.np_array2_domain(T=float).descriptor)


@pytest.mark.skipif('numpy' not in sys.modules, reason="Numpy needed")
def test_np_xTx_domain():
    import numpy as np
    domain = _np_xTx_domain(num_features=4, T=float)
    domain.member(np.random.normal(size=(4, 4)))
