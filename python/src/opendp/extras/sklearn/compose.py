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

import numpy as np
from sklearn.base import clone
from sklearn.compose import ColumnTransformer as SklearnColumnTransformer
from sklearn.exceptions import NotFittedError
from sklearn.utils.validation import _check_feature_names_in, check_is_fitted

from opendp._internal import _make_transformation, _new_pure_function
from opendp.context import domain_of
from opendp.domains import atom_domain, dataframe_domain, series_domain, option_domain
from opendp.extras.numpy import NPArray2Domain, array2_domain
from opendp.extras.sklearn._domains import (
    SklearnFit,
    SklearnFitDescriptor,
    sklearn_fit_domain,
)
from opendp.extras.numpy._make_dataframe_to_numpy import _numpy_dtype
from opendp.mod import DataFrameDomain, Domain
from opendp.extras.sklearn._estimator import (
    MeasurementMixin,
    TransformationMixin,
    _set_fit_metadata,
)
from opendp.extras.sklearn._composition import (
    needs_private_fit,
    unwrap_frozen_estimator,
    allocate_private_stages,
    checked_child_fit,
)
from opendp.metrics import symmetric_distance


def _selector(columns, width, names=None):
    """Resolve public names or positions without inspecting private rows."""
    if names is not None and isinstance(columns, str):
        columns = [columns]
    if (
        names is not None
        and isinstance(columns, (list, tuple, np.ndarray))
        and all(isinstance(c, str) for c in columns)
    ):
        unknown = [c for c in columns if c not in names]
        if unknown:
            raise ValueError(f"Unknown feature columns: {unknown}")
        indices = tuple(names.index(c) for c in columns)
    elif isinstance(columns, slice):
        indices = tuple(range(width)[columns])
    elif isinstance(columns, (int, np.integer)) and not isinstance(
        columns, (bool, np.bool_)
    ):
        indices = (int(columns),)
    elif isinstance(columns, (list, tuple, np.ndarray)):
        items = list(columns)
        if items and all(isinstance(c, (bool, np.bool_)) for c in items):
            if len(items) != width:
                raise ValueError("Boolean selector must match the feature width")
            indices = tuple(i for i, flag in enumerate(items) if flag)
        elif all(
            isinstance(c, (int, np.integer)) and not isinstance(c, (bool, np.bool_))
            for c in items
        ):
            indices = tuple(int(c) for c in items)
        else:
            raise TypeError(
                "Only public integer positions, slices, or boolean masks are supported"
            )
    else:
        raise TypeError("Only public positional column selectors are supported")

    normalized = tuple(i + width if i < 0 else i for i in indices)
    if any(i < 0 or i >= width for i in normalized):
        raise ValueError("Column index is out of bounds")
    if len(set(normalized)) != len(normalized):
        raise ValueError("Repeated column indices are not supported")
    return normalized


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
                    column,
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
        atom = atom_domain(
            T=array.T,
            **({"nan": array.nan} if dtype.kind == "f" else {}),
        )
        columns = [(f"x{i}", atom, dtype) for i in range(array.num_columns)]

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


class ColumnTransformer(
    MeasurementMixin, TransformationMixin, SklearnColumnTransformer
):
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

    def fit(
        self,
        X,
        y=None,
        *,
        privacy_unit=None,
        privacy_loss=None,
        domain=None,
    ):
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

        domain = domain or domain_of(SklearnFit(X, y), infer=True)
        features = _input_features(domain.cast(SklearnFitDescriptor).X)
        fresh = clone(self)
        fresh.rowwise_output_domain(features, method="transform")
        self.transformers_ = [
            (name, step if isinstance(step, str) else clone(step), indices)
            for name, step, indices in fresh._branches(features)
        ]
        self._dp_rowwise_domain_ = features
        _set_fit_metadata(self, domain)
        self._set_child_metadata(domain.cast(SklearnFitDescriptor))
        return self

    def __sklearn_clone__(self):
        """Preserve output configuration while clearing fitted state."""
        params = self.get_params(deep=False)
        fresh = type(self)(**{k: clone(v, safe=False) for k, v in params.items()})
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
        x = _input_features(input_domain.cast(SklearnFitDescriptor).X)
        branches = self._branches(x)

        # Reject missing rowwise methods before consuming privacy. Validate
        # private implementations after fitting, when parameters are available.
        for name, step, indices in branches:
            if isinstance(step, str):
                continue
            transformer = _rowwise_transformer(step, name)
            if not needs_private_fit(step):
                _output_columns(
                    transformer.rowwise_output_domain(
                        _selected_domain(x, indices), method="transform"
                    ),
                    name,
                )

        private = [
            (name, step, indices)
            for name, step, indices in branches
            if needs_private_fit(step)
        ]
        compositor, budgets = allocate_private_stages(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            [name for name, _, _ in private],
            self.privacy_weights,
        )
        measures = []
        for (name, step, indices), budget in zip(private, budgets):
            select = _select_transform(input_domain, input_metric, indices)
            measures.append(
                checked_child_fit(
                    clone(step),
                    name,
                    select,
                    select.output_space,
                    output_measure,
                    d_in,
                    budget,
                )
            )

        # The input domain and width are public. Branch releases are separately
        # accounted, and only fitted parameters appear in the released state.
        def execute(qbl):
            return input_domain, tuple(qbl(measurement) for measurement in measures)

        return compositor >> _new_pure_function(execute)

    def _set_child_metadata(self, descriptor):
        """Set each fitted child's public input metadata."""
        for _, child, indices in self.transformers_:
            if not isinstance(child, str):
                _set_fit_metadata(
                    child,
                    sklearn_fit_domain(
                        _selected_domain(descriptor.X, indices), descriptor.y
                    ),
                )

    def _ingest_release(self, release):
        input_domain, values = release
        desc = input_domain.cast(SklearnFitDescriptor)
        features = _input_features(desc.X)
        releases = iter(values)
        children = []
        for name, step, indices in self._branches(features):
            child = step if isinstance(step, str) else clone(step)
            if not isinstance(child, str) and needs_private_fit(child):
                child._ingest_release(next(releases))
            children.append((name, child, indices))
        self.transformers_ = children
        _set_fit_metadata(self, input_domain)
        self._set_child_metadata(desc)

        # Validate released children against the rowwise contract after accounting.
        self.rowwise_output_domain(desc.X, method="transform")

    def _compile(self, feature_domain):
        x = _input_features(feature_domain)
        fitted = hasattr(self, "transformers_")
        width = len(x.columns) if isinstance(x, DataFrameDomain) else x.num_columns
        if fitted and width != self.n_features_in_:
            raise ValueError("Input feature width differs from the fitted width")

        compiled = []
        output_columns: list[tuple[str, Domain, np.dtype]] = []
        for name, step, indices in self.transformers_ if fitted else self._branches(x):
            if isinstance(step, str) and step == "drop":
                continue
            selected_domain = _selected_domain(x, indices)
            feature_names = (
                tuple(x.columns[i] for i in indices)
                if isinstance(x, DataFrameDomain)
                else None
            )
            if isinstance(step, str) and step == "passthrough":
                branch_domain = selected_domain
            else:
                branch_domain = _rowwise_transformer(step, name).rowwise_output_domain(
                    selected_domain, method="transform"
                )

            columns = _output_columns(branch_domain, name)
            if (
                isinstance(step, str)
                and step == "passthrough"
                and feature_names is None
            ):
                columns = [
                    (f"x{i}", dom, dtype)
                    for i, (_, dom, dtype) in zip(indices, columns)
                ]
            output_columns.extend(
                (
                    f"{name}__{column}" if self.verbose_feature_names_out else column,
                    dom,
                    dtype,
                )
                for column, dom, dtype in columns
            )
            compiled.append((indices, feature_names, step, branch_domain))

        if fitted and self._dp_output == "polars":
            # Extract from the child's raw schema, but publish sklearn names.
            names = self.get_feature_names_out()
            if len(names) != len(output_columns):
                raise ValueError("Feature names must match the output width")
            output_columns = [
                (name, dom, dtype)
                for name, (_, dom, dtype) in zip(names, output_columns)
            ]

        names = [name for name, _, _ in output_columns]
        if len(names) != len(set(names)):
            raise ValueError("Output column names must be unique")
        if not names:
            raise ValueError("ColumnTransformer must emit at least one column")
        return compiled, output_columns

    def rowwise_output_domain(self, input_domain, *, method):
        """Describe the row-aligned output of a possibly nested composite."""
        if method != "transform":
            raise ValueError(f"ColumnTransformer has no rowwise method {method!r}")
        features = _input_features(input_domain)
        _, columns = self._compile(features)
        if self._dp_output == "polars":
            output = dataframe_domain(
                [series_domain(name, dom) for name, dom, _ in columns]
            )
            self._dp_rowwise_domain_ = input_domain
            return output
        # Default sklearn output is one homogeneous numeric NumPy matrix.
        if any(
            getattr(dom, "element_domain", None) is not None for _, dom, _ in columns
        ):
            raise TypeError("Nullable output requires set_output(transform='polars')")
        dtype = np.result_type(*(dtype for _, _, dtype in columns))
        if dtype.kind not in "biuf":
            raise TypeError(
                "Default output requires numeric columns; "
                "use set_output(transform='polars') for mixed types"
            )
        nan = dtype.kind == "f" and any(
            bool(getattr(dom, "nan", False)) for _, dom, _ in columns
        )
        output = array2_domain(
            T=dtype.type,
            num_columns=len(columns),
            size=(None if isinstance(features, DataFrameDomain) else features.size),
            nan=nan,
        )
        self._dp_rowwise_domain_ = input_domain
        return output

    def transform(self, X):
        """Apply the rowwise branches through their ordinary sklearn methods."""
        # A rowwise invocation uses the public domain saved on the snapshot,
        # never a domain inferred from private values.
        features = getattr(self, "_dp_rowwise_domain_", None)
        if features is None:
            raise NotFittedError(
                "Call fit() before transform(), or declare a rowwise input domain"
            )

        compiled, columns = self._compile(features)
        results = []
        for indices, feature_names, step, branch_domain in compiled:
            batch = (
                X.select(feature_names) if feature_names is not None else X[:, indices]
            )
            raw = (
                batch
                if step == "passthrough"
                else unwrap_frozen_estimator(step).transform(batch)
            )
            if isinstance(branch_domain, DataFrameDomain):
                for column in branch_domain.columns:
                    series = raw.get_column(column)
                    declared = branch_domain.get_series_domain(column)
                    results.append(
                        series.to_list() if declared.nullable else series.to_numpy()
                    )
            else:
                values = np.asarray(raw)
                if values.ndim == 1:
                    values = values[:, None]
                results.extend(values.T)

        if len(results) != len(columns):
            raise ValueError("Branch output width differs from the declared schema")
        if self._dp_output == "polars":
            import polars as pl  # type: ignore[import-not-found]
            from opendp.extras.polars._dtypes import _polars_dtype

            return pl.DataFrame(
                {
                    name: pl.Series(
                        name,
                        value,
                        dtype=pl.String if dtype.kind == "O" else _polars_dtype(dtype),
                    )
                    for (name, _, dtype), value in zip(columns, results)
                }
            )
        return np.column_stack(results)

    def get_feature_names_out(self, input_features=None):
        """Get output feature names using the children's sklearn interfaces."""
        check_is_fitted(self, "transformers_")
        input_features = _check_feature_names_in(self, input_features)

        named = []
        for name, step, indices in self.transformers_:
            if isinstance(step, str) and step == "drop":
                continue
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
