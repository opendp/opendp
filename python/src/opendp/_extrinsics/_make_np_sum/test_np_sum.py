import opendp.prelude as dp

dp.enable_features("honest-but-curious", "contrib", "floating-point")


def test_np_sum():
    import numpy as np
    from opendp.extrinsics._make_np_sum import then_np_sum

    # unsized data
    space = dp.np_array2_domain(norm=1., ord=2, T=float), dp.symmetric_distance()
    trans = space >> then_np_sum()
    assert trans.map(1) == 1

    # sized data
    space = dp.np_array2_domain(norm=1., ord=2, size=1000, T=float), dp.symmetric_distance()
    trans = space >> then_np_sum()
    assert trans.map(2) == 2.

    # function
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.sum(axis=0))


def test_private_np_sum():
    import numpy as np
    from opendp.extrinsics._make_np_sum import then_private_np_sum
    space = dp.np_array2_domain(norm=1., ord=2, T=float), dp.symmetric_distance()
    trans = space >> then_private_np_sum(dp.zero_concentrated_divergence(T=float), scale=1.)
    data = np.random.normal(size=(1000, 4))
    print(trans(data))
    assert trans.map(1) == 0.5
