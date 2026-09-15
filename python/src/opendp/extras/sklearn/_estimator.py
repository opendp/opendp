"""Base classes for OpenDP scikit-learn-style differentially private estimators.

Estimators carry algorithm parameters (for example, ``n_clusters`` or
``n_components``). The input domain, input metric, output measure, and
``d_in``/``d_out`` are supplied when constructing the measurement.

    est.fit(context.query(rho=0.5))
    context.query(rho=0.5).sklearn(est).release()

    measurement = est.make(input_domain, input_metric, output_measure, d_in, d_out)
    release = measurement(data)

Subclasses implement :meth:`make` following the calibrated constructor convention:
``measurement.map(d_in) <= d_out``.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING

from opendp._lib import import_optional_dependency

if TYPE_CHECKING:  # pragma: no cover
    from opendp.mod import Domain, Measure, Measurement, Metric
    from opendp.context import Query, _PartialConstructor


_sklearn_base = import_optional_dependency("sklearn.base", raise_error=False)


class _FallbackBaseEstimator:
    """Base used when the optional scikit-learn dependency is unavailable."""


_BaseEstimator = (
    _sklearn_base.BaseEstimator if _sklearn_base is not None else _FallbackBaseEstimator
)


class _DPFitMixin(ABC):
    """Shared OpenDP fitting behavior for sklearn-style estimators."""

    @abstractmethod
    def make(
        self,
        input_domain: "Domain",
        input_metric: "Metric",
        output_measure: "Measure",
        d_in,
        d_out,
    ) -> "Measurement":
        """Construct the OpenDP measurement used to fit this estimator.

        This method must not mutate ``self``. The returned measurement must satisfy
        ``map(d_in) <= d_out``.
        """
        raise NotImplementedError

    def then(
        self,
        output_measure: "Measure",
        d_in,
        d_out,
    ) -> "_PartialConstructor":
        """Partially apply ``make``, deferring the input domain and metric."""
        from opendp.mod import _PartialConstructor

        return _PartialConstructor(
            lambda input_domain, input_metric: self.make(
                input_domain, input_metric, output_measure, d_in, d_out
            )
        )

    @abstractmethod
    def _ingest_release(self, release) -> None:
        """Populate fitted sklearn attributes from a measurement release."""
        raise NotImplementedError

    @staticmethod
    def _reject_fit_params(fit_params) -> None:
        """Reject fit metadata that an estimator does not explicitly support."""
        if fit_params:
            names = ", ".join(sorted(fit_params))
            raise TypeError(f"Unexpected fit parameters: {names}")

    def _prepare_fit_query(self, X: "Query", y=None, **fit_params) -> "Query":
        """Normalize estimator-specific fit arguments into one input query.

        Supervised estimators may override this hook to interpret a symbolic target,
        and estimators supporting metadata may consume arguments such as
        ``sample_weight``. The default accepts neither.
        """
        if y is not None:
            raise TypeError(f"{type(self).__name__}.fit() does not accept y")
        self._reject_fit_params(fit_params)
        return X

    def fit(self, X: "Query", y=None, **fit_params) -> "_DPFitMixin":
        """Fit the estimator by releasing it through a Context query.

        The Context supplies the input domain/metric, output measure, ``d_in`` and
        ``d_out``. It releases the measurement, postprocesses the estimator-specific
        release into fitted attributes, and returns ``self``. The ``X, y=None,
        **fit_params`` signature follows the scikit-learn estimator
        convention, but ``X`` must be a symbolic OpenDP Query rather than an array.
        Subclasses normalize or reject ``y`` and fit metadata in the
        ``_prepare_fit_query`` hook.

        :param X: a Context query, e.g. ``context.query(rho=...)`` (optionally transformed)
        :param y: optional symbolic target, when supported by the estimator
        :param fit_params: estimator-specific fit metadata
        :return: ``self``, with the fitted attributes populated
        """
        from opendp.context import Query

        if not isinstance(X, Query):
            raise TypeError(
                "fit() expects X to be a Query from an OpenDP Context; "
                "use context.query(...) to allocate a privacy budget"
            )

        query = self._prepare_fit_query(X, y=y, **fit_params)
        if not isinstance(query, Query):
            raise TypeError("_prepare_fit_query() must return an OpenDP Query")

        return query.sklearn(self).release()


class _DPEstimator(_DPFitMixin, _BaseEstimator):  # type: ignore
    pass
