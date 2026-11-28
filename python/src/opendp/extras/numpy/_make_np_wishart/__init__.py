"""Internal correctly rounded nuclear-distance noise and structured PCA moments."""

import math
from fractions import Fraction

from opendp._internal import _make_measurement
from opendp._lib import import_optional_dependency
from opendp.extras.numpy import _symmetric_matrix_domain


def _round_up(value):
    """Convert a nonnegative rational to a conservative float."""
    try:
        rounded = float(value)
    except OverflowError:
        return math.inf
    return math.nextafter(rounded, math.inf) if rounded < value else rounded


def make_private_np_wishart(input_domain, input_metric, scale):
    """Add a Wishart difference with scale q and privacy map distance/(2q).

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


def make_private_np_wishart_moments(
    input_domain, input_metric, d_in, d_out, *, estimate_mean
):
    """Release bounded-row PCA moments with joint nuclear/energy calibration.

    The caller supplies a finite zero-centered L2-bounded array domain.
    For unit-radius rows z=(x, alpha), K=1+alpha². Under replacement,
    ||zz^T-z'z'^T||_* + | ||x||²-||x'||² | <= 2K.
    Under addition, ||zz^T||_* + (1-||x||²) = K.
    These are joint bounds, not sequential composition of the two maps.
    Unknown size uses a separate 5% count budget and releases the deficit.
    Returns a measurement and its public normalized Wishart scale.
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")
    desc = input_domain.cast(dp.numpy.NPArray2Domain)
    if (
        input_metric != dp.symmetric_distance()
        or desc.p != 2
        or desc.norm is None
        or not math.isfinite(desc.norm)
        or desc.norm <= 0
        or desc.num_columns is None
        or desc.num_columns < 1
        or desc.nan
        or (desc.origin is not None and np.any(desc.origin != 0))
    ):
        raise ValueError("Expected finite zero-centered L2-bounded rows")
    if not math.isfinite(d_out) or d_out <= 0:
        raise ValueError("Expected positive finite privacy budget")
    d, size, radius = desc.num_columns, desc.size, float(desc.norm)
    fixed = size is not None
    steps = 2 * (d_in // 2) if fixed else d_in
    if steps <= 0:
        raise ValueError("Expected at least one adjacency step")
    budget = Fraction(float(d_out))
    count_budget = Fraction(0) if fixed else budget / 20
    K = Fraction(5, 4) if estimate_mean else Fraction(1)
    q = _round_up(steps * K / (2 * (budget - count_budget)))
    dim = d + int(estimate_mean)
    matrix = make_private_np_wishart(
        _symmetric_matrix_domain(num_features=dim), dp.nuclear_distance(), q
    )
    energy_noise = dp.m.make_laplace(
        dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float), 2 * q
    )
    count_scale = None if fixed else _round_up(steps / count_budget)
    if not fixed:
        if not math.isfinite(count_scale):
            raise ValueError("Count scale must be finite")
        count_noise = dp.m.make_laplace(
            dp.atom_domain(T="i64"), dp.absolute_distance(T="i64"), count_scale
        )

    def release(data):
        X = np.asarray(data, dtype=float) / radius
        Z = np.column_stack((X, np.full(len(X), 0.5))) if estimate_mean else X
        gram = Z.T @ Z
        energy = float(np.trace(gram[:d, :d]))
        noisy = matrix(gram)
        if fixed:
            count = size
            energy = energy_noise(energy)
        else:
            count = count_noise(len(X))
            energy = count - energy_noise(len(X) - energy)
        energy = max(0.0, energy)
        gram = noisy[:d, :d].copy()
        gram.flat[:: d + 1] += (energy - np.trace(gram)) / d
        sums = noisy[:d, d] * (2 * radius) if estimate_mean else None
        return sums, gram * radius**2, count

    def privacy_map(edits):
        steps = 2 * (edits // 2) if fixed else edits
        loss = steps * K / (2 * Fraction(q))
        if not fixed:
            loss += Fraction(steps) / Fraction(count_scale)
        return _round_up(loss)

    measurement = _make_measurement(
        input_domain, input_metric, dp.max_divergence(), release, privacy_map
    )
    return measurement, q
