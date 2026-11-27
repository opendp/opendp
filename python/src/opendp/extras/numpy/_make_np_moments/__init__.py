"""Joint Gram and feature–target cross-product statistics."""

import math

from opendp._internal import _make_transformation
from opendp._lib import import_optional_dependency
from opendp.extras.numpy._make_np_sscp import make_np_sscp
from opendp.extras.numpy._make_np_svec import make_np_svec


def _joint_sensitivity(a, b, dimension, fixed_size, p):
    """Bound one row addition or replacement in normalized coordinates.

    a and b bound the Frobenius norm of the Gram contribution and the L2
    norm of the cross-product contribution, respectively. For p=1,
    first convert each block to L1 with Cauchy–Schwarz.
    """
    if p == 1:
        a *= math.sqrt(dimension * (dimension + 1) // 2)
        b *= math.sqrt(dimension)
    if not fixed_size:
        return a + b if p == 1 else math.hypot(a, b)

    if dimension == 1:
        # One magnitude can be set to one by homogeneity. For the other
        # magnitude v in [0, 1], opposite signs maximize the cross change.
        # Maximize a(1-v²)+b(1+v), or the sum of their squares.
        if p == 1:
            v = min(1.0, b / (2 * a))
            return a * (1 - v * v) + b * (1 + v)
        if b >= a / math.sqrt(2.0):
            return 2 * b
        ratio_sq = (b / a) ** 2
        v = ratio_sq / (1 + math.sqrt(1 - 2 * ratio_sq))
        return max(2 * b, math.hypot(a * (1 - v * v), b * (1 + v)))

    # For unit rows with absolute cosine q, opposite response signs
    # maximize the cross change. With row norms u,v <= 1, the squared
    # bound is a²(u⁴+v⁴)+b²(u²+v²)-2a²t²+2b²t, where
    # t=uv*q lies in [0,1]. Replacing u,v by 1 increases the first
    # two terms without changing t; any such t is attainable for d>=2.
    # The maximum is at q=min(1,b²/(2a²)).
    q = 1.0 if b >= math.sqrt(2.0) * a else (b / a) ** 2 / 2
    bound = math.hypot(a * math.sqrt(2 * (1 - q * q)), b * math.sqrt(2 * (1 + q)))
    return math.sqrt(2.0) * bound if p == 1 else bound


def make_np_joint_moments(
    t_features, *, target=None, target_bound=1.0, size=None, p=2, units=None
):
    """Return a normalized joint statistic transformation and its two units.

    Internal constructor. The caller must supply a rowwise feature
    transformation preserving aligned row adjacency, and, if specified,
    a row-aligned target accessor. A public size may also be inferred from
    the target. Without a target, every response is one (first moments).

    Feature rows must lie in a zero-centered L2 ball. Responses are clipped
    to ±target_bound. Units default to the individual statistic sensitivities
    for one adjacency step; either unit can be overridden for utility.
    The result is ordered as (svec(X.T @ X), X.T @ y), each divided by
    its unit.
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")
    desc = t_features.output_domain.descriptor
    if desc.p != 2 or desc.norm is None or desc.norm <= 0:
        raise ValueError("Expected positive L2 row bound")
    dimension = desc.num_columns
    radius = float(desc.norm)
    if size is None:
        size = desc.size
    fixed_size = size is not None
    edits_per_step = 2 if fixed_size else 1

    cross_norm = math.nextafter(radius * target_bound, math.inf)
    if not math.isfinite(cross_norm) or cross_norm <= 0:
        raise ValueError("Feature or target bounds are not representable")
    cross_bound = (
        cross_norm
        if p == 2
        else math.nextafter(math.sqrt(dimension) * cross_norm, math.inf)
    )
    if not math.isfinite(cross_bound):
        raise ValueError("Cross-product sensitivity overflows")

    t_sscp = make_np_sscp(*t_features.output_space, dp.l2_distance(T=desc.T))
    t_gram = t_sscp >> make_np_svec(*t_sscp.output_space, p=p)
    gram_sensitivity = t_gram.map(edits_per_step)
    cross_sensitivity = edits_per_step * cross_bound

    gram_unit, cross_unit = (None, None) if units is None else units
    gram_unit = gram_sensitivity if gram_unit is None else gram_unit
    cross_unit = cross_sensitivity if cross_unit is None else cross_unit
    if not all(math.isfinite(v) and v > 0 for v in (gram_unit, cross_unit)):
        raise ValueError("Statistic units must be positive and finite")
    a = radius**2 / gram_unit
    b = cross_norm / cross_unit
    per_step = math.nextafter(
        _joint_sensitivity(a, b, dimension, fixed_size, p), math.inf
    )
    if not math.isfinite(per_step):
        raise ValueError("Joint sensitivity overflows")

    def joint_statistics(data):
        X = t_features(data)
        if target is None:
            cross = X.sum(axis=0)
        else:
            y = np.clip(
                np.nan_to_num(
                    np.asarray(target(data), dtype=float),
                    nan=0.0,
                    posinf=target_bound,
                    neginf=-target_bound,
                ),
                -target_bound,
                target_bound,
            )
            cross = X.T @ y
        return np.concatenate(
            (
                np.asarray(t_gram(X)) / gram_unit,
                cross / cross_unit,
            )
        )

    t_joint = _make_transformation(
        *t_features.input_space,
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        (dp.l2_distance if p == 2 else dp.l1_distance)(T=float),
        joint_statistics,
        lambda edits: per_step * (edits // edits_per_step if fixed_size else edits),
    )
    return t_joint, gram_unit, cross_unit
