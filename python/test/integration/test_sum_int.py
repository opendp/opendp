import opendp.prelude as dp
import pytest



def test_make_sized_bounded_int_checked_sum():
    sum_trans = dp.t.make_sized_bounded_int_checked_sum(3, (0, 10))
    assert sum_trans([1, 2, 4]) == 7

def test_make_bounded_int_monotonic_sum():
    sum_trans = dp.t.make_bounded_int_monotonic_sum((0, 10))
    assert sum_trans([1, 2, 4]) == 7
    with pytest.raises(Exception):
        dp.t.make_bounded_int_monotonic_sum((-1, 1))
    
def test_make_sized_bounded_int_monotonic_sum():
    sum_trans = dp.t.make_sized_bounded_int_monotonic_sum(3, (0, 10))
    assert sum_trans([1, 2, 4]) == 7
    with pytest.raises(Exception):
        dp.t.make_sized_bounded_int_monotonic_sum(3, (-1, 1))

def test_make_bounded_int_ordered_sum():
    sum_trans = dp.t.make_bounded_int_ordered_sum((0, 10))
    assert sum_trans([1, 2, 4]) == 7

def test_make_sized_bounded_int_ordered_sum():
    sum_trans = dp.t.make_sized_bounded_int_ordered_sum(3, (0, 10))
    assert sum_trans([1, 2, 4]) == 7

def test_make_bounded_int_split_sum():
    sum_trans = dp.t.make_bounded_int_split_sum((0, 10))
    assert sum_trans([1, 2, 4]) == 7
    
def test_make_sized_bounded_int_split_sum():
    sum_trans = dp.t.make_sized_bounded_int_split_sum(3, (0, 10))
    assert sum_trans([1, 2, 4]) == 7


def test_make_discrete_gaussian_sum():
    meas_dg_sum = dp.t.make_sized_bounded_int_split_sum(3, (0, 10)) >> dp.m.then_gaussian(2.)

    print(meas_dg_sum([1, 2, 4]))
    assert meas_dg_sum.check(3, 12.5)
