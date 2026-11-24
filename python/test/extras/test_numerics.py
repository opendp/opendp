from fractions import Fraction
from math import inf, nextafter
from sys import float_info

import pytest

from opendp._internal import _float_sum_sensitivity, _l01inf_to_lp
from opendp.extras._numerics import _pairwise_sum, _pairwise_sum_depth, _round_up


def test_round_up():
    assert _round_up(Fraction(1)) == 1.0
    assert _round_up(Fraction(1) + Fraction(1, 2**54)) == nextafter(1.0, inf)
    assert _round_up(Fraction(1, 2**1075)) == nextafter(0.0, inf)
    with pytest.raises(ValueError, match="bound is not finite"):
        _round_up(Fraction(float_info.max) + 1)


@pytest.mark.parametrize("size,depth", [(0, 0), (1, 0), (2, 1), (3, 2), (8, 3), (9, 4)])
def test_pairwise_sum(size, depth):
    np = pytest.importorskip("numpy")
    values = np.arange(size * 6, dtype=np.float32).reshape(size, 6)[:, ::2]
    result = _pairwise_sum(values)
    assert result.dtype == np.float64
    np.testing.assert_array_equal(result, values.sum(axis=0, dtype=np.float64))
    assert _pairwise_sum_depth(size) == depth


def test_pairwise_sum_order():
    np = pytest.importorskip("numpy")
    values = np.array([1e16, 1.0, -1e16, 1.0])
    # Distinguish the balanced reduction from a left fold.
    assert _pairwise_sum(values) == 0.0


def test_float_sum_sensitivity():
    assert 1.0 < _float_sum_sensitivity(1.0, 4, 16.0, 32.0) < 1.000001


def test_l01inf_to_lp():
    assert _l01inf_to_lp((3, 5, 2), p=2) == 3.0
