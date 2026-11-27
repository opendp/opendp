"""This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.compose``.

Privacy-aware sklearn ColumnTransformer.

Private fitting is an OpenDP adaptive composition. Stable inference uses a
trusted rowwise contract: each branch emits exactly one value per input row
in the same order. Errors must not depend on private input data; they may
depend on the released fitted estimator.

Do not treat arbitrary sklearn transformers or OpenDP Transformations as
rowwise. Symmetric-distance stability alone does not imply alignment.
"""

from __future__ import annotations

from copy import copy, deepcopy
from sys import modules

import numpy as np
from sklearn.utils import _safe_indexing
from sklearn.compose import ColumnTransformer as SklearnColumnTransformer
from sklearn.exceptions import NotFittedError
from sklearn.utils.validation import _check_feature_names_in, check_is_fitted

from opendp._internal import _make_transformation, _new_pure_function
from opendp.domains import dataframe_domain, series_domain, option_domain
from opendp.extras.numpy import NPArray2Domain, array2_domain
from opendp.extras.sklearn._domains import (
    SklearnFit,
    SklearnFitDescriptor,
    sklearn_fit_domain,
)
from opendp.extras.numpy._make_dataframe_to_numpy import _numpy_dtype
from opendp.extras.numpy._make_numpy_to_dataframe import _array_element_domain
from opendp.mod import DataFrameDomain, Domain
from opendp.extras.sklearn._estimator import (
    TransformationMixin,
    _finish_fit,
)
from opendp.extras.sklearn._composition import (
    _CompositeMixin,
    needs_private_fit,
    unwrap_frozen_estimator,
    allocate_private_stages,
    checked_child_fit,
    fit_child,
    snapshot,
)
from opendp.metrics import symmetric_distance


def _selector(columns, width, names=None):
    """Resolve selectors against public positions, never against private rows."""
    if isinstance(columns, str):
        columns = [columns]
    if isinstance(columns, (list, tuple, np.ndarray)):
        columns = list(columns)
        if names is not None and all(isinstance(c, str) for c in columns):
            if any(c not in names for c in columns):
                raise ValueError("Unknown feature columns")
            columns = [names.index(c) for c in columns]
        elif not (
            all(isinstance(c, (bool, np.bool_)) for c in columns)
            or all(
                isinstance(c, (int, np.integer)) and not isinstance(c, (bool, np.bool_))
                for c in columns
            )
        ):
            raise TypeError("Only public positional column selectors are supported")
    elif not isinstance(columns, (slice, int, np.integer)) or isinstance(
        columns, (bool, np.bool_)
    ):
        raise TypeError("Only public positional column selectors are supported")

    if (
        isinstance(columns, list)
        and columns
        and isinstance(columns[0], (bool, np.bool_))
    ):
        if len(columns) != width:
            raise ValueError("Boolean selector must match the feature width")
    try:
        selected = _safe_indexing(np.arange(width), columns)
    except IndexError as exc:
        raise ValueError("Column index is out of bounds") from exc
    indices = tuple(np.atleast_1d(selected).tolist())
    if len(set(indices)) != len(indices):
        raise ValueError("Repeated column indices are not supported")
    return indices


def _input_features(features):
    if isinstance(features, DataFrameDomain):
        return features
    if not isinstance(features, NPArray2Domain):
        features = features.cast(NPArray2Domain)
    if features.num_columns is None:
        raise ValueError("ColumnTransformer needs a public number of input features")
    return features


def _selected_domain(x, indices):
    """Selecting feature coordinates cannot increase a row norm."""
    if isinstance(x, DataFrameDomain):
        names = x.columns
        return dataframe_domain([x.get_series_domain(names[i]) for i in indices])
    x = x if isinstance(x, NPArray2Domain) else x.cast(NPArray2Domain)
    origin = None if x.origin is None else x.origin[list(indices)]
    cardinalities = None if x.cardinalities is None else x.cardinalities[list(indices)]
    output_x = array2_domain(
        T=x.T,
        num_columns=len(indices),
        size=x.size,
        nan=x.nan,
        norm=x.norm,
        p=x.p,
        origin=origin,
        cardinalities=cardinalities,
    )
    return output_x


def _select_transform(domain, metric, indices):
    """Select public columns while preserving row alignment and targets."""
    desc = domain.cast(SklearnFitDescriptor)
    output_domain = sklearn_fit_domain(_selected_domain(desc.X, indices), desc.y)
    names = (
        tuple(desc.X.columns[i] for i in indices)
        if isinstance(desc.X, DataFrameDomain)
        else None
    )

    def select(data):
        X = data.X.select(names) if names is not None else data.X[:, indices]
        return SklearnFit(X, data.y)

    return _make_transformation(
        domain,
        metric,
        output_domain,
        metric,
        select,
        lambda d: d,
    )


def _output_columns(domain, name):
    """Describe only the output carriers ColumnTransformer can concatenate."""
    if isinstance(domain, DataFrameDomain):
        columns = []
        for column in domain.columns:
            series = domain.get_series_domain(column)
            atom = series.element_domain
            dtype = (
                np.dtype("object")
                if str(atom.carrier_type) == "String"
                else _numpy_dtype(atom.carrier_type)
            )
            columns.append(
                (
                    option_domain(atom) if series.nullable else atom,
                    dtype,
                )
            )
    else:
        try:
            array = domain.cast(NPArray2Domain)
        except ValueError as exc:
            raise TypeError(
                f"Branch {name!r} output must be DataFrameDomain or NPArray2Domain"
            ) from exc
        if array.num_columns is None:
            raise ValueError(f"Branch {name!r} needs a public output width")
        dtype = _numpy_dtype(array.T)
        columns = [(_array_element_domain(array), dtype)] * array.num_columns

    if not columns:
        raise ValueError(f"Branch {name!r} must emit at least one column")
    return columns


def _rowwise_transformer(step, name):
    """Require a rowwise transform without inspecting private records."""
    transformer = unwrap_frozen_estimator(step)
    if not isinstance(transformer, TransformationMixin) or not all(
        callable(getattr(transformer, method, None))
        for method in ("transform", "rowwise_output_domain")
    ):
        raise TypeError(
            f"Branch {name!r} must implement a rowwise transform; "
            "fit predictors separately"
        )
    return transformer


class ColumnTransformer(_CompositeMixin, TransformationMixin, SklearnColumnTransformer):
    """Privately fit parallel branches and concatenate row-aligned outputs.

    Uses sklearn's (name, transformer, columns) configuration. Private branches
    must implement MeasurementMixin; all transforming branches must declare a
    rowwise output domain. Passthrough and drop are supported.

    Named and positional selectors work on eager DataFrames and NumPy matrices.
    Feature names follow sklearn's get_feature_names_out contract;
    output domains describe the actual output schema separately.
    Ordinary sklearn fitting is never invoked on private data.
    """

    def __init__(
        self,
        transformers,
        *,
        remainder="drop",
        verbose_feature_names_out=True,
        privacy_weights=None,
    ):
        super().__init__(
            transformers,
            remainder=remainder,
            verbose_feature_names_out=verbose_feature_names_out,
        )
        self.privacy_weights = privacy_weights
        self._dp_output = "default"

    def _has_private_fit(self):
        return any(needs_private_fit(step) for _, step, _ in self.transformers)

    def _fit_from_domain(self, input_domain):
        """Prepare stable branches in the owned configuration."""
        desc = input_domain.cast(SklearnFitDescriptor)
        branches = self._branches(_input_features(desc.X))
        for name, step, indices in branches:
            if not isinstance(step, str):
                _rowwise_transformer(step, name)
                selected = sklearn_fit_domain(_selected_domain(desc.X, indices), desc.y)
                fit_child(step, selected)
        return self._validate_children(branches, input_domain)

    def __sklearn_clone__(self):
        """Preserve output configuration while clearing fitted state."""
        fresh = super().__sklearn_clone__()
        fresh._dp_output = self._dp_output
        return fresh

    def set_output(self, *, transform=None):
        """Select the sklearn transform container, independently of .lazy().

        The default produces a homogeneous NumPy matrix (carried by SklearnFit
        in OpenDP queries). 'polars' produces an eager typed DataFrameDomain.
        """
        if transform is not None:
            if transform not in {"default", "polars"}:
                raise ValueError("Only 'default' and 'polars' output are supported")
            self._dp_output = transform
        return self

    def _branches(self, x):
        width = len(x.columns) if isinstance(x, DataFrameDomain) else x.num_columns
        column_names = x.columns if isinstance(x, DataFrameDomain) else None
        if self.remainder not in {"drop", "passthrough"}:
            raise TypeError("remainder must be 'drop' or 'passthrough'")
        if not isinstance(self.verbose_feature_names_out, bool):
            raise TypeError("verbose_feature_names_out must be a bool")

        configured = self.transformers
        names = [name for name, _, _ in configured]
        if not all(isinstance(name, str) for name in names):
            raise ValueError("Transformer names must be strings")
        self._validate_names(names)

        used = set()
        branches = []
        for name, step, columns in configured:
            if isinstance(step, str) and step not in {"drop", "passthrough"}:
                raise ValueError(f"Unsupported transformer in branch {name!r}")
            indices = _selector(columns, width, names=column_names)
            if not indices and not (
                isinstance(step, str) and step in {"drop", "passthrough"}
            ):
                raise ValueError(f"Branch {name!r} selects no input columns")
            used.update(indices)
            branches.append((name, step, indices))
        if self.remainder == "passthrough":
            rest = tuple(i for i in range(width) if i not in used)
            if rest:
                branches.append(("remainder", "passthrough", rest))
        return branches

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        """Privately fit the branches under one composed privacy allowance."""
        if input_metric != symmetric_distance():
            raise ValueError(
                "ColumnTransformer currently requires symmetric row distance"
            )
        # clone clears fitted state; deepcopy also detaches FrozenEstimator children.
        prototype = snapshot(self)
        desc = input_domain.cast(SklearnFitDescriptor)
        x = _input_features(desc.X)
        branches = prototype._branches(x)

        selections = {}
        for name, step, indices in branches:
            if isinstance(step, str):
                continue
            _rowwise_transformer(step, name)
            selection = _select_transform(input_domain, input_metric, indices)
            if needs_private_fit(step):
                selections[name] = selection
            else:
                fit_child(step, selection.output_domain)
                _output_columns(
                    _rowwise_transformer(step, name).rowwise_output_domain(
                        selection.output_domain.cast(SklearnFitDescriptor).X,
                        method="transform",
                    ),
                    name,
                )

        compositor, budgets = allocate_private_stages(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            selections,
            prototype.privacy_weights,
        )
        measures = {}
        next_budget = iter(budgets)
        for name, step, _ in branches:
            if name in selections:
                select = selections[name]
                measures[name] = checked_child_fit(
                    step,
                    name,
                    select,
                    select.output_space,
                    output_measure,
                    d_in,
                    next(next_budget),
                )

        def execute(qbl):
            # Each invocation owns its fitted children. Only accounted releases
            # and public metadata reach their ingestion methods.
            children = deepcopy(branches)
            for name, child, _ in children:
                if name in measures:
                    fit_child(
                        child, selections[name].output_domain, qbl, measures[name]
                    )
            return prototype._validate_children(children, input_domain)

        return compositor >> _new_pure_function(execute, TO="ExtrinsicObject")

    def _validate_children(self, children, input_domain):
        # Validate before installing state in the caller, including for refits.
        candidate = copy(self)
        _finish_fit(candidate, children, input_domain)
        candidate.rowwise_output_domain(
            input_domain.cast(SklearnFitDescriptor).X, method="transform"
        )
        return children

    def _ingest_release(self, release):
        self.transformers_ = list(release)
        self._dp_fitted_ = True

    def _check_input_features(self, width, names=None):
        if width != self.n_features_in_:
            raise ValueError("Input feature width differs from the fitted width")
        if names is not None and hasattr(self, "feature_names_in_"):
            if tuple(names) != tuple(self.feature_names_in_):
                raise ValueError("Input feature names differ from the fitted names")

    def _active_branches(self):
        return (
            (name, step, indices)
            for name, step, indices in self.transformers_
            if not (isinstance(step, str) and step == "drop")
        )

    def _output_names(self, width):
        names = self.get_feature_names_out()
        if len(names) != width:
            raise ValueError("Feature names must match the output width")
        return names

    def rowwise_output_domain(self, input_domain, *, method):
        """Certify row-aligned output without changing the fitted estimator."""
        if method != "transform":
            raise ValueError(f"ColumnTransformer has no rowwise method {method!r}")
        check_is_fitted(self, "transformers_")
        x = _input_features(input_domain)
        width = len(x.columns) if isinstance(x, DataFrameDomain) else x.num_columns
        self._check_input_features(
            width, x.columns if isinstance(x, DataFrameDomain) else None
        )

        columns = []
        for name, step, indices in self._active_branches():
            selected = _selected_domain(x, indices)
            output = (
                selected
                if isinstance(step, str)
                else _rowwise_transformer(step, name).rowwise_output_domain(
                    selected, method="transform"
                )
            )
            columns.extend(_output_columns(output, name))
        if not columns:
            raise ValueError("ColumnTransformer must emit at least one column")

        if self._dp_output == "polars":
            names = self._output_names(len(columns))
            return dataframe_domain(
                [series_domain(name, dom) for name, (dom, _) in zip(names, columns)]
            )

        if any(getattr(dom, "element_domain", None) is not None for dom, _ in columns):
            raise TypeError("Nullable output requires set_output(transform='polars')")
        dtype = np.result_type(*(dtype for _, dtype in columns))
        if dtype.kind not in "biuf":
            raise TypeError(
                "Default output requires numeric columns; "
                "use set_output(transform='polars') for mixed types"
            )
        return array2_domain(
            T=dtype.type,
            num_columns=len(columns),
            size=None if isinstance(x, DataFrameDomain) else x.size,
            nan=dtype.kind == "f"
            and any(bool(getattr(dom, "nan", False)) for dom, _ in columns),
        )

    def transform(self, X):
        """Apply fitted branches and concatenate their ordinary sklearn outputs."""
        if not hasattr(self, "transformers_"):
            raise NotFittedError("Call fit() before transform()")
        # Do not import optional Polars just to transform a NumPy array.
        is_frame = isinstance(X, getattr(modules.get("polars"), "DataFrame", ()))
        if not is_frame:
            X = np.asarray(X)
        self._check_input_features(
            X.shape[1] if len(X.shape) == 2 else None,
            X.columns if is_frame else None,
        )

        columns = []
        for _, step, indices in self._active_branches():
            batch = (
                X.select([X.columns[i] for i in indices]) if is_frame else X[:, indices]
            )
            raw = (
                batch
                if isinstance(step, str)
                else unwrap_frozen_estimator(step).transform(batch)
            )
            if isinstance(raw, getattr(modules.get("polars"), "DataFrame", ())):
                columns.extend(raw.get_columns())
            else:
                if not isinstance(raw, np.ndarray) or raw.ndim != 2:
                    raise ValueError("Branch transform must return a feature matrix")
                columns.extend(raw.T)

        if self._dp_output == "polars":
            import polars as pl  # type: ignore[import-not-found]

            names = self._output_names(len(columns))
            return pl.DataFrame(
                [
                    col.clone().rename(name)
                    if isinstance(col, pl.Series)
                    else pl.Series(name, col)
                    for name, col in zip(names, columns)
                ]
            )
        return np.column_stack(columns)

    def get_feature_names_out(self, input_features=None):
        """Get output feature names using the children's sklearn interfaces."""
        check_is_fitted(self, "transformers_")
        input_features = _check_feature_names_in(self, input_features)

        named = []
        for name, step, indices in self._active_branches():
            features = input_features[list(indices)]
            if isinstance(step, str) and step == "passthrough":
                names = features
            else:
                transformer = unwrap_frozen_estimator(step)
                if not hasattr(transformer, "get_feature_names_out"):
                    raise AttributeError(
                        f"Transformer {name} (type {type(transformer).__name__}) "
                        "does not provide get_feature_names_out."
                    )
                names = transformer.get_feature_names_out(features)
            if names is not None:
                named.append((name, names))

        if not named:
            return np.array([], dtype=object)
        return self._add_prefix_for_feature_names_out(named)
