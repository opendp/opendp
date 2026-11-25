"""Compile rowwise sklearn inference into OpenDP transformations."""

from copy import deepcopy
from sys import modules

import numpy as np

from opendp._internal import _make_transformation
from opendp.extras.numpy import NPArray2Domain
from opendp.extras.sklearn._domains import (
    SklearnFit,
    SklearnFitDescriptor,
    sklearn_fit_domain,
)
from opendp.extras.sklearn._estimator import TransformationMixin
from opendp.metrics import symmetric_distance
from opendp.mod import Domain, assert_features


# Only OpenDP implementations reviewed for rowwise inference are trusted.
# This is intentionally not extensible through subclassing or registration.
_TRUSTED_CLASSES = {
    ("opendp.extras.sklearn.linear_model._sgd_classifier", "SGDClassifier"),
    ("opendp.extras.sklearn.linear_model._theil_sen", "TheilSenRegressor"),
    ("opendp.extras.sklearn.decomposition", "PCA"),
    ("opendp.extras.sklearn.cluster._spherical_kmeans", "SphericalKMeans"),
    ("opendp.extras.sklearn.compose", "ColumnTransformer"),
    ("opendp.extras.sklearn.pipeline", "Pipeline"),
}


def _trusted_rowwise(estimator):
    """Recognize reviewed classes and their active composite children.

    As elsewhere in OpenDP, reviewed implementations and fitted state must not
    be modified to violate their contracts. This is not a Python sandbox.
    """
    from sklearn.frozen import FrozenEstimator

    if type(estimator) is FrozenEstimator:
        return _trusted_rowwise(estimator.estimator)

    cls = type(estimator)
    if (cls.__module__, cls.__name__) not in _TRUSTED_CLASSES:
        return False
    if getattr(modules.get(cls.__module__), cls.__name__, None) is not cls:
        return False
    if cls.__name__ == "Pipeline":
        steps = [
            step
            for _, step in estimator.steps
            if step is not None
            and not (isinstance(step, str) and step == "passthrough")
        ]
        return all(_trusted_rowwise(step) for step in steps)
    if cls.__name__ == "ColumnTransformer":
        steps = getattr(estimator, "transformers_", estimator.transformers)
        return all(
            _trusted_rowwise(step) for _, step, _ in steps if not isinstance(step, str)
        )
    return True


def _make_rowwise_transform(
    input_domain, input_metric, estimator, *, method="transform"
):
    """Compile row-independent inference with data-independent failure."""
    if input_metric != symmetric_distance():
        raise ValueError("Rowwise inference requires symmetric_distance()")

    if not isinstance(estimator, TransformationMixin):
        raise TypeError("Stage must implement TransformationMixin")
    if not callable(getattr(estimator, method, None)):
        raise TypeError(f"{type(estimator).__name__} has no sklearn {method!r} method")

    desc = input_domain.cast(SklearnFitDescriptor)
    # Check the same snapshot that will execute inference.
    fitted = deepcopy(estimator)
    if not _trusted_rowwise(fitted):
        assert_features("honest-but-curious")
    output = fitted.rowwise_output_domain(desc.X, method=method)
    if not isinstance(output, Domain):
        raise TypeError("rowwise_output_domain must return a Domain")
    array = output.cast(NPArray2Domain)
    if array.num_columns is None:
        raise ValueError("Rowwise inference requires a public output width")

    infer = getattr(fitted, method)

    def apply(data):
        result = infer(data.X)
        if method == "transform":
            return SklearnFit(result, data.y)
        values = np.asarray(result)
        values = values[:, None] if values.ndim == 1 else values
        return SklearnFit(values, data.y)

    return _make_transformation(
        input_domain,
        input_metric,
        sklearn_fit_domain(output, desc.y),
        input_metric,
        apply,
        lambda d: d,
    )
