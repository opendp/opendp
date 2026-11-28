"""Private sufficient-statistic regression with spectral flooring."""

from __future__ import annotations

import math
from fractions import Fraction
from numbers import Integral, Real

from opendp._internal import _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.context import _check_sized_contributions
from opendp.extras.numpy import (
    NPArray2Domain,
    _symmetric_matrix_domain,
    array2_domain,
    make_np_clamp,
)
from opendp.extras.numpy._make_np_moments import make_np_joint_moments
from opendp.extras.numpy._make_np_svec import unsvec
from opendp.extras.numpy._make_np_wishart import _round_up, make_private_np_wishart
from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.mod import AtomDomain, VectorDomain
from opendp.typing import _NUMERIC_TYPES


def _solve(gram, cross, gram_tail):
    """Stabilize the already-private Gram matrix and solve."""
    np = import_optional_dependency("numpy")
    eigvals, eigvecs = np.linalg.eigh(gram)
    theta = eigvecs @ ((eigvecs.T @ cross) / np.maximum(eigvals, gram_tail))
    return np.nan_to_num(theta, copy=False)


def _gaussian_floor(dimension, scale, failure_probability):
    # With svec noise, diagonal variance is sigma² and off-diagonal
    # variance is sigma²/2. Take the better of two public tail bounds.
    t = math.log(dimension) - math.log(failure_probability)
    return scale * min(
        math.sqrt(2.0 * dimension)
        + math.sqrt(-2.0 * math.log(failure_probability)),
        math.sqrt((dimension + 1) * t),
    )


def _wishart_floor(dimension, scale, failure_probability, restore_count):
    # The augmented matrix includes one response coordinate. Its Gram
    # principal submatrix has dimension p and p+2 degrees of freedom.
    m = dimension + 2
    t = math.sqrt(2 * (math.log(4) - math.log(failure_probability)))
    upper = (math.sqrt(m) + math.sqrt(dimension) + t) ** 2
    lower = max(0.0, math.sqrt(m) - math.sqrt(dimension) - t) ** 2
    # Four Gaussian singular-value tails, followed by the PSD difference bound.
    # Zeroing the public intercept diagonal increases operator norm by at
    # most 2/sqrt(3), by nuclear/operator norm duality.
    correction = 2 / math.sqrt(3) if restore_count else 1.0
    return scale * (upper - lower) * correction


def _make_wishart_regression(
    t_features, *, dimension, public_size, clip_norm, y_bound, fit_intercept,
    d_in, d_out, failure_probability,
):
    """Release the normalized feature/response augmented Gram once."""
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    alpha, gamma = 0.75, 0.5
    K = Fraction(25, 16) + Fraction(1, 4) * fit_intercept

    def statistics(data):
        X = t_features(data) / clip_norm
        y = np.clip(
            np.nan_to_num(np.asarray(data.y, dtype=float), nan=0.0),
            -y_bound,
            y_bound,
        ) / y_bound
        parts = [X]
        if fit_intercept:
            parts.append(np.full(len(X), gamma))
        parts.append(alpha * y)
        Z = np.column_stack(parts)
        return Z.T @ Z

    t_gram = _make_transformation(
        *t_features.input_space,
        _symmetric_matrix_domain(num_features=dimension + 1),
        dp.nuclear_distance(),
        statistics,
        lambda edits: _round_up(K * (2 * (edits // 2) if public_size is not None else edits)),
    )
    scale = _round_up(Fraction(t_gram.map(d_in)) / (2 * Fraction(float(d_out))))
    measurement = t_gram >> make_private_np_wishart(*t_gram.output_space, scale)
    restore_count = fit_intercept and public_size is not None
    floor = _wishart_floor(dimension, scale, failure_probability, restore_count)

    def solve(matrix):
        gram = matrix[:dimension, :dimension].copy()
        if restore_count:
            gram[-1, -1] = gamma**2 * public_size
        theta = _solve(gram, matrix[:dimension, dimension] / alpha, floor)
        theta[: dimension - int(fit_intercept)] *= y_bound / clip_norm
        if fit_intercept:
            theta[-1] *= y_bound * gamma
        return theta

    return measurement >> _new_pure_function(solve)


def make_private_linear_regression(
    input_domain,
    input_metric,
    output_measure,
    d_in,
    d_out,
    *,
    clip_norm,
    y_bound,
    fit_intercept=True,
    failure_probability=0.05,
):
    """Fit using private Gram and cross-product sufficient statistics.

    Inspired by AdaSSP-Revised, but regularizes the already-private Gram
    matrix rather than spending privacy budget on a minimum eigenvalue.
    Supports Gaussian/zCDP and augmented Wishart/pure-DP releases.
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")

    if input_metric != dp.symmetric_distance():
        raise ValueError("LinearRegression requires symmetric_distance()")
    if output_measure == dp.zero_concentrated_divergence():
        gaussian = True
    elif output_measure == dp.max_divergence():
        gaussian = False
    else:
        raise ValueError("Expected zero_concentrated_divergence() or max_divergence()")
    if (
        isinstance(d_out, bool)
        or not isinstance(d_out, Real)
        or not 0 < float(d_out) < math.inf
    ):
        raise ValueError("d_out must be a finite positive privacy allowance")

    if isinstance(d_in, bool) or not isinstance(d_in, Integral) or d_in <= 0:
        raise ValueError("d_in must be a positive number of row edits")

    for name, value in (("clip_norm", clip_norm), ("y_bound", y_bound)):
        if (
            isinstance(value, bool)
            or not isinstance(value, Real)
            or not 0 < float(value) < math.inf
        ):
            raise ValueError(f"{name} must be a finite positive number")
    if not 0 < failure_probability < 1:
        raise ValueError("failure_probability must be in (0, 1)")
    if not isinstance(fit_intercept, bool):
        raise ValueError("fit_intercept must be boolean")

    desc = input_domain.cast(SklearnFitDescriptor)
    if desc.y is None:
        raise ValueError("LinearRegression requires targets")
    x_desc = desc.X.cast(NPArray2Domain)
    if x_desc.T not in _NUMERIC_TYPES:
        raise ValueError("X must contain numeric elements")
    if not isinstance(desc.y, VectorDomain) or not isinstance(
        desc.y.element_domain, AtomDomain
    ):
        raise ValueError("y must be a one-dimensional numeric vector")
    if desc.y.element_domain.carrier_type not in _NUMERIC_TYPES:
        raise ValueError("y must contain numeric elements")
    if x_desc.num_columns is None or x_desc.num_columns < 1:
        raise ValueError("X must have a positive, public number of features")
    if (
        x_desc.size is not None
        and desc.y.size is not None
        and x_desc.size != desc.y.size
    ):
        raise ValueError("X and y must have matching public row counts")
    public_size = x_desc.size if x_desc.size is not None else desc.y.size
    _check_sized_contributions(input_domain, input_metric, d_in)

    clip_norm, y_bound = float(clip_norm), float(y_bound)
    dimension = x_desc.num_columns + int(fit_intercept)
    feature_norm = math.hypot(clip_norm, float(fit_intercept))
    feature_norm = float(np.nextafter(feature_norm, np.inf))
    # Projection preserves row adjacency. The existing NumPy clamp certifies
    # the rowwise L2 bound before the constant intercept feature is appended.
    t_x = _make_transformation(
        input_domain,
        input_metric,
        desc.X,
        input_metric,
        lambda data: data.X,
        lambda edits: edits,
    )
    t_clamp = make_np_clamp(desc.X, input_metric, clip_norm, p=2)
    t_features = t_x >> t_clamp
    if not gaussian:
        return _make_wishart_regression(
            t_features,
            dimension=dimension,
            public_size=public_size,
            clip_norm=clip_norm,
            y_bound=y_bound,
            fit_intercept=fit_intercept,
            d_in=d_in,
            d_out=d_out,
            failure_probability=failure_probability,
        )
    if fit_intercept:
        t_append = _make_transformation(
            t_clamp.output_domain,
            input_metric,
            array2_domain(
                T=float,
                num_columns=dimension,
                size=x_desc.size,
                norm=feature_norm,
                p=2,
                nan=False,
            ),
            input_metric,
            lambda X: np.column_stack((X, np.ones(len(X)))),
            lambda edits: edits,
        )
        t_features = t_features >> t_append

    num_entries = dimension * (dimension + 1) // 2
    t_joint, gram_unit, cross_unit = make_np_joint_moments(
        t_features,
        target=lambda data: data.y,
        target_bound=y_bound,
        size=public_size,
        p=2,
    )

    def make_measurement(scale):
        return t_joint >> dp.m.then_noise(output_measure, scale)

    # The joint privacy map is calibrated in its native output measure.
    multiplier = dp.binary_search_param(
        make_measurement, d_in=d_in, d_out=d_out, T=float
    )
    gram_scale = multiplier * gram_unit
    measurement = make_measurement(multiplier)
    if not measurement.check(d_in, d_out):
        raise ValueError("Privacy calibration exceeds the requested allowance")
    floor = _gaussian_floor(dimension, gram_scale, failure_probability)

    def solve(release):
        gram = unsvec(np.asarray(release[:num_entries]) * gram_unit, dimension)
        if fit_intercept and public_size is not None:
            gram[-1, -1] = public_size
        return _solve(gram, np.asarray(release[num_entries:]) * cross_unit, floor)

    return measurement >> _new_pure_function(solve)
