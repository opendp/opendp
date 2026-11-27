"""Joint first and second moments of zero-centered L2-bounded rows."""

import math

from opendp._internal import _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.extras.numpy._make_np_sscp import make_np_sscp
from opendp.extras.numpy._make_np_sum import make_np_sum
from opendp.extras.numpy._make_np_svec import make_np_svec, unsvec


def _moment_sensitivity_sq(ratio, fixed_size):
    """Squared L2 sensitivity for unit rows and relative sum noise ratio >= 1."""
    if not fixed_size:
        return 1.0 + ratio**-2
    # With normalized rows x,y and q = -<x,y>, the squared change is
    # ||x||^4 + ||y||^4 + (||x||² + ||y||²)/ratio²
    # - 2q² + 2q/ratio². The last two terms are bounded by
    # 1/(2 ratio^4) by completing the square; unit rows attain the
    # bound at q = 1/(2 ratio²) in dimension >= 2.
    return 2.0 + 2.0 * ratio**-2 + 0.5 * ratio**-4


def make_private_np_moments(input_domain, input_metric, output_measure, d_in, d_out):
    """Jointly release sum and SSCP of L2-bounded rows about zero.

    The relative noise scale minimizes a first-order error proxy approximately:
    in dimensionless coordinates, matrix noise contributes O(d²) squared
    Frobenius error and sum noise contributes O(d) after centering.
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")
    desc = input_domain.descriptor
    if desc.p != 2 or desc.norm is None or desc.norm <= 0:
        raise ValueError("Expected positive L2 row bound")
    if desc.origin is not None and np.any(desc.origin != 0):
        raise ValueError("Expected zero origin")
    if output_measure != dp.zero_concentrated_divergence():
        raise ValueError("Expected zero-concentrated divergence")

    dimension = desc.num_columns
    radius = float(desc.norm)
    ratio = max(1.0, (dimension / 4.0)**0.25)
    t_sum = make_np_sum(input_domain, input_metric)
    t_sscp = make_np_sscp(input_domain, input_metric, dp.l2_distance(T=desc.T))
    t_gram = t_sscp >> make_np_svec(*t_sscp.output_space)
    sum_unit = radius * ratio
    gram_unit = radius**2
    fixed_size = desc.size is not None
    sensitivity = math.nextafter(math.sqrt(_moment_sensitivity_sq(ratio, fixed_size)), math.inf)

    def statistics(data):
        return np.concatenate((
            np.asarray(t_sum(data)) / sum_unit,
            np.asarray(t_gram(data)) / gram_unit,
        ))

    t_joint = _make_transformation(
        input_domain, input_metric,
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        dp.l2_distance(T=float),
        statistics,
        lambda edits: sensitivity * (edits // 2 if fixed_size else edits),
    )
    measurement = dp.binary_search_chain(
        lambda scale: t_joint >> dp.m.then_gaussian(scale),
        d_in=d_in, d_out=d_out, T=float,
    )
    return measurement >> _new_pure_function(
        lambda values: (
            np.asarray(values[:dimension]) * sum_unit,
            unsvec(np.asarray(values[dimension:]) * gram_unit, dimension),
        )
    )
