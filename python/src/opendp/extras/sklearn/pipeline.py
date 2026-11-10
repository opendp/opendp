"""Private sklearn Pipeline with sequential, adaptive OpenDP composition.

Stable stages use certified frozen adapters or wholly stable compositions.
Private fitting uses adaptive composition, never ordinary sklearn.fit().
"""

from __future__ import annotations

from sklearn.base import clone
from sklearn.pipeline import Pipeline as SklearnPipeline

from opendp._internal import _make_transformation, _new_pure_function
from opendp.extras.sklearn._composition import (
    allocate_private_stages, checked_child_fit,
)
from opendp.extras.sklearn._estimator import (
    DPEstimator, DPTransformer, needs_private_fit,
)
from opendp.extras.sklearn._frozen import FrozenTransformer


def _passthrough(step):
    return step is None or (isinstance(step, str) and step == "passthrough")


def _snapshot(steps):
    if not steps:
        raise ValueError("Pipeline requires at least one step")
    names = [name for name, _ in steps]
    if (
        any(not isinstance(name, str) or "__" in name for name in names)
        or len(names) != len(set(names))
    ):
        raise ValueError("Pipeline names must be unique strings without '__'")

    result = []
    for name, step in steps:
        if _passthrough(step):
            result.append((name, None))
        elif isinstance(step, (DPEstimator, FrozenTransformer)):
            result.append((name, clone(step)))
        else:
            raise TypeError(
                f"Unsupported step {name!r}: use an OpenDP estimator "
                "or a trusted stable sklearn adapter"
            )
    return tuple(result)


def _append(prefix, transform):
    return transform if prefix is None else prefix >> transform


def _identity(domain, metric):
    return _make_transformation(
        domain, metric, domain, metric, lambda data: data, lambda d: d,
    )


def _stable(step, domain, metric):
    """Compile a frozen or wholly stable OpenDP stage."""
    if isinstance(step, (FrozenTransformer, DPTransformer)) and not needs_private_fit(step):
        return step.make_transform(domain, metric)
    raise TypeError(f"{type(step).__name__} is not a trusted stable stage")


def _fitted_transform(step, domain, metric):
    if not isinstance(step, DPTransformer):
        raise TypeError(
            f"{type(step).__name__} cannot transform records after fitting"
        )
    return step.make_transform(domain, metric)


class Pipeline(SklearnPipeline, DPTransformer):
    """Sequential private fit and stable transformations.

    Each private-fitting stage receives a share of the overall privacy budget,
    optionally selected with privacy_weights. Subsequent stages see stable
    preprocessing based only on previously released fitted parameters.

    make() returns child fitting releases, never the input records. Ingesting
    those releases populates a fresh fitted clone, consistent with
    Query.sklearn(...).release().
    """

    def __init__(self, steps, *, memory=None, verbose=False, privacy_weights=None):
        super().__init__(steps=steps, memory=memory, verbose=verbose)
        self.privacy_weights = privacy_weights

    def _has_private_fit(self):
        return any(needs_private_fit(step) for _, step in self.steps if not _passthrough(step))

    def _supports_rowwise(self):
        from opendp.extras.sklearn._frozen import supports_rowwise

        return all(
            _passthrough(step) or supports_rowwise(step)
            for _, step in self.steps
        )

    def make_rowwise(self, input_domain):
        """Compose certified rowwise operations, retaining column alignment."""
        from opendp.extras.sklearn._domains import (
            SklearnFitDescriptor, sklearn_fit_domain,
        )
        from opendp.extras.sklearn._frozen import _RowwiseSpec, rowwise_spec
        from opendp.extras.numpy import NPArray2Domain
        from opendp.mod import DataFrameDomain

        if not self._supports_rowwise():
            raise TypeError("Pipeline contains a non-rowwise stage")
        desc = input_domain.cast(SklearnFitDescriptor)
        steps = [step for _, step in self.steps if not _passthrough(step)]
        space = input_domain
        functions = []
        output = desc.X
        for i, step in enumerate(steps):
            spec = rowwise_spec(step, space)
            functions.append(spec.function)
            output = spec.output_domain
            if i + 1 < len(steps):
                if not isinstance(output, DataFrameDomain):
                    output.cast(NPArray2Domain)
                space = sklearn_fit_domain(output, desc.y)

        def transform(X):
            for function in functions:
                X = function(X)
            return X

        return _RowwiseSpec(transform, output)

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        """Construct one adaptively composed DP fitting measurement.

        Errors propagate; the compositor accounts for failed invocations.
        """
        if self.memory is not None:
            raise NotImplementedError("Private Pipeline does not support caching")

        snapshots = _snapshot(self.steps)
        # Validate capabilities before any private measurement is invoked.
        for name, step in snapshots[:-1]:
            if step is not None and not isinstance(step, (DPTransformer, FrozenTransformer)):
                raise TypeError(f"Intermediate stage {name!r} cannot transform records")
        for name, step in snapshots:
            if step is not None and not needs_private_fit(step) and not isinstance(
                step, (DPTransformer, FrozenTransformer)
            ):
                raise TypeError(f"Stage {name!r} is not a trusted stable stage")
        private_names = [
            name for name, step in snapshots
            if step is not None and needs_private_fit(step)
        ]
        compositor, budgets = allocate_private_stages(
            input_domain, input_metric, output_measure, d_in, d_out,
            private_names, self.privacy_weights,
        )

        def execute(qbl):
            working = tuple(
                (name, None if step is None else clone(step))
                for name, step in snapshots
            )
            prefix = None
            space = (input_domain, input_metric)
            releases = []
            next_budget = iter(budgets)
            for i, (_, step) in enumerate(working):
                if step is None:
                    releases.append(None)
                    continue

                if needs_private_fit(step):
                    budget = next(next_budget)
                    query = checked_child_fit(
                        step, working[i][0], prefix, space,
                        output_measure, d_in, budget,
                    )
                    release = qbl(query)
                    releases.append(release)
                    # A final predictor need not support transformation.
                    if not any(later is not None for _, later in working[i + 1:]):
                        continue
                    step._ingest_release(release)
                    trans = _fitted_transform(step, *space)
                else:
                    trans = _stable(step, *space)
                    releases.append(None)

                prefix = _append(prefix, trans)
                space = trans.output_space
            return tuple(releases)

        return compositor >> _new_pure_function(execute)

    def _ingest_release(self, releases):
        """Build fitted stages after the outer Context accounts for release."""
        if len(releases) != len(self.steps):
            raise ValueError("Unexpected number of fitting releases")

        self.steps = [
            (name, None if _passthrough(step) else clone(step))
            for name, step in self.steps
        ]
        for (_, step), release in zip(self.steps, releases):
            if step is None:
                continue
            if needs_private_fit(step):
                step._ingest_release(release)

    def make_transform(self, input_domain, input_metric):
        """Compose fitted or public stable stages into one transformation."""
        prefix, space = None, (input_domain, input_metric)
        for _, step in self.steps:
            if _passthrough(step):
                continue
            if needs_private_fit(step):
                trans = _fitted_transform(step, *space)
            else:
                trans = _stable(step, *space)
            prefix = _append(prefix, trans)
            space = trans.output_space
        return prefix if prefix is not None else _identity(input_domain, input_metric)

    def fit(self, X, y=None, *, privacy_unit, privacy_loss, domain=None):
        """Fit privately, requiring explicit privacy parameters."""
        return DPEstimator.fit(
            self, X, y,
            privacy_unit=privacy_unit, privacy_loss=privacy_loss, domain=domain,
        )

    def fit_transform(self, X, y=None, **kwargs):
        """Do not release unprotected transformations of the training data."""
        raise NotImplementedError(
            "Private fit_transform is unsupported; fit privately, then "
            "transform public data or use .sklearn(fitted.as_transformer())"
        )

    def fit_predict(self, X, y=None, **kwargs):
        """Do not release unprotected predictions on training records."""
        raise NotImplementedError(
            "Private fit_predict is unsupported; fit privately, then "
            "predict on public data or use .sklearn(fitted.as_transformer('predict'))"
        )
