"""Shared privacy allocation and checked child fitting for sklearn composites."""

from math import isfinite

from opendp.context import _cast_measure, _normalize_compositor


def unwrap_frozen_estimator(step):
    """Get the fitted estimator stored in a FrozenEstimator."""
    from sklearn.frozen import FrozenEstimator
    from sklearn.utils.validation import check_is_fitted

    if isinstance(step, FrozenEstimator):
        check_is_fitted(step)
        return step.estimator
    return step


def needs_private_fit(step):
    """Private fitting is required only for unfrozen measurement-capable steps."""
    from sklearn.frozen import FrozenEstimator
    from opendp.extras.sklearn._estimator import MeasurementMixin

    return (
        not isinstance(step, FrozenEstimator)
        and isinstance(step, MeasurementMixin)
        and (not hasattr(step, "_has_private_fit") or step._has_private_fit())
    )


def allocate_private_stages(
    input_domain,
    input_metric,
    output_measure,
    d_in,
    d_out,
    stage_names,
    privacy_weights,
):
    """Create an accountant and budgets for the named private-fitting stages."""
    names = tuple(stage_names)
    if not names:
        raise ValueError("Composition has no private fitting stages")
    if privacy_weights is None:
        weights = None
    else:
        if not isinstance(privacy_weights, dict) or set(privacy_weights) != set(names):
            raise ValueError("privacy_weights must name exactly the private stages")
        weights = [float(privacy_weights[name]) for name in names]
        if any(not isfinite(w) or w <= 0 for w in weights):
            raise ValueError("privacy_weights must be finite and positive")

    compositor, budgets, _ = _normalize_compositor(
        input_domain,
        (input_metric, d_in),
        (output_measure, d_out),
        split_evenly_over=len(names) if weights is None else None,
        split_by_weights=weights,
    )
    assert budgets is not None
    return compositor, tuple(budgets)


def checked_child_fit(
    stage,
    name,
    prefix,
    space,
    output_measure,
    d_in,
    budget,
):
    """Construct and check a child measurement before invoking the accountant."""
    distance = d_in if prefix is None else prefix.map(d_in)
    child = stage.make_measurement(*space, output_measure, distance, budget)
    measurement = child if prefix is None else prefix >> child
    measurement = _cast_measure(measurement, output_measure, budget)
    if not measurement.check(d_in, budget):
        raise ValueError(f"Private stage {name!r} exceeds its privacy allowance")
    return measurement
