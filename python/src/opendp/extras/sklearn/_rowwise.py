"""Compile certified rowwise sklearn inference into OpenDP transformations."""

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
from opendp.metrics import (
    change_one_distance,
    hamming_distance,
    insert_delete_distance,
    symmetric_distance,
)
from opendp.mod import Domain


def _make_rowwise_transform(
    input_domain, input_metric, estimator, *, method="transform"
):
    """Compile row-independent inference with data-independent failure."""
    if input_metric not in (
        symmetric_distance(),
        insert_delete_distance(),
        change_one_distance(),
        hamming_distance(),
    ):
        raise ValueError("Certified rowwise inference requires a row-adjacency metric")

    if not isinstance(estimator, TransformationMixin):
        raise TypeError("Stage must implement TransformationMixin")
    if not callable(getattr(estimator, method, None)):
        raise TypeError(f"{type(estimator).__name__} has no sklearn {method!r} method")

    desc = input_domain.cast(SklearnFitDescriptor)
    # Certify the same snapshot that will execute inference. Any public schema
    # metadata recorded during certification stays with this snapshot.
    fitted = deepcopy(estimator)
    output = fitted.rowwise_output_domain(desc.X, method=method)
    if not isinstance(output, Domain):
        raise TypeError("rowwise_output_domain must return a Domain")
    array = output.cast(NPArray2Domain)
    if array.num_columns is None:
        raise ValueError("Certified rowwise inference requires a public output width")

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
