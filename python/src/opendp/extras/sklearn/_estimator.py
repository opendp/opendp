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
    measurement = est.new_measurement(input_domain, input_metric, output_measure, d_in, d_out)
    release = measurement(data)

Subclasses implement :meth:`new_measurement`, which must follow the calibrated
constructor convention ``make_*(input_domain, input_metric, output_measure, d_in,
d_out, *, <algorithm params>)`` -- the units of ``d_in`` are defined by the input
metric and the units of ``d_out`` by the output measure.  The estimator computes its
own internal noise so that ``measurement.map(d_in) <= d_out``; there is no ``scale``
or ``epsilon``/``rho`` parameter on the estimator surface.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    from opendp.mod import Domain, Measure, Measurement, Metric
    from opendp.context import Query, _PartialConstructor


class SklearnEstimator:
    """Base class for OpenDP scikit-learn-style DP estimators.

    This is the type accepted by the Context API's ``.sklearn(...)`` query method.
    Subclasses must implement :meth:`new_measurement` and :meth:`_ingest_release`.
    """

    def new_measurement(
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
        :return: a Measurement releasing the fitted model
        """
        raise NotImplementedError

    def then_measurement(
        self,
        output_measure: "Measure",
        d_in,
        d_out,
    ) -> "_PartialConstructor":
        """Partially apply :meth:`new_measurement`, deferring ``input_domain`` and ``input_metric``.

        Used by the Context API's ``.sklearn(...)`` query method to chain the estimator
        onto the current query space.

        :param output_measure: measure in whose units ``d_out`` is expressed
        :param d_in: upper bound on the distance between adjacent input datasets
        :param d_out: privacy budget, in the units of ``output_measure``
        :return: a partial constructor awaiting ``(input_domain, input_metric)``
        """
        from opendp.mod import _PartialConstructor

        return _PartialConstructor(
            lambda input_domain, input_metric: self.new_measurement(
                input_domain, input_metric, output_measure, d_in, d_out
            )
        )

    def _ingest_release(self, release) -> None:
        """Store the released model on ``self`` (sets the fitted ``*_`` attributes).

        :param release: the value produced by the fitted measurement
        """
        raise NotImplementedError

    def fit(self, query: "Query") -> "SklearnEstimator":
        """Fit the estimator by releasing it through a Context query.

        The Context supplies the input domain/metric, output measure, ``d_in`` and
        ``d_out``; this method calibrates and releases, then stores the fitted model on
        ``self``.

        :param query: a Context query, e.g. ``context.query(rho=...)`` (optionally with
            prior transformations)
        :return: ``self``, with the fitted attributes populated
        """
        release = query.sklearn(self).release()
        self._ingest_release(release)
        return self
