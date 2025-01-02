from opendp.transformations import *
from opendp.mod import enable_features
enable_features("contrib")

def test_make_bounded_float_checked_sum():
    sum_trans = make_bounded_float_checked_sum(100, (0., 10.))
    assert sum_trans([1., 2., 4.]) == 7

    make_bounded_float_checked_sum(100, (0., 10.), "Pairwise<T>")([1., 2.])
    make_bounded_float_checked_sum(100, (0., 10.), "Pairwise<f64>")([1., 2.])
    make_bounded_float_checked_sum(100, (0., 10.), "Sequential<T>")([1., 2.])
    make_bounded_float_checked_sum(100, (0., 10.), "Sequential<f64>")([1., 2.])

def test_make_sized_bounded_float_checked_sum():
    sum_trans = make_sized_bounded_float_checked_sum(100, (0., 10.))
    assert sum_trans([1., 2., 4.]) == 7
    make_sized_bounded_float_checked_sum(100, (0., 10.), "Sequential<T>")([1., 2.])

def test_make_bounded_float_ordered_sum():
    sum_trans = make_bounded_float_ordered_sum(100, (0., 10.))
    assert sum_trans([1., 2., 4.]) == 7
    make_bounded_float_ordered_sum(100, (0., 10.), "Sequential<f64>")

def test_make_sized_bounded_float_ordered_sum():
    sum_trans = make_sized_bounded_float_ordered_sum(3, (0., 10.))
    assert sum_trans([1., 2., 4.]) == 7
    make_sized_bounded_float_ordered_sum(3, (0., 10.), "Sequential<T>")

