"""Sklearn query integration: representation adaptation and accounted fitting."""

from sklearn.base import clone

from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.extras.sklearn._estimator import DPEstimator, needs_private_fit
from opendp.extras.sklearn._frozen import FrozenTransformer
from opendp.extras.sklearn._make_sklearn_fit import make_sklearn_fit
from opendp.mod import LazyFrameDomain, Transformation


def sklearn_query(query, estimator, *, y=None):
    """Return a stable query or a measurement releasing a fitted clone."""
    stable = isinstance(estimator, FrozenTransformer) or (
        isinstance(estimator, DPEstimator) and not needs_private_fit(estimator)
    )
    if not stable and not isinstance(estimator, DPEstimator):
        raise TypeError("sklearn requires an OpenDP estimator or frozen adapter")

    prefix = query._chain
    if isinstance(prefix, tuple):
        domain, metric = prefix
    elif isinstance(prefix, Transformation):
        domain, metric = prefix.output_space
    else:
        raise ValueError("sklearn requires a metric space or stable transformation")

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
        conversion = make_sklearn_fit(domain, metric, y=y, as_numpy=False)
        prefix = conversion if isinstance(prefix, tuple) else prefix >> conversion
        query = query.new_with(chain=prefix)
        domain, metric = conversion.output_space

    if stable:
        transform = estimator.make_transform(domain, metric)
        chain = transform if isinstance(prefix, tuple) else prefix >> transform
        return query.new_with(chain=chain)

    distance = query._d_in if isinstance(prefix, tuple) else prefix.map(query._d_in)
    prototype = clone(estimator)
    measurement = prototype.make(
        domain, metric, query._output_measure, distance, query._d_out,
    )

    def ingest(release):
        fitted = clone(prototype)
        fitted._ingest_release(release)
        return fitted

    # Postprocessing is part of the measurement so resolve() and release()
    # both produce a fitted estimator. Preserve any enclosing release wrapper.
    measurement = measurement >> ingest
    chain = measurement if isinstance(prefix, tuple) else prefix >> measurement
    return query.new_with(chain=chain)
