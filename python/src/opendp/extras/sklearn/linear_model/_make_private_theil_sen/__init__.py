"""Private Theil-Sen regression."""

from __future__ import annotations

import math
from numbers import Integral, Real

import numpy as np

from opendp._internal import _new_pure_function
from opendp._lib import get_np_csprng
from opendp.extras.sklearn._domains import SklearnFit, SklearnFitDescriptor
from opendp.mod import AtomDomain, Domain, Measure, Measurement, Metric, VectorDomain
from opendp.typing import _NUMERIC_TYPES

_CANDIDATE_COUNT = 100


def _validate_bounds(bounds, name):
    try:
        if len(bounds) != 2:
            raise ValueError
        pair = tuple(bounds)
        if any(isinstance(v, bool) or not isinstance(v, Real) for v in pair):
            raise ValueError
        lower, upper = map(float, pair)
        if not math.isfinite(lower) or not math.isfinite(upper) or lower >= upper:
            raise ValueError
    except (TypeError, ValueError, OverflowError):
        raise ValueError(f"{name} must be a finite, strictly increasing pair") from None
    return lower, upper


def _input_size(input_domain, input_metric):
    import opendp.prelude as dp

    desc = getattr(input_domain, "descriptor", None)
    if not isinstance(desc, SklearnFitDescriptor):
        raise ValueError("TheilSenRegressor requires a SklearnFit input domain")
    if desc.y is None:
        raise ValueError("TheilSenRegressor requires a SklearnFit input domain with y")
    x_desc = getattr(desc.X, "descriptor", None)
    if not isinstance(x_desc, dp.numpy.NPArray2Domain) or x_desc.num_columns != 1:
        raise ValueError("TheilSenRegressor requires exactly one feature column")
    if not isinstance(desc.y, VectorDomain):
        raise ValueError("TheilSenRegressor requires a vector target domain")
    if (
        x_desc.T not in _NUMERIC_TYPES
        or not isinstance(desc.y.element_domain, AtomDomain)
        or str(desc.y.element_domain.carrier_type) not in _NUMERIC_TYPES
    ):
        raise ValueError("TheilSenRegressor requires numeric elements")
    if input_metric != dp.symmetric_distance():
        raise ValueError("TheilSenRegressor supports symmetric_distance() only")
    if (
        x_desc.size is not None
        and desc.y.size is not None
        and x_desc.size != desc.y.size
    ):
        raise ValueError("X and y domains must have matching row counts")
    return x_desc.size if x_desc.size is not None else desc.y.size


def pairwise_slopes(data: SklearnFit):
    """Finite slopes from a random disjoint matching."""
    X, y = np.asarray(data.X, dtype=np.float64), np.asarray(data.y, dtype=np.float64)
    indices = np.arange(len(X))
    get_np_csprng().shuffle(indices)
    first, second = np.array_split(indices[: len(X) // 2 * 2], 2)
    with np.errstate(all="ignore"):
        dx = X[second, 0] - X[first, 0]
        valid = dx != 0
        slopes = (y[second] - y[first])[valid] / dx[valid]
    return slopes[np.isfinite(slopes)]


def make_pairwise_slopes(input_domain, input_metric):
    """Randomly matched slopes.

    Under even-to-odd insertion, either omit the new row or replace one
    endpoint of a pair and omit that endpoint. Odd-to-even insertion adds
    one pair. Both couplings change at most one slope. Filtering is nonexpansive.
    """
    import opendp.prelude as dp

    size = _input_size(input_domain, input_metric)
    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        dp.symmetric_distance(),
        function=pairwise_slopes,
        stability_map=(lambda d: 2 * d) if size is None else (lambda d: 2 * (d // 2)),
    )


def make_residuals(input_domain, input_metric, slope, intercept_bounds):
    """Size-preserving residuals for a fixed slope."""
    import opendp.prelude as dp

    size = _input_size(input_domain, input_metric)
    lower, upper = _validate_bounds(intercept_bounds, "intercept_bounds")

    def residuals(data):
        with np.errstate(all="ignore"):
            values = (
                np.asarray(data.y, dtype=np.float64)
                - slope * np.asarray(data.X, dtype=np.float64)[:, 0]
            )
        # Replace only non-finite residuals.
        return np.nan_to_num(values, nan=lower, posinf=upper, neginf=lower)

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=float, nan=False), size=size),
        dp.symmetric_distance(),
        function=residuals,
        stability_map=lambda d: d,
    )


def make_private_theil_sen(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    d_in,
    d_out,
    *,
    slope_bounds,
    intercept_bounds,
) -> Measurement:
    """Private slope median followed by private residual median.

    Bounds specify output candidates, not input clipping.
    """
    import opendp.prelude as dp

    slope_bounds = _validate_bounds(slope_bounds, "slope_bounds")
    intercept_bounds = _validate_bounds(intercept_bounds, "intercept_bounds")
    size = _input_size(input_domain, input_metric)
    if size == 0:
        raise ValueError("input size must be positive")
    if isinstance(d_in, bool) or not isinstance(d_in, Integral) or d_in <= 0:
        raise ValueError("d_in must be a positive integer")
    if size is not None and d_in % 2:
        raise ValueError("d_in must be even for fixed-size domains")
    if (
        isinstance(d_out, bool)
        or not isinstance(d_out, Real)
        or not math.isfinite(d_out)
        or d_out <= 0
    ):
        raise ValueError("d_out must be finite and positive")
    if output_measure not in (dp.max_divergence(), dp.zero_concentrated_divergence()):
        raise ValueError("output_measure must be max divergence or zCDP")

    def compose(budget):
        return dp.c.make_adaptive_composition(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            [budget, budget],
        )

    # Respect directed rounding when splitting the budget.
    budget = dp.binary_search_param(
        compose,
        d_in=d_in,
        d_out=float(d_out),
        bounds=(0.0, float(d_out)),
        T=float,
    )
    if budget == 0:
        raise ValueError("d_out is too small to allocate two positive stage budgets")
    compositor = compose(budget)
    slope_candidates = np.unique(np.linspace(*slope_bounds, _CANDIDATE_COUNT))
    intercept_candidates = np.unique(np.linspace(*intercept_bounds, _CANDIDATE_COUNT))
    slopes = make_pairwise_slopes(input_domain, input_metric)
    slope_measurement = dp.binary_search_chain(
        lambda scale: (
            slopes
            >> dp.m.then_private_quantile(
                output_measure, slope_candidates, alpha=0.5, scale=scale
            )
        ),
        d_in=d_in,
        d_out=budget,
        T=float,
    )

    residual_domain = dp.vector_domain(dp.atom_domain(T=float, nan=False), size=size)
    residual_median = dp.binary_search_chain(
        lambda scale: (
            (residual_domain, input_metric)
            >> dp.m.then_private_quantile(
                output_measure,
                intercept_candidates,
                alpha=0.5,
                scale=scale,
            )
        ),
        d_in=d_in,
        d_out=budget,
        T=float,
    )

    def fit(queryable):
        slope = queryable(slope_measurement)
        residuals = make_residuals(input_domain, input_metric, slope, intercept_bounds)
        intercept = queryable(residuals >> residual_median)
        return slope, intercept

    return compositor >> _new_pure_function(fit)
