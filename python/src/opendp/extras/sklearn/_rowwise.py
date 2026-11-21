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
from opendp.mod import DataFrameDomain, Domain, assert_features


# Only OpenDP implementations reviewed for rowwise inference are trusted.
# This is intentionally not extensible through subclassing or registration.
_TRUSTED_METHODS = {
    ("opendp.extras.sklearn.linear_model._logistic_regression", "LogisticRegression"): (
        "predict",
        "predict_proba",
    ),
    ("opendp.extras.sklearn.linear_model._theil_sen", "TheilSenRegressor"): (
        "predict",
    ),
    ("opendp.extras.sklearn.decomposition", "PCA"): (
        "transform",
        "inverse_transform",
        "score_samples",
    ),
    ("opendp.extras.sklearn.cluster._spherical_kmeans", "SphericalKMeans"): (
        "transform",
        "predict",
    ),
    ("opendp.extras.sklearn.compose", "ColumnTransformer"): ("transform",),
    ("opendp.extras.sklearn.pipeline", "Pipeline"): (
        "transform",
        "predict",
        "predict_proba",
        "decision_function",
        "inverse_transform",
        "score_samples",
    ),
}


def _trusted_rowwise(estimator, method):
    """Recognize reviewed inference implementations, including composite children."""
    from sklearn.frozen import FrozenEstimator

    if isinstance(estimator, FrozenEstimator):
        if type(estimator) is not FrozenEstimator or method in vars(estimator):
            return False
        estimator = estimator.estimator

    cls = type(estimator)
    if method not in _TRUSTED_METHODS.get((cls.__module__, cls.__name__), ()):
        return False
    if getattr(modules.get(cls.__module__), cls.__name__, None) is not cls:
        return False
    # An instance-level override can replace a trusted method or its dependency.
    if vars(estimator).keys() & {
        "rowwise_output_domain",
        "transform",
        "predict",
        "predict_proba",
        "decision_function",
        "inverse_transform",
        "score_samples",
        "get_precision",
    }:
        return False

    if cls.__name__ == "Pipeline":
        steps = [
            step
            for _, step in estimator.steps
            if step is not None
            and not (isinstance(step, str) and step == "passthrough")
        ]
        return all(
            _trusted_rowwise(step, method if i == len(steps) - 1 else "transform")
            for i, step in enumerate(steps)
        )
    if cls.__name__ == "ColumnTransformer":
        steps = getattr(estimator, "transformers_", estimator.transformers)
        return all(
            _trusted_rowwise(step, "transform")
            for _, step, _ in steps
            if not isinstance(step, str)
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
    # Validate the same snapshot that executes inference, including public metadata.
    fitted = deepcopy(estimator)
    if not _trusted_rowwise(fitted, method):
        assert_features("honest-but-curious")
    output = fitted.rowwise_output_domain(desc.X, method=method)
    if not isinstance(output, Domain):
        raise TypeError("rowwise_output_domain must return a Domain")
    try:
        array = output.cast(NPArray2Domain)
    except ValueError:
        array = None
        if not isinstance(output, DataFrameDomain):
            raise TypeError("Rowwise output must be DataFrameDomain or NPArray2Domain")
    if array is not None and array.num_columns is None:
        raise ValueError("Rowwise inference requires a public output width")

    infer = getattr(fitted, method)

    def apply(data):
        result = infer(data.X)
        if array is not None:
            values = np.asarray(result)
            values = values[:, None] if values.ndim == 1 else values
        else:
            import polars as pl  # type: ignore[import-not-found]

            if isinstance(result, pl.DataFrame):
                values = result
            else:
                values = np.asarray(result)
                values = values[:, None] if values.ndim == 1 else values
                values = pl.DataFrame(values, schema=output.columns, orient="row")
        return SklearnFit(values, data.y)

    return _make_transformation(
        input_domain,
        input_metric,
        sklearn_fit_domain(output, desc.y),
        input_metric,
        apply,
        lambda d: d,
    )
