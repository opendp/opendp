from opendp.extras._utilities import to_then
from opendp.mod import Domain, Metric, Transformation
from opendp.context import register
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation


def make_np_clamp(
    input_domain: Domain, input_metric: Metric, norm, p, origin=None
) -> Transformation:
    """Construct a Transformation that clamps the norm of input data.

    :param input_domain: instance of `array2_domain(...)`
    :param input_metric: instance of `symmetric_distance()`
    :param norm: clamp each row to this norm. Required if data is not already bounded
    :param p: designates L`p` norm
    :param origin: norm clamping is centered on this point. Defaults to zero
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")

    dp.assert_features("contrib")

    if not str(input_domain).startswith("NPArray2Domain"):
        raise ValueError("input_domain must be NPArray2Domain")  # pragma: no cover

    norm = float(norm)
    if norm <= 0.0:
        raise ValueError("norm must positive")  # pragma: no cover
    if np.isnan(norm):
        raise ValueError("norm must not be NaN")  # pragma: no cover
    if p not in {1, 2}:
        raise ValueError("order p must be 1 or 2")  # pragma: no cover

    if origin is None:
        origin = 0.0

    if not np.all(np.isfinite(origin)):
        raise ValueError("origin must be finite")  # pragma: no cover

    def get_norm(x):
        with np.errstate(over="ignore"):
            current_norm = np.linalg.norm(x, ord=p, axis=1, keepdims=True)
        return np.nan_to_num(current_norm)

    def _function(arg):
        with np.errstate(over="ignore"):
            # don't mutate the input array
            arg = arg - origin
        arg = np.nan_to_num(arg)

        # may have to run multiple times due to FP rounding
        current_norm = get_norm(arg)
        while current_norm.max() > norm:
            with np.errstate(under="ignore", over="ignore"):
                factor = current_norm / norm
            arg /= np.maximum(np.nan_to_num(factor), 1)
            current_norm = get_norm(arg)

        return arg + origin

    kwargs = input_domain.descriptor._asdict() | {
        "norm": norm,
        "p": p,
        "origin": origin,
        "nan": False,
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
