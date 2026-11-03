"""Private matched-pair slope and adaptive median-residual regression."""

from __future__ import annotations

import math
from numbers import Integral, Real
from secrets import SystemRandom

from opendp._internal import _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.extras.sklearn._domains import SklearnFit, SklearnFitDescriptor
from opendp.mod import AtomDomain, Domain, Measure, Measurement, Metric, VectorDomain
from opendp.typing import _NUMERIC_TYPES

# SystemRandom.shuffle uses OS entropy and unbiased integer rejection sampling.
# Neither NumPy's RNG nor an analyst-provided seed controls the matching.
_shuffle = SystemRandom().shuffle
_CANDIDATE_COUNT = 100


def _numpy():
    return import_optional_dependency("numpy")


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
    """Return finite float64 slopes from one secure, uniform disjoint matching."""
    np = _numpy()
    X, y = np.asarray(data.X, dtype=np.float64), np.asarray(data.y, dtype=np.float64)
    indices = np.arange(len(X))
    _shuffle(indices)
    # Shuffle before omitting the last row, so the omitted row is uniform too.
    first, second = np.array_split(indices[: len(X) // 2 * 2], 2)
    with np.errstate(all="ignore"):
        dx = X[second, 0] - X[first, 0]
        valid = dx != 0
        slopes = (y[second] - y[first])[valid] / dx[valid]
    return slopes[np.isfinite(slopes)]


def make_pairwise_slopes(input_domain, input_metric):
    """Make the randomized matched-pair slope transformation.

    Uniform shuffling gives a uniform matching and, for odd sizes, a uniformly
    omitted row. Couple insertion into an even-sized input by choosing the new
    omitted row uniformly: omit the new row and keep the old matching, or replace
    an omitted old row with the new row in its pair. At most one slope is replaced
    (two symmetric edits). For an odd-sized input, pair the previously omitted row
    with the new row, adding at most one slope. These couplings have the correct
    uniform matching marginals. Filtering slopes is deterministic and row-local
    on the matched pairs, so it cannot increase distance. Permutations do not
    change the output distribution.
    """
    import opendp.prelude as dp

    size = _input_size(input_domain, input_metric)
    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        dp.symmetric_distance(),
        function=pairwise_slopes,
        # Fixed-size replacements change at most one matched slope per replacement.
        stability_map=(lambda d: 2 * d) if size is None else (lambda d: 2 * (d // 2)),
    )


def make_residuals(input_domain, input_metric, slope, intercept_bounds):
    """Make a row-local, size-preserving residual transformation for a fixed slope."""
    import opendp.prelude as dp

    np = _numpy()
    size = _input_size(input_domain, input_metric)
    lower, upper = _validate_bounds(intercept_bounds, "intercept_bounds")

    def residuals(data):
        with np.errstate(all="ignore"):
            values = (
                np.asarray(data.y, dtype=np.float64)
                - slope * np.asarray(data.X, dtype=np.float64)[:, 0]
            )
        # Retain every row. Finite residuals need not lie within the candidate interval.
        return np.nan_to_num(values, nan=lower, posinf=upper, neginf=lower)

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=float, nan=False), size=size),
        dp.symmetric_distance(),
        function=residuals,
        stability_map=lambda d: d,
    )


def _candidate_grid(bounds):
    np = _numpy()
    with np.errstate(all="ignore"):
        values = np.linspace(*bounds, _CANDIDATE_COUNT)
        if not np.isfinite(values).all():
            # Avoid overflow in upper - lower for wide finite candidate intervals.
            weights = np.linspace(0.0, 1.0, _CANDIDATE_COUNT)
            values = (1.0 - weights) * bounds[0] + weights * bounds[1]
    return np.clip(values, *bounds)


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
    """Release a private median matched-pair slope and an adaptive median residual.

    The two required public bounds are finite, strictly increasing candidate
    intervals, not clipping constraints on private data. Supports numeric
    SklearnFit inputs with one feature, symmetric distance, and epsilon-DP or
    zCDP. One secure random matching and 100 candidates per coefficient are used.
    Half the privacy budget is allocated to each stage through OpenDP composition.
    """
    import opendp.prelude as dp

    slope_bounds = _validate_bounds(slope_bounds, "slope_bounds")
    intercept_bounds = _validate_bounds(intercept_bounds, "intercept_bounds")
    size = _input_size(input_domain, input_metric)
    if isinstance(d_in, bool) or not isinstance(d_in, Integral) or d_in <= 0:
        raise ValueError("d_in must be a positive integer")
    if not isinstance(d_out, Real) or not math.isfinite(d_out) or d_out <= 0:
        raise ValueError("d_out must be finite and positive")
    if output_measure not in (dp.max_divergence(), dp.zero_concentrated_divergence()):
        raise ValueError("output_measure must be max divergence or zCDP")
    # Distance 1 is vacuous for fixed-size inputs. Calibrate at least a replacement.
    calibration_d_in = max(d_in, 2) if size is not None else d_in

    def compose(budget):
        return dp.c.make_adaptive_composition(
            input_domain,
            input_metric,
            output_measure,
            calibration_d_in,
            [budget, budget],
        )

    # Search using OpenDP's upward-rounded composition map, rather than d_out / 2.
    budget = dp.binary_search_param(
        compose,
        d_in=calibration_d_in,
        d_out=float(d_out),
        bounds=(0.0, float(d_out)),
        T=float,
    )
    if budget == 0:
        raise ValueError("d_out is too small to allocate two positive stage budgets")
    compositor = compose(budget)
    slope_candidates = _candidate_grid(slope_bounds)
    intercept_candidates = _candidate_grid(intercept_bounds)
    slopes = make_pairwise_slopes(input_domain, input_metric)
    slope_measurement = dp.binary_search_chain(
        lambda scale: (
            slopes
            >> dp.m.then_private_quantile(
                output_measure, slope_candidates, alpha=0.5, scale=scale
            )
        ),
        d_in=calibration_d_in,
        d_out=budget,
        T=float,
    )

    residual_domain = make_residuals(
        input_domain, input_metric, 0.0, intercept_bounds
    ).output_domain
    residual_space = (residual_domain, input_metric)
    if size == 0:
        # The native quantile constructor currently divides by zero for size=0.
        # Forget only this public zero size; keep the residual transform size-preserving.
        residual_space = dp.t.make_user_transformation(
            *residual_space,
            dp.vector_domain(dp.atom_domain(T=float, nan=False)),
            input_metric,
            function=lambda values: values,
            stability_map=lambda d: d,
        )
    residual_median = dp.binary_search_chain(
        lambda scale: (
            residual_space
            >> dp.m.then_private_quantile(
                output_measure,
                intercept_candidates,
                alpha=0.5,
                scale=scale,
            )
        ),
        d_in=calibration_d_in,
        d_out=budget,
        T=float,
    )

    def fit(queryable):
        slope = queryable(slope_measurement)
        residuals = make_residuals(input_domain, input_metric, slope, intercept_bounds)
        intercept = queryable(residuals >> residual_median)
        return slope, intercept

    return compositor >> _new_pure_function(fit)
