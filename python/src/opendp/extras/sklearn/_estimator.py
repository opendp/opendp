"""Shared standalone fitting for OpenDP sklearn estimators."""

from abc import ABC, abstractmethod

from opendp.context import _cast_measure, domain_of
from sklearn.base import BaseEstimator

from opendp.extras.sklearn._domains import SklearnFit, sklearn_fit_domain


class DPEstimator(BaseEstimator, ABC):
    @abstractmethod
    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        """Construct a fitting measurement without mutating self.

        The measurement consumes SklearnFit and releases learned parameters.
        Capture configuration values and copy mutable parameters as needed so
        later changes to self cannot invalidate the function or privacy map.
        """
        raise NotImplementedError  # pragma: no cover

    @abstractmethod
    def _ingest_release(self, release):
        """Replace learned state with a complete fitted state from the release."""
        raise NotImplementedError  # pragma: no cover

    def fit(
        self,
        X,
        y=None,
        *,
        domain=None,
        privacy_unit=None,
        privacy_loss=None,
    ):
        """Fit private data, mutating and returning self.

        ``privacy_unit=dp.unit_of(...)`` and ``privacy_loss=dp.loss_of(...)``
        are required. ``domain`` describes the complete SklearnFit input;
        if omitted, feature width and dtype are assumed public and inferred.

        Context fitting instead uses ``query.sklearn(model).release()``, which
        releases a new fitted clone without mutating the supplied estimator.
        Each standalone fit incurs its own privacy loss. Per-record transform
        outputs are not private releases for a nonprivate downstream estimator.
        """
        if privacy_unit is None or privacy_loss is None:
            raise TypeError("fit() requires explicit privacy_unit and privacy_loss")

        data = SklearnFit(X, y=y)
        if domain is None:
            domain = sklearn_fit_domain(
                domain_of(X, infer=True),
                y=None if data.y is None else domain_of(data.y, infer=True),
            )

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
    def make_transform(self, input_domain, input_metric):
        """Construct a stable transformation using fitted parameters.

        Consumes and returns SklearnFit, preserving aligned metadata.
        Must not fit or modify the estimator. Snapshot mutable configuration and
        fitted parameters so later changes to self cannot invalidate the
        function or stability map.
        """
        raise NotImplementedError  # pragma: no cover
