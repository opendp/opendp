"""This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.pipeline``.

Private sklearn Pipeline with sequential, adaptive OpenDP composition.

Stable stages are trusted OpenDP transformers, including frozen estimators.
Private fitting uses adaptive composition, never ordinary sklearn.fit().
"""

from __future__ import annotations

from copy import deepcopy
from typing import Any

from sklearn.frozen import FrozenEstimator
from sklearn.pipeline import Pipeline as SklearnPipeline

from opendp._internal import _new_pure_function
from opendp.extras.sklearn._rowwise import _make_rowwise_transform
from opendp.extras.sklearn._composition import (
    _CompositeMixin,
    needs_private_fit,
    unwrap_frozen_estimator,
    allocate_private_stages,
    checked_child_fit,
    fit_child,
    snapshot,
)
from opendp.extras.sklearn._estimator import (
    MeasurementMixin,
    TransformationMixin,
)


def _passthrough(step):
    return step is None or (isinstance(step, str) and step == "passthrough")


class Pipeline(_CompositeMixin, TransformationMixin, SklearnPipeline):
    """Sequential private fit and stable transformations.

    Each private-fitting stage receives a share of the overall privacy budget,
    optionally selected with privacy_weights. Subsequent stages see stable
    preprocessing based only on previously released fitted parameters.

    make_measurement() returns fitted children constructed from accounted releases,
    never the input records.
    """

    def __init__(self, steps, *, privacy_weights=None):
        super().__init__(steps=steps)
        self.steps: list[tuple[str, Any]]
        self.privacy_weights = privacy_weights

    def _has_private_fit(self):
        return any(
            needs_private_fit(step) for _, step in self.steps if not _passthrough(step)
        )

    def __sklearn_is_fitted__(self):
        return getattr(self, "_dp_fitted_", False) or super().__sklearn_is_fitted__()

    def _validated_steps(self):
        """Validate public configuration, normalizing passthrough steps."""
        if not self.steps:
            raise ValueError("Pipeline requires at least one step")
        names = [name for name, _ in self.steps]
        if not all(isinstance(name, str) for name in names):
            raise ValueError("Pipeline names must be strings")
        self._validate_names(names)

        steps = []
        for i, (name, step) in enumerate(self.steps):
            if _passthrough(step):
                steps.append((name, None))
                continue
            estimator = unwrap_frozen_estimator(step)
            if not isinstance(estimator, (MeasurementMixin, TransformationMixin)):
                raise TypeError(
                    f"Unsupported step {name!r}: use an OpenDP estimator "
                    "or a trusted stable OpenDP transformer"
                )
            final = i == len(self.steps) - 1
            if not final and (
                not isinstance(estimator, TransformationMixin)
                or not callable(getattr(step, "transform", None))
            ):
                raise TypeError(f"Intermediate stage {name!r} cannot transform records")
            if (
                not needs_private_fit(step)
                and not isinstance(estimator, TransformationMixin)
                and not (final and isinstance(step, FrozenEstimator))
            ):
                raise TypeError(f"Stage {name!r} is not a certified stable stage")
            steps.append((name, step))
        return steps

    def _fit_from_domain(self, input_domain):
        from opendp.metrics import symmetric_distance

        return self._fit_steps(
            self._validated_steps(), (input_domain, symmetric_distance())
        )

    @staticmethod
    def _fit_steps(steps, space, fit_private=None):
        """Fit in order, certifying only the preprocessing of later stages."""
        prefix = None
        for i, (name, step) in enumerate(steps):
            if step is None:
                continue
            if needs_private_fit(step):
                fit_private(step, name, prefix, space)
            else:
                fit_child(step, space[0])

            if i != len(steps) - 1 or (
                fit_private is None and callable(getattr(step, "transform", None))
            ):
                trans = _make_rowwise_transform(*space, step, method="transform")
                prefix = trans if prefix is None else prefix >> trans
                space = trans.output_space
        return steps

    def rowwise_output_domain(self, feature_domain, *, method):
        """Certify that every stage of this sklearn inference is rowwise."""
        if not self.steps or _passthrough(self.steps[-1][1]):
            if method != "transform":
                raise ValueError(f"Pipeline has no method {method!r}")

        domain = feature_domain
        for i, (_, step) in enumerate(self.steps):
            if _passthrough(step):
                continue
            operation = method if i == len(self.steps) - 1 else "transform"
            certified = unwrap_frozen_estimator(step)
            if not isinstance(certified, TransformationMixin) or not callable(
                getattr(certified, operation, None)
            ):
                raise TypeError(f"Pipeline stage cannot certify {operation!r}")
            domain = certified.rowwise_output_domain(domain, method=operation)
        return domain

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        """Construct one adaptively composed DP fitting measurement.

        Errors propagate; the compositor accounts for failed invocations.
        """
        prototype = snapshot(self)
        steps = prototype._validated_steps()
        private_names = [
            name for name, step in steps if step is not None and needs_private_fit(step)
        ]
        compositor, budgets = allocate_private_stages(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            private_names,
            prototype.privacy_weights,
        )

        def execute(qbl):
            next_budget = iter(budgets)

            def fit_private(step, name, prefix, space):
                measurement = checked_child_fit(
                    step, name, prefix, space, output_measure, d_in, next(next_budget)
                )
                fit_child(step, space[0], qbl, measurement)

            return prototype._fit_steps(
                deepcopy(steps), (input_domain, input_metric), fit_private
            )

        return compositor >> _new_pure_function(execute, TO="ExtrinsicObject")

    def _ingest_release(self, release):
        self.steps = list(release)
        self._dp_fitted_ = True
