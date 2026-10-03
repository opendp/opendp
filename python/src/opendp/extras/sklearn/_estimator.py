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

    def _adapt_fit_query(self, query: "Query") -> "Query":
        """Lower orchestration inputs to this estimator's natural input space."""
        return query

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

        The Query already represents all private fit inputs, including private
        targets and row-aligned metadata. Estimators may later support separate
        fit arguments as explicitly public auxiliary inputs. The default accepts
        neither ``y`` nor fit metadata.
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

        :param X: a Context query containing all private fit inputs (optionally transformed)
        :param y: optional public auxiliary input, when explicitly supported
        :param fit_params: estimator-specific public fit metadata
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


class _DPXEstimator(_DPEstimator):
    """Sklearn bridge for estimators whose framework measurement consumes X only.

    ``make`` and ``then`` retain the algorithm's natural domain. Only the sklearn
    query bridge projects aligned inputs; the caller's query remains intact for
    subsequent pipeline steps.
    """

    def _adapt_fit_query(self, query: "Query") -> "Query":
        from opendp.extras.sklearn._aligned import AlignedDomain, then_project_X
        from opendp.mod import Transformation

        chain = query._chain
        if isinstance(chain, tuple):
            domain = chain[0]
        elif isinstance(chain, Transformation):
            domain = chain.output_domain
        else:
            return query  # The Query bridge diagnoses unfinished/invalid chains.

        if isinstance(getattr(domain, "descriptor", None), AlignedDomain):
            return query.new_with(chain=chain >> then_project_X())
        return query
