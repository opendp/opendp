"""Shared standalone fitting for OpenDP sklearn estimators."""

from abc import ABC, abstractmethod
from typing import Any, TypeVar

from opendp.context import _cast_measure, domain_of
from sklearn.base import BaseEstimator

from opendp.extras.sklearn._domains import SklearnFit
from opendp.mod import Domain, Metric, Measure, Measurement, Transformation

_EstimatorT = TypeVar("_EstimatorT", bound="DPEstimator")


class DPEstimator(BaseEstimator, ABC):
    @abstractmethod
    def make(
        self,
        input_domain: Domain,
        input_metric: Metric,
        output_measure: Measure,
        d_in: Any,
        d_out: Any,
    ) -> Measurement:
        """Construct a fitting measurement from SklearnFit to learned parameters.

        Do not mutate self. Snapshot configuration so later mutations cannot change
        the measurement's function or privacy map.
        """
        ...

    @abstractmethod
    def _ingest_release(self, release: Any) -> None:
        """Replace learned state with a complete fitted state from the release."""
        ...

    def fit(  # noqa: PYI019 - Python 3.10 docs compatibility
        self: _EstimatorT,
        X: Any,
        y: Any | None = None,
        *,
        privacy_unit: tuple[Metric, Any],
        privacy_loss: tuple[Measure, Any],
        domain: Domain | None = None,
    ) -> _EstimatorT:
        """Fit private data, mutating and returning self.

        Requires ``privacy_unit`` and ``privacy_loss``.

        ``domain`` should be constructed with ``sklearn_fit_domain(...)``.
        Otherwise, dtypes and feature count are inferred from the data.

        Context's ``query.sklearn(model).release()`` instead returns a fitted clone.
        """
        data = SklearnFit(X, y=y)
        if domain is None:
            domain = domain_of(data, infer=True)

        input_metric, d_in = privacy_unit
        output_measure, d_out = privacy_loss
        measurement = self.make(domain, input_metric, output_measure, d_in, d_out)
        measurement = _cast_measure(measurement, output_measure, d_out)
        if not measurement.check(d_in, d_out):
            raise ValueError("Fitting measurement exceeds the privacy allowance")

        self._ingest_release(measurement(data))
        return self


class DPTransformer(DPEstimator):
    """A DP estimator that supports stable post-fit transformations."""

    @abstractmethod
    def make_transform(
        self, input_domain: Domain, input_metric: Metric
    ) -> Transformation:
        """Construct a stable SklearnFit transformation preserving aligned metadata.

        Do not fit or modify self. Snapshot configuration and fitted parameters so
        later mutations cannot change the function or stability map.
        """
        ...
