"""Stable packaging of aligned sklearn inputs, with optional NumPy conversion."""

from __future__ import annotations

from numbers import Integral

from opendp._internal import _make_transformation
from opendp.extras.numpy._make_dataframe_to_numpy import _NUMPY_DTYPES, _row_distance
from opendp.extras.sklearn._domains import (
    SklearnFit,
    SklearnFitDescriptor,
    sklearn_fit_domain,
)
from opendp.mod import DataFrameDomain, LazyFrameDomain, SeriesDomain


def _vector_target_domain(series):
    """Domain for a non-null target converted from Polars to Python/NumPy."""
    from opendp.domains import atom_domain, vector_domain

    if series.nullable:
        raise ValueError("Target must be non-nullable for NumPy conversion")
    atom = series.element_domain
    T = str(atom.carrier_type)
    if T not in {*_NUMPY_DTYPES, "String"}:
        raise TypeError("Target must be numeric, boolean, or string")
    return vector_domain(atom_domain(
        T=T, **({"nan": atom.nan} if T in {"f32", "f64"} else {}),
    ))


def _series_to_vector(series):
    import polars as pl

    return series.to_list() if series.dtype == pl.String else series.to_numpy()


def make_sklearn_fit_numpy(input_domain, input_metric):
    """Adapt only the representations inside an existing aligned SklearnFit.

    This transformation does not select features or targets. Numerical estimators
    opt into it; heterogeneous meta-estimators can retain Polars feature frames.
    """
    from opendp.extras.numpy._make_dataframe_to_numpy import make_dataframe_to_numpy

    desc = input_domain.cast(SklearnFitDescriptor)
    output_metric, stability = _row_distance(input_metric)
    x_conversion = (
        make_dataframe_to_numpy(desc.X, output_metric)
        if isinstance(desc.X, DataFrameDomain) else None
    )
    y_conversion = isinstance(desc.y, SeriesDomain)
    y_domain = _vector_target_domain(desc.y) if y_conversion else desc.y
    output_domain = sklearn_fit_domain(
        x_conversion.output_domain if x_conversion else desc.X, y_domain,
    )

    def convert(data):
        X = x_conversion(data.X) if x_conversion else data.X
        target = _series_to_vector(data.y) if y_conversion else data.y
        return SklearnFit(X, target)

    return _make_transformation(
        input_domain, input_metric, output_domain, output_metric,
        convert, stability,
    )


def _make_dataframe_fit(input_domain, input_metric, y):
    """Select features and an optional target without changing their carriers."""
    from opendp.domains import dataframe_domain

    if not isinstance(input_domain, DataFrameDomain):
        raise TypeError("Expected DataFrameDomain")
    if y is not None and (not isinstance(y, str) or y not in input_domain.columns):
        raise ValueError("y must name an existing DataFrame column")

    features = [name for name in input_domain.columns if name != y]
    if not features:
        raise ValueError("At least one feature column is required")
    x_domain = dataframe_domain([
        input_domain.get_series_domain(name) for name in features
    ])
    y_domain = input_domain.get_series_domain(y) if y is not None else None
    if y_domain is not None and y_domain.nullable:
        raise ValueError("Target must be non-nullable")

    output_metric, stability = _row_distance(input_metric)

    def convert(frame):
        return SklearnFit(
            frame.select(features),
            frame.get_column(y) if y is not None else None,
        )

    return _make_transformation(
        input_domain, input_metric, sklearn_fit_domain(x_domain, y_domain),
        output_metric, convert, stability,
    )


def make_dataframe_to_sklearn_fit(
    input_domain, input_metric, *, y=None, as_numpy=True,
):
    """Package an eager DataFrame as aligned sklearn features and target."""
    packed = _make_dataframe_fit(input_domain, input_metric, y)
    if not as_numpy:
        return packed
    return packed >> make_sklearn_fit_numpy(*packed.output_space)


def _make_numpy_fit(input_domain, input_metric, y):
    """Package NumPy features and optionally split out a target column."""
    from opendp.domains import atom_domain, vector_domain
    from opendp.extras.numpy import NPArray2Domain, array2_domain

    desc = input_domain.cast(NPArray2Domain)
    output_metric, stability = _row_distance(input_metric)
    if y is not None and (isinstance(y, bool) or not isinstance(y, Integral)):
        raise TypeError("y must be an integer column index for NumPy arrays")
    if y is None:
        return _make_transformation(
            input_domain, input_metric, sklearn_fit_domain(input_domain),
            output_metric, lambda X: SklearnFit(X), stability,
        )

    if desc.num_columns is None:
        raise ValueError("NumPy feature width must be public when selecting y")
    ncols = desc.num_columns
    index = int(y)
    if not -ncols <= index < ncols:
        raise ValueError("y column index is out of bounds")
    index %= ncols
    if ncols < 2:
        raise ValueError("At least one feature column must remain")

    selected = [i for i in range(ncols) if i != index]
    features_domain = array2_domain(
        T=desc.T, num_columns=len(selected), size=desc.size,
        nan=desc.nan, norm=desc.norm, p=desc.p,
        origin=None if desc.origin is None else desc.origin[selected],
        cardinalities=(
            None if desc.cardinalities is None else desc.cardinalities[selected]
        ),
    )
    target_domain = vector_domain(
        atom_domain(
            T=desc.T,
            **({"nan": desc.nan} if str(desc.T) in {"f32", "f64"} else {}),
        ),
        size=desc.size,
    )
    return _make_transformation(
        input_domain, input_metric, sklearn_fit_domain(features_domain, target_domain),
        output_metric, lambda X: SklearnFit(X[:, selected], X[:, index].copy()),
        stability,
    )


def make_sklearn_fit(input_domain, input_metric, *, y=None, as_numpy=True):
    """Construct aligned sklearn data, optionally adapting X and y to NumPy.

    Query composition keeps the original representation (as_numpy=False);
    numerical estimators request the explicit second conversion step.
    """
    if isinstance(input_domain, LazyFrameDomain):
        raise TypeError("Call .collect() before .sklearn() on a LazyFrame query")
    if isinstance(input_domain, DataFrameDomain):
        return make_dataframe_to_sklearn_fit(
            input_domain, input_metric, y=y, as_numpy=as_numpy,
        )

    try:
        input_domain.cast(SklearnFitDescriptor)
    except ValueError:
        packed = _make_numpy_fit(input_domain, input_metric, y)
    else:
        if y is not None:
            raise ValueError("y is already part of SklearnFit")
        output_metric, stability = _row_distance(input_metric)
        packed = _make_transformation(
            input_domain, input_metric, input_domain, output_metric,
            lambda data: data, stability,
        )

    if not as_numpy:
        return packed
    return packed >> make_sklearn_fit_numpy(*packed.output_space)
