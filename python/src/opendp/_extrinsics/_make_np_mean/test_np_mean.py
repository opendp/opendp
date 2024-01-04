import opendp.prelude as dp

dp.enable_features("honest-but-curious", "contrib", "floating-point")


def test_private_np_mean():
    import numpy as np
    from opendp.extrinsics._make_np_mean import then_private_np_mean
    space = dp.np_array2_domain(size=1000, T=float), dp.symmetric_distance()
    trans = space >> then_private_np_mean(scale=.001, norm=1.)
    print(trans(np.random.normal(size=(1000, 4))))
    assert trans.map(2) == 2
