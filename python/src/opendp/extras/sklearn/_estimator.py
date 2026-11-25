"""Shared standalone fitting for OpenDP sklearn estimators."""

from abc import ABC, abstractmethod
from typing import Any, TypeVar

from opendp.context import _cast_measure, domain_of
from sklearn.utils.metaestimators import available_if

from opendp.extras.sklearn._domains import SklearnFit
from opendp.mod import Domain, Metric, Measure, Measurement

_EstimatorT = TypeVar("_EstimatorT", bound="MeasurementMixin")


def _inherited_method(name):
    """Expose only sklearn shortcuts supplied by the estimator's other mixins."""
    return lambda self: hasattr(super(MeasurementMixin, self), name)


class MeasurementMixin(ABC):
    @abstractmethod
    def make_measurement(
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

        measurement = self.make_measurement(
            domain, input_metric, output_measure, d_in, d_out
        )
        measurement = _cast_measure(measurement, output_measure, d_out)
        if not measurement.check(d_in, d_out):
            raise ValueError("Fitting measurement exceeds the privacy allowance")

        self._ingest_release(measurement(data))
        return self

    @available_if(_inherited_method("fit_transform"))
    def fit_transform(self, X, y=None, **fit_params):
        """Transform public records only when fitting consumes no privacy budget."""
        raise NotImplementedError(
            "Private fit_transform is unsupported; use fit with a privacy budget "
            "then call query.sklearn(fitted_estimator, method='transform')"
        )

    @available_if(_inherited_method("fit_predict"))
    def fit_predict(self, X, y=None, **fit_params):
        """Never expose row-level predictions on privately fitted records."""
        raise NotImplementedError(
            "Private fit_predict is unsupported; use fit with a privacy budget "
            "then use rowwise prediction on private records"
        )


class TransformationMixin(ABC):
    """Describe the output domain of rowwise sklearn inference.

    Implementations must process rows independently, preserve row count and
    alignment, and avoid data-dependent failures for inputs in the declared domain.
    Failures may depend on the released fitted model.
    The transform method must return the declared feature carrier directly.
    Prediction-like methods may return a one-dimensional array, which queries
    represent as one feature column. Unsupported methods must raise.
    """

    @abstractmethod
    def rowwise_output_domain(self, input_domain: Domain, *, method: str) -> Domain:
        """Return the output domain of a rowwise sklearn method."""
        ...
