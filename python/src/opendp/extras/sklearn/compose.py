"""Privacy-aware sklearn ColumnTransformer.

Private fitting is an OpenDP adaptive composition. Stable inference uses a
trusted rowwise contract: each branch emits exactly one value per input row,
with no side effects or data-dependent failures, and preserves row order.

Do not treat arbitrary sklearn transformers or OpenDP Transformations as
rowwise. Symmetric-distance stability alone does not imply alignment.
"""

from __future__ import annotations

import numpy as np
from sklearn.base import clone
from sklearn.compose import ColumnTransformer as SklearnColumnTransformer

from opendp._internal import _make_transformation, _new_pure_function
from opendp.context import domain_of
from opendp.domains import atom_domain, dataframe_domain, series_domain, option_domain
from opendp.extras.numpy import NPArray2Domain, array2_domain
from opendp.extras.sklearn._domains import (
    SklearnFit, SklearnFitDescriptor, sklearn_fit_domain,
)
from opendp.extras.sklearn._frozen import _RowwiseSpec, rowwise_spec, supports_rowwise
from opendp.extras.numpy._make_dataframe_to_numpy import _numpy_dtype, _polars_dtype
from opendp.mod import DataFrameDomain
from opendp.extras.sklearn._estimator import (
    DPEstimator, DPTransformer, needs_private_fit,
)
from opendp.extras.sklearn._composition import (
    allocate_private_stages, checked_child_fit,
)
from opendp.metrics import symmetric_distance


def _selector(columns, width, names=None):
    """Resolve public names or positions without inspecting private rows."""
    if names is not None and isinstance(columns, str):
        columns = [columns]
    if names is not None and isinstance(columns, (list, tuple, np.ndarray)) and all(
        isinstance(c, str) for c in columns
    ):
        unknown = [c for c in columns if c not in names]
        if unknown:
            raise ValueError(f"Unknown feature columns: {unknown}")
        return tuple(names.index(c) for c in columns)
    if isinstance(columns, slice):
        indices = tuple(range(width)[columns])
    elif isinstance(columns, (int, np.integer)) and not isinstance(columns, (bool, np.bool_)):
        indices = (int(columns),)
    elif isinstance(columns, (list, tuple, np.ndarray)):
        items = list(columns)
        if items and all(isinstance(c, (bool, np.bool_)) for c in items):
            if len(items) != width:
                raise ValueError("Boolean selector must match the feature width")
            indices = tuple(i for i, flag in enumerate(items) if flag)
        elif all(isinstance(c, (int, np.integer)) and
                 not isinstance(c, (bool, np.bool_)) for c in items):
            indices = tuple(int(c) for c in items)
        else:
            raise TypeError("Only public integer positions, slices, or boolean masks are supported")
    else:
        raise TypeError("Only public positional column selectors are supported")

    normalized = tuple(i + width if i < 0 else i for i in indices)
    if any(i < 0 or i >= width for i in normalized):
        raise ValueError("Column index is out of bounds")
    if len(set(normalized)) != len(normalized):
        raise ValueError("Repeated column indices are not supported")
    return normalized


def _input_descriptor(domain, metric):
    if metric != symmetric_distance():
        raise ValueError("ColumnTransformer currently requires symmetric row distance")
    desc = domain.cast(SklearnFitDescriptor)
    if isinstance(desc.X, DataFrameDomain):
        return desc, desc.X
    x = desc.X.cast(NPArray2Domain)
    if x.num_columns is None:
        raise ValueError("ColumnTransformer needs a public number of input features")
    return desc, x


def _selected_domain(desc, indices):
    """Selecting feature coordinates cannot increase a row norm."""
    if isinstance(desc.X, DataFrameDomain):
        names = desc.X.columns
        selected = dataframe_domain(
            [desc.X.get_series_domain(names[i]) for i in indices]
        )
        return sklearn_fit_domain(selected, desc.y)
    x = desc.X.cast(NPArray2Domain)
    origin = None if x.origin is None else x.origin[list(indices)]
    cardinalities = (
        None if x.cardinalities is None else x.cardinalities[list(indices)]
    )
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
    return sklearn_fit_domain(output_x, desc.y)


def _select_transform(domain, metric, indices):
    desc = domain.cast(SklearnFitDescriptor)
    output_domain = _selected_domain(desc, indices)
    positions = tuple(indices)

    names = (
        tuple(desc.X.columns[i] for i in positions)
        if isinstance(desc.X, DataFrameDomain) else None
    )

    def select(data):
        X = data.X.select(names) if names is not None else data.X[:, positions]
        return SklearnFit(X, data.y)

    return _make_transformation(
        domain, metric, output_domain, metric, select, lambda d: d,
    )


def _output_columns(spec, name):
    """Describe only the output carriers ColumnTransformer can concatenate."""
    if not isinstance(spec, _RowwiseSpec) or not callable(spec.function):
        raise TypeError(f"Branch {name!r} must return a _RowwiseSpec")

    domain = spec.output_domain
    if isinstance(domain, DataFrameDomain):
        columns = []
        for column in domain.columns:
            series = domain.get_series_domain(column)
            atom = series.element_domain
            dtype = (
                np.dtype("object") if str(atom.carrier_type) == "String"
                else _numpy_dtype(atom.carrier_type)
            )
            columns.append((
                column, option_domain(atom) if series.nullable else atom, dtype,
            ))
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
            T=array.T, **({"nan": array.nan} if dtype.kind == "f" else {}),
        )
        columns = [(f"x{i}", atom, dtype) for i in range(array.num_columns)]

    if not columns:
        raise ValueError(f"Branch {name!r} must emit at least one column")
    return columns


class ColumnTransformer(SklearnColumnTransformer, DPTransformer):
    """Compile private parallel fitting and certified rowwise column stacking.

    Uses the familiar sklearn list of (name, transformer, columns). Private
    branches must be OpenDP DPEstimator instances. Transform branches must
    provide a trusted make_rowwise method, or be passthrough/drop.

    Supports named and positional selectors on eager DataFrames or NumPy
    matrices. Never invokes ordinary sklearn fitting on private data.

    Rowwise branches must be OpenDP estimators with a rowwise implementation.
    """

    def __init__(
        self, transformers, *, remainder="drop", sparse_threshold=0.3,
        n_jobs=None, transformer_weights=None, verbose=False,
        verbose_feature_names_out=True, privacy_weights=None,
    ):
        super().__init__(
            transformers, remainder=remainder, sparse_threshold=sparse_threshold,
            n_jobs=n_jobs, transformer_weights=transformer_weights,
            verbose=verbose, verbose_feature_names_out=verbose_feature_names_out,
        )
        self.privacy_weights = privacy_weights
        self._dp_output = "default"

    def _has_private_fit(self):
        return any(needs_private_fit(step) for _, step, _ in self.transformers)

    def _supports_rowwise(self):
        return all(
            (isinstance(step, str) and step in {"drop", "passthrough"})
            or supports_rowwise(step)
            for _, step, _ in self.transformers
        )

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

    def _branches(self, domain, metric, *, fitted=False):
        _, x = _input_descriptor(domain, metric)
        width = len(x.columns) if isinstance(x, DataFrameDomain) else x.num_columns
        column_names = x.columns if isinstance(x, DataFrameDomain) else None
        if self.remainder not in {"drop", "passthrough"}:
            raise TypeError("remainder must be 'drop' or 'passthrough'")
        if self.n_jobs not in (None, 1):
            raise ValueError("Parallel branch fitting is not supported")
        if self.transformer_weights is not None:
            raise ValueError("transformer_weights is not supported; use privacy_weights for fitting")
        if not isinstance(self.verbose_feature_names_out, bool):
            raise TypeError("verbose_feature_names_out must be a bool")

        configured = self.transformers_ if fitted else self.transformers
        names = [name for name, _, _ in configured]
        if len(names) != len(set(names)) or not all(isinstance(name, str) for name in names):
            raise ValueError("Transformer names must be unique strings")
        used = set()
        branches = []
        for name, step, columns in configured:
            if isinstance(step, str) and step not in {"drop", "passthrough"}:
                raise ValueError(f"Unsupported transformer in branch {name!r}")
            indices = _selector(columns, width, names=column_names)
            if not indices and not (isinstance(step, str) and step in {"drop", "passthrough"}):
                raise ValueError(f"Branch {name!r} selects no input columns")
            used.update(indices)
            branches.append((name, step, indices))
        if self.remainder == "passthrough":
            if "remainder" in names:
                raise ValueError("The name 'remainder' is reserved")
            rest = tuple(i for i in range(width) if i not in used)
            if rest:
                branches.append(("remainder", "passthrough", rest))
        return branches

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        """Privately fit the branches under one composed privacy allowance."""
        branches = self._branches(input_domain, input_metric)
        desc = input_domain.cast(SklearnFitDescriptor)

        # Check public structure before constructing any private measurement.
        # A private branch cannot certify its output until it has been fitted,
        # but it must at least declare its rowwise implementation in advance.
        for name, step, columns in branches:
            if isinstance(step, str) and step in {"drop", "passthrough"}:
                continue
            if not supports_rowwise(step):
                raise TypeError(f"Branch {name!r} is not certified rowwise")
            if not needs_private_fit(step):
                rowwise_spec(step, _selected_domain(desc, columns))

        private_names = [
            name for name, step, _ in branches if needs_private_fit(step)
        ]
        compositor, budgets = allocate_private_stages(
            input_domain, input_metric, output_measure, d_in, d_out,
            private_names, self.privacy_weights,
        )
        measures = []
        budget_iter = iter(budgets)
        for name, step, columns in branches:
            if not needs_private_fit(step):
                continue
            select = _select_transform(input_domain, input_metric, columns)
            measures.append(checked_child_fit(
                clone(step), name, select, select.output_space,
                output_measure, d_in, next(budget_iter),
            ))

        # The input domain and width are public. Branch releases are separately
        # accounted, and only fitted parameters appear in the released state.
        def execute(qbl):
            return input_domain, tuple(qbl(measurement) for measurement in measures)

        return compositor >> _new_pure_function(execute)

    def _ingest_release(self, release):
        input_domain, values = release
        children = []
        it = iter(values)
        for name, step, columns in self.transformers:
            if needs_private_fit(step):
                child = clone(step)
                child._ingest_release(next(it))
            elif supports_rowwise(step):
                child = clone(step)
            else:
                child = step
            children.append((name, child, columns))
        self.transformers_ = children
        self._dp_input_domain_ = input_domain
        x_domain = input_domain.cast(SklearnFitDescriptor).X
        self.n_features_in_ = (
            len(x_domain.columns) if isinstance(x_domain, DataFrameDomain)
            else x_domain.cast(NPArray2Domain).num_columns
        )

    def _compile(self, input_domain, input_metric):
        desc, x = _input_descriptor(input_domain, input_metric)
        fitted = hasattr(self, "transformers_")
        width = len(x.columns) if isinstance(x, DataFrameDomain) else x.num_columns
        if fitted and width != self.n_features_in_:
            raise ValueError("Input feature width differs from the fitted width")
        # An unfitted composition works with certified rowwise or passthrough branches.
        branches = self._branches(input_domain, input_metric, fitted=fitted)
        compiled = []
        output_columns = []
        for name, step, indices in branches:
            if isinstance(step, str) and step == "drop":
                continue
            selected_domain = _selected_domain(desc, indices)
            as_frame = isinstance(desc.X, DataFrameDomain)
            feature_names = (
                tuple(desc.X.columns[i] for i in indices)
                if as_frame else None
            )
            if isinstance(step, str) and step == "passthrough":
                spec = _RowwiseSpec(
                    lambda batch: batch,
                    selected_domain.cast(SklearnFitDescriptor).X,
                )
            else:
                spec = rowwise_spec(step, selected_domain)

            columns = _output_columns(spec, name)
            if isinstance(step, str) and step == "passthrough" and not as_frame:
                columns = [
                    (f"x{i}", dom, dtype)
                    for i, (_, dom, dtype) in zip(indices, columns)
                ]
            named = [
                (f"{name}__{column}" if self.verbose_feature_names_out else column,
                 dom, dtype)
                for column, dom, dtype in columns
            ]
            compiled.append((tuple(indices), feature_names, spec))
            output_columns.extend(named)

        names = [name for name, _, _ in output_columns]
        if len(names) != len(set(names)):
            raise ValueError("Output column names must be unique")
        if not names:
            raise ValueError("ColumnTransformer must emit at least one column")
        return desc, compiled, output_columns

    def make_rowwise(self, input_domain):
        """Certify aligned rowwise column stacking, including nested composites."""
        desc, compiled, columns = self._compile(input_domain, symmetric_distance())
        to_polars = self._dp_output == "polars"
        if to_polars:
            import polars as pl
            domains = [
                series_domain(name, dom) for name, dom, _ in columns
            ]
            output_domain = dataframe_domain(domains)
        else:
            # Match sklearn's default homogeneous numeric output. Keep the
            # intermediate dataset as a NumPy-backed SklearnFit so subsequent
            # numerical Pipeline stages need no conversion back from Polars.
            if any(getattr(dom, "element_domain", None) is not None
                   for _, dom, _ in columns):
                raise TypeError(
                    "Nullable output requires set_output(transform='polars')"
                )
            dtype = np.result_type(*(dtype for _, _, dtype in columns))
            if dtype.kind not in "biuf":
                raise TypeError(
                    "Default output requires numeric columns; "
                    "use set_output(transform='polars') for mixed types"
                )
            nan = dtype.kind == "f" and any(
                bool(getattr(dom, "nan", False))
                for _, dom, _ in columns
            )
            output_domain = array2_domain(
                T=dtype.type,
                num_columns=len(columns),
                size=(None if isinstance(desc.X, DataFrameDomain)
                      else desc.X.cast(NPArray2Domain).size),
                nan=nan,
            )

        def transform(X):
            results = []
            for indices, feature_names, spec in compiled:
                batch = (
                    X.select(feature_names)
                    if feature_names is not None else X[:, indices]
                )
                raw = spec.function(batch)
                if isinstance(spec.output_domain, DataFrameDomain):
                    for column in spec.output_domain.columns:
                        series = raw.get_column(column)
                        declared = spec.output_domain.get_series_domain(column)
                        results.append(
                            series.to_list() if declared.nullable else series.to_numpy()
                        )
                else:
                    results.extend(
                        raw[:, i] for i in range(spec.output_domain.num_columns)
                    )
            if to_polars:
                # Preserve the declared column types, including integer
                # cluster labels alongside floating-point distances.
                items = {
                    name: pl.Series(
                        name, value,
                        dtype=(pl.String if dtype.kind == "O" else _polars_dtype(dtype)),
                    )
                    for (name, _, dtype), value in zip(columns, results)
                }
                return pl.DataFrame(items)
            return np.column_stack(results)

        return _RowwiseSpec(transform, output_domain)

    def make_transform(self, input_domain, input_metric):
        """Wrap the certified rowwise operation as an aligned stable transform."""
        if input_metric != symmetric_distance():
            raise ValueError("ColumnTransformer currently requires symmetric row distance")
        desc = input_domain.cast(SklearnFitDescriptor)
        spec = self.make_rowwise(input_domain)
        return _make_transformation(
            input_domain, input_metric,
            sklearn_fit_domain(spec.output_domain, desc.y), input_metric,
            lambda data: SklearnFit(spec.function(data.X), data.y),
            lambda d: d,
        )

    def fit(self, X, y=None, *, privacy_unit, privacy_loss, domain=None):
        """Standalone private fit, requiring explicit privacy parameters."""
        return DPEstimator.fit(
            self, X, y, privacy_unit=privacy_unit,
            privacy_loss=privacy_loss, domain=domain,
        )

    def fit_transform(self, X, y=None, **kwargs):
        """Never implicitly expose transformed private training records."""
        raise NotImplementedError(
            "Private fitting and row-level transformation are separate operations. "
            "Call fit with privacy parameters, then construct a private query "
            "with make_transform."
        )

    def transform(self, X):
        """Ordinary sklearn inference on public input, using fitted branches."""
        from opendp.metrics import symmetric_distance

        import polars as pl

        domain = None
        if isinstance(X, pl.DataFrame):
            if hasattr(self, "_dp_input_domain_"):
                features = self._dp_input_domain_.cast(SklearnFitDescriptor).X
                if isinstance(features, DataFrameDomain):
                    domain = sklearn_fit_domain(features)
            else:
                # Only public inference may inspect values to infer nullability.
                inferred = domain_of(X, infer=True)
                domain = sklearn_fit_domain(dataframe_domain([
                    (
                        inferred.get_series_domain(name)
                        if X[name].null_count()
                        else series_domain(
                            name, inferred.get_series_domain(name).element_domain
                        )
                    )
                    for name in X.columns
                ]))
        if domain is None:
            X = np.asarray(X)
            domain = domain_of(SklearnFit(X), infer=True)
        result = self.make_transform(domain, symmetric_distance())(SklearnFit(X))
        return result.X

    def get_feature_names_out(self, input_features=None):
        """Names generated from certified branch output metadata."""
        if not hasattr(self, "n_features_in_"):
            raise ValueError("ColumnTransformer must be fitted first")
        if input_features is not None:
            raise NotImplementedError("Custom input feature names are not yet supported")
        _, _, columns = self._compile(
            self._dp_input_domain_, symmetric_distance()
        )
        return np.asarray([name for name, _, _ in columns], dtype=object)
