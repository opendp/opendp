"""Internal correctly rounded nuclear-distance matrix noise."""

import math
from fractions import Fraction

from opendp._internal import _make_measurement
from opendp._lib import import_optional_dependency


def _round_up(value):
    """Convert a nonnegative rational to a conservative float."""
    try:
        rounded = float(value)
    except OverflowError:
        return math.inf
    return math.nextafter(rounded, math.inf) if rounded < value else rounded


def make_private_np_wishart(input_domain, input_metric, scale):
    """Add a Wishart difference with scale q and privacy map distance/(2q).

    For d+1 degrees of freedom, the Wishart density on the PSD cone is
    proportional to exp(-trace(W)/2). To compare outputs at matrices G and G',
    translate the two independent Wisharts by the positive and negative
    spectral parts of (G-G')/q. The translations are inside the PSD cone and
    their total trace equals the nuclear distance divided by q. The joint
    density ratio is bounded by exp(distance/(2q)); integrating over factors
    proves the output bound. Rounding the sum is deterministic postprocessing.

    The input is a finite symmetric f64 matrix. Runtime is data dependent.
    This internal measurement certifies output rounding, not the upstream
    computation of the matrix or a timing-side-channel guarantee.
    """
    import opendp.prelude as dp
    from opendp._data import sample_wishart_difference

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib")
    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("Expected a symmetric matrix domain")
    if input_metric != dp.nuclear_distance():
        raise ValueError("Expected nuclear distance")
    desc = input_domain.descriptor
    if desc.T != dp.f64:
        raise ValueError("Expected an f64 matrix")
    scale = float(scale)
    if not math.isfinite(2 * scale) or scale <= 0:
        raise ValueError("Wishart scale must be positive with finite double")
    d = desc.num_features
    upper = np.triu_indices(d)
    denominator = 2 * Fraction(scale)

    if d == 1:
        # A difference of chi-square(2) variates is Laplace with scale 2.
        scalar = dp.m.make_laplace(
            dp.atom_domain(T=float, nan=False),
            dp.absolute_distance(T=float),
            2 * scale,
        )

        def release(matrix):
            return np.array([[scalar(float(matrix[0, 0]))]])

    else:

        def release(matrix):
            values = sample_wishart_difference(matrix[upper].tolist(), d, scale)
            result = np.empty((d, d))
            result[upper] = values
            result[(upper[1], upper[0])] = values
            return result

    return _make_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(),
        release,
        lambda distance: (
            _round_up(Fraction(distance) / denominator)
            if math.isfinite(distance)
            else math.inf
        ),
    )

