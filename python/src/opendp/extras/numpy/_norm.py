"""Numerically scaled membership checks for bounded NumPy rows."""

from fractions import Fraction

from opendp._lib import import_optional_dependency


def _rows_within_norm(x, norm, p):
    """Return an (n, 1) mask for centered 2-D rows inside an Lp ball."""
    np = import_optional_dependency("numpy")
    x = np.ascontiguousarray(x)
    with np.errstate(all="ignore"):
        scale = np.max(np.abs(x), axis=1, keepdims=True, initial=0.0)
        scale = np.where(scale == 0.0, 1.0, scale)
        row_norm = np.linalg.norm(x / scale, ord=p, axis=1, keepdims=True)
        return np.isfinite(x).all(axis=1, keepdims=True) & (row_norm <= norm / scale)


def _integer_rows_within_norm(x, origin, norm, p):
    """Check integer rows without rounding their coordinates or offsets."""
    np = import_optional_dependency("numpy")
    centers = [
        Fraction(*v.as_integer_ratio()) for v in np.broadcast_to(origin, (x.shape[1],))
    ]
    limit = Fraction(norm) ** p
    return all(
        sum(abs(int(v) - o) ** p for v, o in zip(row, centers)) <= limit for row in x
    )
