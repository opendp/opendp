import opendp.prelude as dp

dp.enable_features("honest-but-curious", "contrib", "floating-point")


def test_np_array2():
    print(dp.np_array2_domain(T=float))
    print(dp.np_array2_domain(T=float).descriptor)
