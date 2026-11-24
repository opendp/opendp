from opendp.extras._utilities import to_then, with_privacy
from opendp.mod import Domain, Metric, Transformation
from opendp._lib import import_optional_dependency
from opendp._internal import _float_sum_sensitivity, _make_transformation
from opendp.extras._numerics import _pairwise_sum, _pairwise_sum_depth, _round_up
from fractions import Fraction
from functools import partial
from math import ceil, hypot, inf, nextafter


# planning to make this public, but may make more API changes


def make_np_sum(input_domain: Domain, input_metric: Metric) -> Transformation:
    """Construct a Transformation that computes a sum over the row axis of a 2-dimensional array.

    :param input_domain: instance of `array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`

    :return: a Transformation that computes the sum
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")

    dp.assert_features("contrib", "idealized-numerics")

    from opendp.extras.numpy import NPArray2Domain

    desc = input_domain.cast(NPArray2Domain)
    if desc.nan:
        raise ValueError(
            f"input_domain ({input_domain}) must not permit NaN elements"
        )  # pragma: no cover

    if input_metric != dp.symmetric_distance():  # |\label{metric-check}|
        raise ValueError(
            "input_metric must be the symmetric distance"
        )  # pragma: no cover

    if desc.num_columns is None:  # |\label{columns-check}|
        raise ValueError(
            f"input_domain ({input_domain}) must specify num_columns"
        )  # pragma: no cover

    norm = desc.norm
    if norm is None:  # |\label{norm-check}|
        raise ValueError(
            f"input_domain ({input_domain}) must have bounds. See make_np_clamp"
        )  # pragma: no cover

    output_metric = {1: dp.l1_distance, 2: dp.l2_distance}[desc.p]  # |\label{p-check}|
    origin = [abs(Fraction(*v.as_integer_ratio())) for v in np.atleast_1d(desc.origin)]
    if desc.p == 1:
        origin_norm = sum(origin, Fraction(0))
    else:
        # Round coordinates and the norm outward, including extended-precision origins.
        magnitude = hypot(*(_round_up(v) for v in origin))
        origin_norm = Fraction(nextafter(magnitude, inf)) if magnitude else Fraction(0)

    radius = Fraction(norm)
    row_bound = radius + origin_norm
    n = desc.max_size if desc.size is None else desc.size
    step = 1 if desc.size is None else 2
    per_change = row_bound if step == 1 else 2 * radius
    max_changes = 2 * n // step

    T = str(desc.T)
    integer = T[0] in {"i", "u"}

    def ideal_sensitivity(d_in):
        bound = min(d_in // step, max_changes) * per_change
        return ceil(bound) if integer else bound

    if T in {"f32", "f64"}:
        output_T = "f64"
        # A balanced tree of at most n leaves has this maximum depth.
        depth = _pairwise_sum_depth(n)
        magnitude = _round_up(n * row_bound)
        # Fail at construction if any intermediate could overflow.
        _float_sum_sensitivity(0.0, depth, magnitude, magnitude, T=output_T)

        function = _pairwise_sum

        # Even permutations (distance zero) can change floating-point
        # reductions, so retain the rounding allowance at zero distance.
        def stability(d_in):
            return _float_sum_sensitivity(
                _round_up(ideal_sensitivity(d_in)),
                depth,
                magnitude,
                magnitude,
                T=output_T,
            )

    elif T in {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64"}:
        output_T = "i64" if T[0] == "i" else "u64"
        output_dtype = np.int64 if T[0] == "i" else np.uint64
        max_output = int(np.iinfo(output_dtype).max)

        # All partial sums and all output distances must be representable.
        if 2 * n * row_bound > max_output:
            raise ValueError("possible integer sum overflow; tighten size or bounds")

        function = partial(np.sum, axis=0, dtype=output_dtype)
        stability = ideal_sensitivity

    else:
        raise ValueError(f"sum does not support input type {T}")

    return _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=output_T, nan=False)),
        output_metric(T=output_T),
        function,  # |\label{function}|
        stability,
    )


# generate then and private variants of the constructor
then_np_sum = to_then(make_np_sum)
make_private_np_sum = with_privacy(make_np_sum)
then_private_np_sum = to_then(make_private_np_sum)
