"""Private sufficient-statistic regression with spectral flooring."""

from __future__ import annotations

import math
from numbers import Integral, Real
from typing import Literal

from opendp._internal import _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.extras.numpy import (
    NPArray2Domain,
    array2_domain,
    make_np_clamp,
)
from opendp.extras.numpy._make_np_sscp import make_np_sscp
from opendp.extras.numpy._make_np_svec import make_np_svec, unsvec
from opendp.extras.numpy._make_np_sum import make_np_sum
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
    # Certify finite row-product and Gram-matrix sensitivity bounds.
    cross_norm = float(np.nextafter(feature_norm * y_bound, np.inf))
    if not math.isfinite(cross_norm) or cross_norm <= 0:
        raise ValueError("Feature or target bounds are not representable")

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

    t_sscp = t_features >> make_np_sscp(
        *t_features.output_space, output_metric=dp.l2_distance(T=float)
    )
    num_entries = dimension * (dimension + 1) // 2
    t_gram = t_sscp >> make_np_svec(
        *t_sscp.output_space, p=2 if gaussian else 1
    )

    # Each row contributes y_i * x_i. The sum constructor selects L1
    # or L2 sensitivity from the row norm descriptor.
    def row_products(data):
        X = np.asarray(t_features(data), dtype=float)
        y = np.clip(
            np.nan_to_num(
                np.asarray(data.y, dtype=float),
                nan=0.0,
                posinf=y_bound,
                neginf=-y_bound,
            ),
            -y_bound,
            y_bound,
        )
        return X * y[:, None]

    cross_p: Literal[1, 2] = 2 if gaussian else 1
    cross_bound = (
        cross_norm
        if gaussian
        else math.nextafter(math.sqrt(dimension) * cross_norm, math.inf)
    )
    if not math.isfinite(cross_bound):
        raise ValueError("Cross-product sensitivity overflows")
    t_products = _make_transformation(
        input_domain,
        input_metric,
        array2_domain(
            T=float,
            num_columns=dimension,
            size=public_size,
            norm=cross_bound,
            p=cross_p,
            nan=False,
        ),
        input_metric,
        row_products,
        lambda edits: edits,
    )
    t_cross = t_products >> make_np_sum(*t_products.output_space)

    # Release the sufficient statistics as one normalized vector.
    # Under fixed-size row replacement, with row norms aR, cR and
    # absolute angle cosine t, the sum of squared normalized L2 changes
    # is at most (a**4 + c**4 - 2*a**2*c**2*t**2)/2
    # + (a**2 + c**2 + 2*a*c*t)/4 <= 25/16.
    # Maximizing first over t and then a,c in [0,1] gives
    # a=c=1, t=1/4. Larger declared normalization units only help.
    # Thus joint L2 <= 5/4; joint L1 <= sqrt(2)*5/4.
    # With add/remove adjacency, a <= 1 and b <= 1, giving
    # joint L2 <= sqrt(2) and joint L1 <= 2.
    fixed_size = public_size is not None
    edits_per_step = 2 if fixed_size else 1
    gram_unit = t_gram.map(edits_per_step)
    cross_unit = t_cross.map(edits_per_step)
    if not all(math.isfinite(value) and value > 0 for value in (gram_unit, cross_unit)):
        raise ValueError("Statistic sensitivities must be positive and finite")

    metric = dp.l2_distance(T=float) if gaussian else dp.l1_distance(T=float)
    if fixed_size:
        per_step = (
            5.0 / 4.0
            if gaussian
            else math.nextafter(5.0 * math.sqrt(2.0) / 4.0, math.inf)
        )
    else:
        per_step = math.sqrt(2.0) if gaussian else 2.0

    def joint_statistics(data):
        return np.concatenate(
            (
                np.asarray(t_gram(data)) / gram_unit,
                np.asarray(t_cross(data)) / cross_unit,
            )
        )

    t_joint = _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        metric,
        joint_statistics,
        lambda edits: per_step * (edits // edits_per_step if fixed_size else edits),
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
