"""Internal numerical primitives for Python transformations."""

from fractions import Fraction
from math import inf, isfinite, nextafter

from opendp._lib import import_optional_dependency


def _round_up(bound: Fraction) -> float:
    """Return the least finite binary64 upper bound on a rational."""
    value = float(bound)
    if value < bound:
        value = nextafter(value, inf)
    if not isfinite(value):
        raise ValueError("floating-point bound is not finite")
    return value


def _pairwise_sum_depth(size: int) -> int:
    """Maximum number of additions on a path in _pairwise_sum."""
    return (max(1, size) - 1).bit_length()


def _pairwise_sum(values):
    """Sum along axis zero in binary64 with logarithmic addition depth.

    Each level pairs consecutive rows and carries an unpaired row forward
    without an addition. The empty sum is zero.
    """
    np = import_optional_dependency("numpy")
    values = np.asarray(values, dtype=np.float64)
    if len(values) == 0:
        return np.zeros(values.shape[1:], dtype=np.float64)

    while len(values) > 1:
        n = len(values) // 2 * 2
        pairs = values[:n:2] + values[1:n:2]
        values = (
            np.concatenate((pairs, values[n:]), axis=0) if n < len(values) else pairs
        )
    return values[0]
