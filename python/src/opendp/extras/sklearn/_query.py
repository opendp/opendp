"""Sklearn query integration: rowwise inference or accounted private fitting."""

from opendp._internal import _new_pure_function

from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.extras.sklearn._estimator import (
    MeasurementMixin,
    TransformationMixin,
    _finish_fit,
)
from opendp.extras.sklearn._composition import (
    unwrap_frozen_estimator,
    needs_private_fit,
    snapshot,
)
from opendp.extras.sklearn._make_sklearn_fit import make_sklearn_fit
from opendp.extras.sklearn._rowwise import _make_rowwise_transform
from opendp.mod import LazyFrameDomain


def sklearn_query(query, estimator, *, y=None, method=None):
    """Fit privately or apply a rowwise sklearn inference method."""
    unwrapped = unwrap_frozen_estimator(estimator)
    if method is None and not isinstance(
        unwrapped, (MeasurementMixin, TransformationMixin)
    ):
        raise TypeError(
            "sklearn requires an OpenDP fitting or transformation capability"
        )

    domain, metric = query._stable_space()
    try:
        desc = domain.cast(SklearnFitDescriptor)
    except ValueError:
        desc = None
    if desc is not None:
        if y is not None:
            raise ValueError("y is already present in SklearnFit")
        if isinstance(desc.X, LazyFrameDomain):
            raise TypeError("Call .collect() before .sklearn() on a LazyFrame query")
    else:
        conversion = make_sklearn_fit(domain, metric, y=y)
        query = query._append(conversion)
        domain, metric = conversion.output_space

    if method is not None:
        return query._append(
            _make_rowwise_transform(domain, metric, estimator, method=method)
        )

    if not needs_private_fit(estimator):
        if not isinstance(unwrapped, TransformationMixin):
            raise TypeError("Stable inference requires TransformationMixin")
        return query._append(
            _make_rowwise_transform(domain, metric, estimator, method="transform")
        )

    distance = query._stable_distance()
    # FrozenEstimator.clone retains identity; a query owns a separate snapshot.
    prototype = snapshot(estimator)
    measurement = prototype.make_measurement(
        domain, metric, query._output_measure, distance, query._d_out
    )

    def ingest(release):
        fitted = snapshot(prototype)
        _finish_fit(fitted, release, domain)
        return fitted

    return query._append(
        measurement >> _new_pure_function(ingest, TO="ExtrinsicObject")
    )
