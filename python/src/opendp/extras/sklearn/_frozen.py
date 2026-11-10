"""Use fitted sklearn methods as stable OpenDP transformations."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable

from sklearn.base import BaseEstimator
from opendp.mod import Domain


@dataclass(frozen=True)
class _RowwiseSpec:
    """Precursor for building row-by-row Transformations from multiple aligned computations.

    - Outcomes of the function on its given input domain are members of the output_domain.
    - function is total (infallible) on its input domain.
    - function is row-by-row and preserves alignment.
    """

    function: Callable
    output_domain: Domain



def supports_rowwise(step):
    """Whether an OpenDP stage declares a certified rowwise implementation."""
    from opendp.extras.sklearn._estimator import DPEstimator

    if isinstance(step, FrozenTransformer):
        if not isinstance(step.estimator, DPEstimator):
            return False
        if step.method == "transform" and supports_rowwise(step.estimator):
            return True
        return callable(getattr(step.estimator, "_make_rowwise_method", None))

    if not isinstance(step, DPEstimator) or not callable(
        getattr(step, "make_rowwise", None)
    ):
        return False
    nested = getattr(step, "_supports_rowwise", None)
    return nested() if callable(nested) else True


def _validate_rowwise_spec(spec):
    if (
        not isinstance(spec, _RowwiseSpec)
        or not callable(spec.function)
        or not isinstance(spec.output_domain, Domain)
    ):
        raise TypeError("Certified method must return a _RowwiseSpec with a Domain")
    return spec


def rowwise_spec(step, input_domain):
    """Obtain the trusted rowwise contract of a fitted or public stage."""
    if not supports_rowwise(step):
        raise TypeError(f"{type(step).__name__} has no certified rowwise implementation")
    return _validate_rowwise_spec(step.make_rowwise(input_domain))


class FrozenTransformer(BaseEstimator):
    """Use a fitted estimator's selected method as a stable transformation."""

    def __init__(self, estimator, method="transform"):
        self.estimator = estimator
        self.method = method

    def __sklearn_clone__(self):
        # Like sklearn's FrozenEstimator, cloning cannot discard fitted state.
        return self

    def fit(self, X, y=None):
        """A frozen model is already fitted; do not refit on private records."""
        return self

    def transform(self, X):
        """Ordinary sklearn inference on public data."""
        return getattr(self.estimator, self.method)(X)

    def _has_private_fit(self):
        return False

    def make_rowwise(self, input_domain):
        if not supports_rowwise(self):
            raise TypeError(
                "Rowwise transformations require a certified OpenDP estimator method"
            )
        if self.method == "transform" and supports_rowwise(self.estimator):
            return rowwise_spec(self.estimator, input_domain)
        maker = self.estimator._make_rowwise_method
        return _validate_rowwise_spec(maker(self.method, input_domain))

    def make_transform(self, domain, metric):
        from opendp.extras.sklearn._estimator import DPTransformer
        if self.method == "transform" and isinstance(self.estimator, DPTransformer):
            return self.estimator.make_transform(domain, metric)

        from opendp._internal import _make_transformation
        from opendp.mod import DataFrameDomain
        from opendp.extras.numpy import NPArray2Domain
        from opendp.metrics import (
            symmetric_distance, insert_delete_distance,
            change_one_distance, hamming_distance,
        )

        if metric not in (
            symmetric_distance(), insert_delete_distance(),
            change_one_distance(), hamming_distance(),
        ):
            raise ValueError(
                "Generic rowwise stability requires a row-adjacency metric; "
                "identifier-aware stability needs a certified make_transform"
            )
        from opendp.extras.sklearn._domains import (
            SklearnFit, SklearnFitDescriptor, sklearn_fit_domain,
        )

        spec = self.make_rowwise(domain)
        desc = domain.cast(SklearnFitDescriptor)
        output = spec.output_domain

        try:
            output.cast(NPArray2Domain)
            feature_output = True
        except ValueError:
            feature_output = isinstance(output, DataFrameDomain)

        if feature_output:
            return _make_transformation(
                domain, metric, sklearn_fit_domain(output, desc.y), metric,
                lambda data: SklearnFit(spec.function(data.X), data.y),
                lambda d: d,
            )

        if desc.y is not None:
            raise TypeError("Cannot retain target for this rowwise output domain")
        return _make_transformation(
            domain, metric, output, metric,
            lambda data: spec.function(data.X), lambda d: d,
        )
