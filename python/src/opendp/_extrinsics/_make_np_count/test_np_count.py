import opendp.prelude as dp

dp.enable_features("honest-but-curious", "contrib", "floating-point")


def test_np_count():
    import numpy as np
    from opendp.extrinsics._make_np_count import then_np_count
    space = dp.np_array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_count()
    assert trans(np.zeros(1000)) == 1000
    assert trans.map(1) == 1


def test_private_np_count():
    import numpy as np
    from opendp.extrinsics._make_np_count import then_private_np_count
    space = dp.np_array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_private_np_count(dp.zero_concentrated_divergence(T=float), scale=1.)
    print(trans(np.zeros(1000)))
    assert trans.map(1) == 0.5
