from opendp.transformations import *
from opendp.measurements import *
from opendp.mod import enable_features
import pytest

enable_features("contrib")


def test_make_sized_bounded_int_checked_sum():
    sum_trans = make_sized_bounded_int_checked_sum(3, (0, 10))
    assert sum_trans([1, 2, 4]) == 7

def test_make_bounded_int_monotonic_sum():
    sum_trans = make_bounded_int_monotonic_sum((0, 10))
    assert sum_trans([1, 2, 4]) == 7
    with pytest.raises(Exception):
        make_bounded_int_monotonic_sum((-1, 1))
    
def test_make_sized_bounded_int_monotonic_sum():
    sum_trans = make_sized_bounded_int_monotonic_sum(3, (0, 10))
    assert sum_trans([1, 2, 4]) == 7
    with pytest.raises(Exception):
        make_sized_bounded_int_monotonic_sum(3, (-1, 1))

def test_make_bounded_int_ordered_sum():
    sum_trans = make_bounded_int_ordered_sum((0, 10))
    assert sum_trans([1, 2, 4]) == 7

def test_make_sized_bounded_int_ordered_sum():
    sum_trans = make_sized_bounded_int_ordered_sum(3, (0, 10))
    assert sum_trans([1, 2, 4]) == 7

def test_make_bounded_int_split_sum():
    sum_trans = make_bounded_int_split_sum((0, 10))
    assert sum_trans([1, 2, 4]) == 7
    
def test_make_sized_bounded_int_split_sum():
    sum_trans = make_sized_bounded_int_split_sum(3, (0, 10))
    assert sum_trans([1, 2, 4]) == 7


def test_make_discrete_gaussian_sum():
    meas_dg_sum = make_sized_bounded_int_split_sum(3, (0, 10)) >> then_base_discrete_gaussian(2.)

    print(meas_dg_sum([1, 2, 4]))
    assert meas_dg_sum.check(3, 12.5)
