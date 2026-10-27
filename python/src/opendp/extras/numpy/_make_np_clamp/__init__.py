from dataclasses import asdict
from numbers import Integral, Real

from opendp.extras._utilities import to_then
from opendp.mod import Domain, Metric, Transformation
from opendp.context import register
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation
from opendp.typing import _NUMERIC_TYPES
from opendp.extras.numpy._norm import _rows_within_norm


def make_np_clamp(
    input_domain: Domain, input_metric: Metric, norm, p, origin=None
) -> Transformation:
    """Construct a Transformation that clamps the norm of input data.

    The output has f64 elements. Clamping can increase column cardinalities,
    so cardinality constraints are not retained in the output domain.

    :param input_domain: instance of `array2_domain(...)`
    :param input_metric: instance of `symmetric_distance()`
    :param norm: clamp each row to this norm. Required if data is not already bounded
    :param p: designates L`p` norm
    :param origin: norm clamping is centered on this point. Defaults to zero
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPArray2Domain"):
        raise ValueError("input_domain must be NPArray2Domain")  # pragma: no cover
    if input_domain.descriptor.T not in _NUMERIC_TYPES:
        raise ValueError("input_domain must have a numeric element type")
    if input_metric != dp.symmetric_distance():
        raise ValueError("input_metric must be symmetric distance")

    if isinstance(norm, bool) or not isinstance(norm, Real):
        raise ValueError("norm must be a real number")
    norm = float(norm)
    if not np.isfinite(norm) or norm <= 0.0:
        raise ValueError("norm must be finite and positive")

    if isinstance(p, bool) or not isinstance(p, Integral) or p not in {1, 2}:
        raise ValueError("order p must be the integer 1 or 2")
    p = int(p)

    origin = np.asarray(0.0 if origin is None else origin)
    if origin.dtype.kind not in {"i", "u", "f"}:
        raise ValueError("origin must be numeric")
    # Snapshot the caller's array: later mutations must not change this transformation.
    origin = origin.astype(float, copy=True)

    if origin.ndim not in {0, 1}:
        raise ValueError("origin must be a scalar or one-dimensional")
    if not np.all(np.isfinite(origin)):
        raise ValueError("origin must be finite")

    num_columns = input_domain.descriptor.num_columns
    if origin.ndim == 0 and origin != 0.0:
        if num_columns is None:
            raise ValueError(
                "input_domain must specify num_columns for a nonzero scalar origin"
            )
        origin = np.full(num_columns, origin.item(), dtype=float)
    elif origin.ndim == 1:
        if num_columns is None:
            raise ValueError(
                "input_domain must specify num_columns when origin is an array"
            )
        if origin.size != num_columns:
            raise ValueError(f"origin must have num_columns={num_columns} values")

    # Reduce rounding-induced fallbacks; the final membership check enforces containment.
    margin = 1.0 - 4 * np.finfo(float).eps

    def _function(arg):
        with np.errstate(all="ignore"):
            # Compute in f64, scaling before taking the norm to avoid overflow.
            raw = np.asarray(arg, dtype=float)
            delta = raw - origin
            centered = np.nan_to_num(delta)
            scale = np.max(np.abs(centered), axis=1, keepdims=True, initial=0.0)
            unit = centered / np.where(scale == 0, 1.0, scale)
            unit_norm = np.linalg.norm(unit, ord=p, axis=1, keepdims=True)
            limit = norm / np.maximum(unit_norm, 1.0)
            projected = origin + unit * (limit * margin)
            # Offset rounding can push a projection outward at a nonzero origin.
            # Leave zero-origin coordinates alone to preserve subnormal results.
            np.nextafter(projected, origin, out=projected, where=origin != 0)
            # Preserve admissible rows exactly, avoiding subtraction/addition rounding.
            candidate = np.where(_rows_within_norm(delta, norm, p), raw, projected)
            # Apply the same containment predicate as the output domain on every row.
            # The origin is always admissible when rounding invalidates a projection.
            return np.where(
                _rows_within_norm(candidate - origin, norm, p), candidate, origin
            )

    kwargs = asdict(input_domain.descriptor) | {
        "norm": norm,
        "p": p,
        "origin": origin,
        "nan": False,
        "T": dp.f64,
        "cardinalities": None,
    }
    return _make_transformation(
        input_domain,
        input_metric,
        dp.numpy.array2_domain(**kwargs),
        input_metric,
        _function,
        lambda d_in: d_in,
    )


# generate then variant of the constructor
# TODO: Show this in the API Reference?
register(make_np_clamp)
then_np_clamp = to_then(make_np_clamp)
