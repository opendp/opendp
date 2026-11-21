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

from typing import Any

from sklearn.base import clone
from sklearn.frozen import FrozenEstimator
from sklearn.pipeline import Pipeline as SklearnPipeline

from opendp._internal import _new_pure_function
from opendp.extras.sklearn._rowwise import _make_rowwise_transform
from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.extras.sklearn._composition import (
    needs_private_fit,
    unwrap_frozen_estimator,
    allocate_private_stages,
    checked_child_fit,
)
from opendp.extras.sklearn._estimator import (
    MeasurementMixin,
    TransformationMixin,
    _set_fit_metadata,
)


def _passthrough(step):
    return step is None or (isinstance(step, str) and step == "passthrough")


def _snapshot(steps):
    if not steps:
        raise ValueError("Pipeline requires at least one step")
    names = [name for name, _ in steps]
    if any(not isinstance(name, str) or "__" in name for name in names) or len(
        names
    ) != len(set(names)):
        raise ValueError("Pipeline names must be unique strings without '__'")

    result = []
    for name, step in steps:
        if _passthrough(step):
            result.append((name, None))
        elif isinstance(
            unwrap_frozen_estimator(step), (MeasurementMixin, TransformationMixin)
        ):
            result.append((name, clone(step)))
        else:
            raise TypeError(
                f"Unsupported step {name!r}: use an OpenDP estimator "
                "or a trusted stable OpenDP transformer"
            )
    return tuple(result)


def _append(prefix, transform):
    return transform if prefix is None else prefix >> transform


class Pipeline(MeasurementMixin, TransformationMixin, SklearnPipeline):
    """Sequential private fit and stable transformations.

    Each private-fitting stage receives a share of the overall privacy budget,
    optionally selected with privacy_weights. Subsequent stages see stable
    preprocessing based only on previously released fitted parameters.

    make_measurement() returns child fitting releases, never the input records. Ingesting
    those releases populates a fresh fitted clone, consistent with
    Query.sklearn(...).release().
    """

    def __init__(self, steps, *, memory=None, verbose=False, privacy_weights=None):
        super().__init__(steps=steps, memory=memory, verbose=verbose)
        self.steps: list[tuple[str, Any]]
        self.privacy_weights = privacy_weights

    def _has_private_fit(self):
        return any(
            needs_private_fit(step) for _, step in self.steps if not _passthrough(step)
        )

    def _set_first_step_metadata(self, input_domain):
        if self.steps and not _passthrough(self.steps[0][1]):
            _set_fit_metadata(self.steps[0][1], input_domain)

    def fit(
        self,
        X,
        y=None,
        *,
        privacy_unit=None,
        privacy_loss=None,
        domain=None,
    ):
        """Fit privately if necessary; otherwise validate certified stable stages."""
        if self._has_private_fit():
            if privacy_unit is None or privacy_loss is None:
                raise TypeError("Private fit requires privacy_unit and privacy_loss")
            return super().fit(
                X,
                y,
                privacy_unit=privacy_unit,
                privacy_loss=privacy_loss,
                domain=domain,
            )
        if privacy_unit is not None or privacy_loss is not None:
            raise ValueError("Stable-only fit does not consume a privacy budget")
        if self.memory is not None:
            raise NotImplementedError("Private Pipeline does not support caching")

        from opendp.context import domain_of
        from opendp.extras.sklearn._domains import SklearnFit
        from opendp.metrics import symmetric_distance

        if domain is None:
            domain = domain_of(SklearnFit(X, y), infer=True)
        snapshots = _snapshot(self.steps)
        space = (domain, symmetric_distance())
        last = next(
            (
                i
                for i in range(len(snapshots) - 1, -1, -1)
                if snapshots[i][1] is not None
            ),
            None,
        )
        for i, (name, step) in enumerate(snapshots):
            if step is None:
                continue
            if i == last and not callable(getattr(step, "transform", None)):
                continue  # sklearn allows a final predictor
            if not callable(getattr(step, "transform", None)):
                raise TypeError(f"Stage {name!r} is not a sklearn transformer")
            space = _make_rowwise_transform(
                *space, step, method="transform"
            ).output_space

        self.steps = list(snapshots)
        for _, step in self.steps:
            if step is not None:
                step._dp_fitted_ = True
        self._set_first_step_metadata(domain)
        return self

    def rowwise_output_domain(self, feature_domain, *, method):
        """Certify that every stage of this sklearn inference is rowwise."""
        steps = [step for _, step in self.steps if not _passthrough(step)]
        if not steps:
            if method != "transform":
                raise ValueError(f"Pipeline has no method {method!r}")
            return feature_domain

        domain = feature_domain
        for i, step in enumerate(steps):
            operation = method if i == len(steps) - 1 else "transform"
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
        if self.memory is not None:
            raise NotImplementedError("Private Pipeline does not support caching")

        snapshots = _snapshot(self.steps)
        # Validate the stage types before executing a private measurement.
        last = next(
            (
                i
                for i in range(len(snapshots) - 1, -1, -1)
                if snapshots[i][1] is not None
            ),
            None,
        )
        for i, (name, step) in enumerate(snapshots):
            if step is None:
                continue
            if i != last and (
                not callable(getattr(step, "transform", None))
                or not isinstance(unwrap_frozen_estimator(step), TransformationMixin)
            ):
                raise TypeError(f"Intermediate stage {name!r} cannot transform records")
            if (
                not needs_private_fit(step)
                and not isinstance(unwrap_frozen_estimator(step), TransformationMixin)
                and not (i == last and isinstance(step, FrozenEstimator))
            ):
                raise TypeError(f"Stage {name!r} is not a certified stable stage")
        private_names = [
            name
            for name, step in snapshots
            if step is not None and needs_private_fit(step)
        ]
        compositor, budgets = allocate_private_stages(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            private_names,
            self.privacy_weights,
        )

        def execute(qbl):
            working = tuple(
                (name, None if step is None else clone(step))
                for name, step in snapshots
            )
            prefix = None
            space = (input_domain, input_metric)
            releases: list[Any] = []
            next_budget = iter(budgets)
            for i, (_, step) in enumerate(working):
                if step is None:
                    releases.append(None)
                    continue

                if needs_private_fit(step):
                    budget = next(next_budget)
                    query = checked_child_fit(
                        step,
                        working[i][0],
                        prefix,
                        space,
                        output_measure,
                        d_in,
                        budget,
                    )
                    release = qbl(query)
                    releases.append(release)
                    # A final predictor need not support transformation.
                    if not any(later is not None for _, later in working[i + 1 :]):
                        continue
                    step._ingest_release(release)
                else:
                    releases.append(None)

                if i == last:
                    continue
                trans = _make_rowwise_transform(*space, step, method="transform")
                prefix = _append(prefix, trans)
                space = trans.output_space
            return input_domain, tuple(releases)

        return compositor >> _new_pure_function(execute)

    def _ingest_release(self, release):
        """Build fitted stages after the outer Context accounts for release."""
        input_domain, releases = release
        if len(releases) != len(self.steps):
            raise ValueError("Unexpected number of fitting releases")

        self.steps = [
            (name, None if _passthrough(step) else clone(step))
            for name, step in self.steps
        ]
        for (_, step), stage_release in zip(self.steps, releases):
            if step is None:
                continue
            if needs_private_fit(step):
                step._ingest_release(stage_release)
        self._set_first_step_metadata(input_domain)
