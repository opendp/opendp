"""Private sufficient-statistic regression with spectral flooring."""

from __future__ import annotations

import math
from numbers import Integral, Real

from opendp._internal import _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.extras.numpy import (
    NPArray2Domain,
    array2_domain,
    make_np_clamp,
)
from opendp.extras.numpy._make_np_moments import make_np_joint_moments
from opendp.extras.numpy._make_np_svec import unsvec
from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.mod import AtomDomain, VectorDomain
from opendp.typing import _NUMERIC_TYPES


def _solve(
    release, *, dimension, gram_scale, failure_probability, gaussian, public_count=None
):
    """Stabilize the already-private Gram matrix and solve."""
    np = import_optional_dependency("numpy")
    noisy_upper, noisy_cross = release
    gram = unsvec(noisy_upper, dimension)
    if public_count is not None:
        # The final feature is constant 1, so this entry is exactly n.
        # Replacing its noisy value is postprocessing using public data.
        gram[-1, -1] = public_count

    t = math.log(dimension) - math.log(failure_probability)
    if gaussian:
        # With svec Gaussian noise, the symmetric matrix has diagonal
        # variance sigma^2 and off-diagonal variance sigma^2/2.
        # Take the better of Gaussian comparison/concentration and
        # the exact Gaussian matrix-mgf Chernoff bound.
        gram_tail = gram_scale * min(
            math.sqrt(2.0 * dimension)
            + math.sqrt(-2.0 * math.log(failure_probability)),
            math.sqrt((dimension + 1) * t),
        )
    else:
        # svec Laplace noise has diagonal scale b and off-diagonal
        # scale b/sqrt(2). The exact matrix log-mgf is bounded by
        # -log(1-u^2) -(d-1) log(1-u^2/2). Evaluating a public
        # Chernoff parameter u avoids iterative optimization.
        u = min(
            math.nextafter(1.0, 0.0),
            math.sqrt(2.0 * t / (dimension + 2.0 * t)),
        )
        mgf_tail = (
            t - math.log1p(-u * u) - (dimension - 1) * math.log1p(-u * u / 2.0)
        ) / u
        entries = dimension * (dimension + 1) // 2
        gram_tail = gram_scale * min(
            mgf_tail,
            dimension * (math.log(entries) - math.log(failure_probability)),
        )

    # Eigenvalue flooring preserves strong directions rather than adding
    # the same ridge penalty to every coordinate. Everything here is
    # postprocessing of the two private releases.
    eigvals, eigvecs = np.linalg.eigh(gram)
    theta = eigvecs @ ((eigvecs.T @ noisy_cross) / np.maximum(eigvals, gram_tail))
    return np.nan_to_num(theta, copy=False)


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
    Supports native Gaussian/zCDP and Laplace/pure-DP releases.
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
    if public_size is not None and d_in % 2:
        raise ValueError("Sized data requires even d_in (row substitution)")

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
    units = None
    if public_size is not None and gaussian and fit_intercept:
        # Experimental weighting relative to the ordinary replacement bound.
        cross_norm = math.nextafter(feature_norm * y_bound, math.inf)
        units = (None, math.nextafter(1.5 * cross_norm, math.inf))
    t_joint, gram_unit, cross_unit = make_np_joint_moments(
        t_features,
        target=lambda data: data.y,
        target_bound=y_bound,
        size=public_size,
        p=2 if gaussian else 1,
        units=units,
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
    return measurement >> _new_pure_function(
        lambda release: _solve(
            (
                np.asarray(release[:num_entries]) * gram_unit,
                np.asarray(release[num_entries:]) * cross_unit,
            ),
            dimension=dimension,
            gram_scale=gram_scale,
            failure_probability=failure_probability,
            gaussian=gaussian,
            public_count=public_size if fit_intercept else None,
        )
    )
