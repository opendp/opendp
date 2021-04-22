import opendp

odp = opendp.OpenDP()


def test_sum_float():
    mean = odp.trans.make_bounded_sum(b"<HammingDistance, L1Sensitivity<f64>>", opendp.f64_p(0.), opendp.f64_p(10.))
    result = odp.transformation_invoke(mean, [1., 2., 3., 4., 5.])
    assert isinstance(result, float)
    assert result == 15.


def test_sum_int():
    mean = odp.trans.make_bounded_sum(b"<HammingDistance, L1Sensitivity<i32>>", opendp.i32_p(0), opendp.i32_p(10))
    result = odp.transformation_invoke(mean, [1, 2, 3, 4, 5])
    assert isinstance(result, int)
    assert result == 15


def test_mean():
    mean = odp.trans.make_bounded_mean(b"<HammingDistance, L1Sensitivity<f64>, f64>", opendp.f64_p(0.), opendp.f64_p(10.), 5)
    result = odp.transformation_invoke(mean, [1., 2., 3., 4., 5.])
    assert result == 3.
