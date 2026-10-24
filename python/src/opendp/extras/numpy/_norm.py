"""Numerically scaled membership checks for bounded NumPy rows."""

from opendp._lib import import_optional_dependency


def _rows_within_norm(x, norm, p):
    """Return an (n, 1) mask for centered 2-D rows inside an Lp ball."""
    np = import_optional_dependency("numpy")
    with np.errstate(divide="ignore", invalid="ignore", over="ignore", under="ignore"):
        scale = np.max(np.abs(x), axis=1, keepdims=True, initial=0.0)
        scale = np.where(scale == 0.0, 1.0, scale)
        row_norm = np.linalg.norm(x / scale, ord=p, axis=1, keepdims=True)
        return np.isfinite(x).all(axis=1, keepdims=True) & (row_norm <= norm / scale)
