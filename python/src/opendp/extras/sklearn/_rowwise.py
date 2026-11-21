"""Compile rowwise sklearn inference into OpenDP transformations."""

from copy import deepcopy

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
from opendp.mod import Domain


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
    output = fitted.rowwise_output_domain(desc.X, method=method)
    if not isinstance(output, Domain):
        raise TypeError("rowwise_output_domain must return a Domain")
    array = output.cast(NPArray2Domain)
    if array.num_columns is None:
        raise ValueError("Rowwise inference requires a public output width")

    infer = getattr(fitted, method)

    def apply(data):
        result = infer(data.X)
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
