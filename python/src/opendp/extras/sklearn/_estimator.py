"""Base class shared by OpenDP scikit-learn-style differentially private estimators.

An estimator instance carries *only algorithm hyperparameters* (e.g. ``n_clusters``,
``n_components``).  Everything privacy- or data-related -- the input domain, input
metric, output measure, and the ``d_in``/``d_out`` distances -- is supplied later,
either by the Context API or by an explicit call.  This keeps one estimator instance
reusable across contexts and lets all of the following hit a single code path:

    # via a Context query (the Context fills domain/metric/measure/d_in/d_out):
    est.fit(context.query(rho=0.5))
    context.query(rho=0.5).sklearn(est).release()

    # directly, supplying the pieces yourself:
    measurement = est.make(input_domain, input_metric, output_measure, d_in, d_out)
    release = measurement(data)

Subclasses implement :meth:`make`, which must follow the calibrated
constructor convention ``make_*(input_domain, input_metric, output_measure, d_in,
d_out, *, <algorithm params>)`` -- the units of ``d_in`` are defined by the input
metric and the units of ``d_out`` by the output measure.  The estimator computes its
own internal noise so that ``measurement.map(d_in) <= d_out``; there is no ``scale``
or ``epsilon``/``rho`` parameter on the estimator surface.
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


class SklearnEstimator(_BaseEstimator, ABC):  # type: ignore
    """Base class for OpenDP scikit-learn-style DP estimators.

    This is the type accepted by the Context API's ``.sklearn(...)`` query method.
    Subclasses must implement :meth:`make` and :meth:`_ingest_release`.
    """

    @abstractmethod
    def make(
        self,
        input_domain: "Domain",
        input_metric: "Metric",
        output_measure: "Measure",
        d_in,
        d_out,
    ) -> "Measurement":
        """Construct the measurement that releases a fitted model.

        Subclasses implement this following the calibrated-mechanism convention; the
        returned measurement must satisfy ``map(d_in) <= d_out``.

        :param input_domain: domain of the input dataset
        :param input_metric: metric of the input dataset
        :param output_measure: measure in whose units ``d_out`` is expressed
        :param d_in: upper bound on the distance between adjacent input datasets
        :param d_out: privacy budget, in the units of ``output_measure``
        """
        raise NotImplementedError

    def then(
        self,
        output_measure: "Measure",
        d_in,
        d_out,
    ) -> "_PartialConstructor":
        """Partially apply :meth:`make`, deferring ``input_domain`` and ``input_metric``.

        Used by the Context API's ``.sklearn(...)`` query method to chain the estimator
        onto the current query space.

        :param output_measure: measure in whose units ``d_out`` is expressed
        :param d_in: upper bound on the distance between adjacent input datasets
        :param d_out: privacy budget, in the units of ``output_measure``
        :return: a partial constructor awaiting ``(input_domain, input_metric)``
        """
        from opendp.mod import _PartialConstructor

        return _PartialConstructor(
            lambda input_domain, input_metric: self.make(
                input_domain, input_metric, output_measure, d_in, d_out
            )
        )

    @abstractmethod
    def _ingest_release(self, release) -> None:
        """Store the released model on ``self`` (sets the fitted ``*_`` attributes).

        :param release: the value produced by the fitted measurement
        """
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

    def fit(self, X: "Query", y=None, **fit_params) -> "SklearnEstimator":
        """Fit the estimator by releasing it through a Context query.

        The Context supplies the input domain/metric, output measure, ``d_in`` and
        ``d_out``; this method calibrates and releases, then stores the fitted model on
        ``self``. The ``X, y=None, **fit_params`` signature follows the scikit-learn estimator
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

        release = query.sklearn(self).release()
        self._ingest_release(release)
        return self
