"""Objective-perturbed binary logistic regression with explicit pure-DP accounting.

Privacy proof (ideal real arithmetic): given clipped augmented row norm R, a
single add/remove change changes the gradient by at most R and the logistic
Hessian by one PSD rank-one term <= R**2/4. Spherical Laplace b with density
proportional to exp(-||b||/scale) incurs R/scale in the inverse-gradient
change-of-variables density ratio. The Jacobian incurs at most
log(1 + R**2/(4*ridge)). These costs compose per changed row.

A numerical minimizer satisfying ||gradient|| <= tol lies within tol/ridge
of the exact minimizer. Independent spherical Laplace repair with scale
2*tol/(ridge*repair_eps) adds at most repair_eps via two pointwise density
ratio comparisons with the ideal objective perturbation output distribution.
All privacy claims assume idealized numerical operations.
"""
from __future__ import annotations

import math
from numbers import Integral, Real

from opendp.extras.numpy import NPArray2Domain
from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.mod import AtomDomain, Domain, Measure, Measurement, Metric, VectorDomain
from opendp.typing import _NUMERIC_TYPES

from ._objective_perturbation import perturb_and_fit


def make_private_objective_perturbation(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    d_in,
    d_out,
    *,
    C: float = 1.0,
    feature_norm: float = 1.0,
    fit_intercept: bool = True,
    tol: float = 1e-8,
    max_iter: int = 100,
) -> Measurement:
    """Objective-perturbed logistic fitting under symmetric row distance.

    Supports add/remove distance 1 and replacement distance 2 (for fixed
    row count); alpha from sklearn-style C is 1/C on the *summed* loss.
    A publicly determined additional ridge may be required by the theorem.
    In particular the intercept is regularized; this is intentionally unlike
    the ordinary sklearn logistic-regression penalty.
    """
    import opendp.prelude as dp

    dp.assert_features("contrib", "honest-but-curious", "idealized-numerics")

    if input_metric != dp.symmetric_distance():
        raise ValueError("input_metric must be symmetric_distance")
    if isinstance(d_in, bool) or not isinstance(d_in, Integral) or d_in <= 0:
        raise ValueError("d_in must be a positive integer symmetric distance")
    for name, value in (("C", C), ("feature_norm", feature_norm), ("tol", tol)):
        if (isinstance(value, bool) or not isinstance(value, Real)
                or not math.isfinite(value) or value <= 0):
            raise ValueError(f"{name} must be a finite positive real")
    if (
        isinstance(max_iter, bool)
        or not isinstance(max_iter, Integral)
        or max_iter <= 0
    ):
        raise ValueError("max_iter must be a positive integer")

    fit_desc = input_domain.cast(SklearnFitDescriptor)
    if fit_desc.y is None or not isinstance(fit_desc.y, VectorDomain):
        raise ValueError("LogisticRegression requires a vector target domain")
    x_desc = fit_desc.X.cast(NPArray2Domain)
    y_atom = fit_desc.y.element_domain
    if not isinstance(y_atom, AtomDomain):
        raise ValueError("target elements must be numeric or boolean")
    if (x_desc.T not in _NUMERIC_TYPES
            or y_atom.carrier_type not in (_NUMERIC_TYPES | {"bool"})):
        raise ValueError("X and y must contain numeric elements (y may be boolean)")
    if x_desc.num_columns is None:
        raise ValueError("feature domain must specify num_columns")
    if (x_desc.size is not None and fit_desc.y.size is not None
            and x_desc.size != fit_desc.y.size):
        raise ValueError("X and y domains must have the same row count")
    n = x_desc.size if x_desc.size is not None else fit_desc.y.size
    if n is not None and d_in % 2:
        raise ValueError(
            "sized input requires even symmetric distance for substitutions"
        )

    if output_measure == dp.max_divergence():
        eps = float(d_out)
    elif output_measure == dp.zero_concentrated_divergence():
        eps = math.sqrt(2 * float(d_out))
    else:
        raise ValueError("output_measure must be pure DP or zCDP")
    if not math.isfinite(eps) or eps <= 0:
        raise ValueError("privacy allowance must be positive and finite")

    eps_row = eps / d_in
    if eps_row <= 0 or not math.isfinite(eps_row):
        raise ValueError("privacy allowance divided by d_in underflowed")
    repair_eps = 0.05 * eps_row
    if repair_eps <= 0:
        raise ValueError("repair privacy allocation underflowed")
    radius = math.hypot(feature_norm, 1.0) if fit_intercept else float(feature_norm)
    hmax = radius**2 / 4
    # Reserve <=45% of the per-row allowance for the Jacobian ratio.
    min_ridge = hmax / math.expm1(min(eps_row * 0.45, 700.0))
    ridge = max(1.0 / float(C), min_ridge)
    if not math.isfinite(ridge) or ridge <= 0:
        raise ValueError("ridge regularization must remain positive and finite")
    jacobian_eps = math.log1p(hmax / ridge)
    noise_eps = eps_row - jacobian_eps - repair_eps
    if noise_eps <= 0 or not math.isfinite(noise_eps):
        raise ValueError("privacy budget is insufficient for objective perturbation")
    noise_scale = radius / noise_eps
    repair_scale = 2 * float(tol) / (ridge * repair_eps)
    if (
        not math.isfinite(noise_scale)
        or not math.isfinite(repair_scale)
        or repair_scale <= 0
    ):
        raise ValueError("noise scales must be finite and positive")

    # For neighboring distance k, inverse-gradient and Jacobian terms
    # contribute <=k*(noise_eps+jacobian_eps); repair contributes once
    # (<=k*repair_eps for k>=1), so privacy_map(k)=k*eps_row is valid.
    mechanism = dp.m.make_user_measurement(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=dp.max_divergence(),
        function=lambda data: perturb_and_fit(
            data, feature_norm=float(feature_norm),
            fit_intercept=bool(fit_intercept), ridge=ridge,
            noise_scale=noise_scale, repair_scale=repair_scale,
            tolerance=float(tol), max_iter=int(max_iter),
        ),
        privacy_map=lambda d: float(d) * eps_row,
    )
    if output_measure == dp.zero_concentrated_divergence():
        return dp.c.make_pureDP_to_zCDP(mechanism)
    return mechanism
