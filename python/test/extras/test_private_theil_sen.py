from opendp.extras.numpy import make_private_theil_sen


def test_private_theil_sen():
    x_bounds = -3, 3
    y_bounds = -10, 10
    meas = make_private_theil_sen(x_bounds, y_bounds, scale=1.0)
    assert meas.map(1) == 2