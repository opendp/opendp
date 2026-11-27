"""Shared privacy allocation and checked child fitting for sklearn composites."""

from copy import deepcopy
from math import isfinite

from sklearn.base import clone
from sklearn.frozen import FrozenEstimator
from sklearn.utils.validation import check_is_fitted

from opendp.context import _cast_measure, _normalize_compositor, domain_of
from opendp.extras.sklearn._domains import SklearnFit
from opendp.extras.sklearn._estimator import (
    MeasurementMixin,
    _finish_fit,
    _set_fit_metadata,
)


def unwrap_frozen_estimator(step):
    """Get the fitted estimator stored in a FrozenEstimator."""
    if isinstance(step, FrozenEstimator):
        check_is_fitted(step)
        return step.estimator
    return step


def snapshot(estimator):
    """Clone public configuration and detach even frozen children."""
    return deepcopy(clone(estimator))


def fit_child(step, input_domain, queryable=None, measurement=None):
    """Install accounted parameters, or initialize a stable child from its domain."""
    if measurement is not None:
        _finish_fit(step, queryable(measurement), input_domain)
    elif isinstance(step, FrozenEstimator):
        return
    elif isinstance(step, _CompositeMixin):
        if not getattr(step, "_dp_fitted_", False):
            _finish_fit(step, step._fit_from_domain(input_domain), input_domain)
    else:
        _set_fit_metadata(step, input_domain)


def needs_private_fit(step):
    """Private fitting is required only for unfrozen measurement-capable steps."""
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


class _CompositeMixin(MeasurementMixin):
    """Fit private children with accounting, or stable children from public domains."""

    def fit(self, X, y=None, *, privacy_unit=None, privacy_loss=None, domain=None):
        if self._has_private_fit():
            if privacy_unit is None or privacy_loss is None:
                raise TypeError("Private fit requires privacy_unit and privacy_loss")
            return super().fit(
                X,
                y,
                privacy_unit=privacy_unit,
                privacy_loss=privacy_loss,
                domain=domain,
            )
        if privacy_unit is not None or privacy_loss is not None:
            raise ValueError("Stable-only fit does not consume a privacy budget")
        if domain is None:
            domain = domain_of(SklearnFit(X, y), infer=True)

        # Validate a fresh configuration before replacing any fitted state.
        fresh = clone(self)
        _finish_fit(self, fresh._fit_from_domain(domain), domain)
        return self
