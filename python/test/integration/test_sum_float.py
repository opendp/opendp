from opendp.transformations import *
from opendp.mod import enable_features

enable_features("contrib")


def test_make_bounded_float_checked_sum():
    sum_trans = make_bounded_float_checked_sum(100, (0.0, 10.0))
    assert sum_trans([1.0, 2.0, 4.0]) == 7

    make_bounded_float_checked_sum(100, (0.0, 10.0), "Pairwise<T>")([1.0, 2.0])
    make_bounded_float_checked_sum(100, (0.0, 10.0), "Pairwise<f64>")([1.0, 2.0])
    make_bounded_float_checked_sum(100, (0.0, 10.0), "Sequential<T>")([1.0, 2.0])
    make_bounded_float_checked_sum(100, (0.0, 10.0), "Sequential<f64>")([1.0, 2.0])


def test_make_sized_bounded_float_checked_sum():
    sum_trans = make_sized_bounded_float_checked_sum(100, (0.0, 10.0))
    assert sum_trans([1.0, 2.0, 4.0]) == 7
    make_sized_bounded_float_checked_sum(100, (0.0, 10.0), "Sequential<T>")([1.0, 2.0])


def test_make_bounded_float_ordered_sum():
    sum_trans = make_bounded_float_ordered_sum(100, (0.0, 10.0))
    assert sum_trans([1.0, 2.0, 4.0]) == 7
    make_bounded_float_ordered_sum(100, (0.0, 10.0), "Sequential<f64>")


def test_make_sized_bounded_float_ordered_sum():
    sum_trans = make_sized_bounded_float_ordered_sum(3, (0.0, 10.0))
    assert sum_trans([1.0, 2.0, 4.0]) == 7
    make_sized_bounded_float_ordered_sum(3, (0.0, 10.0), "Sequential<T>")
