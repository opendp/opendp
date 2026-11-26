"""Stable packaging of aligned sklearn inputs, with optional NumPy conversion."""

from __future__ import annotations

from opendp._internal import _make_transformation
from opendp.extras.numpy._make_dataframe_to_numpy import _NUMPY_DTYPES
from opendp.extras.numpy._row_distance import (
    _frame_distance_to_symmetric_distance,
    _row_distance,
)
from opendp.extras.sklearn._domains import (
    SklearnFit,
    SklearnFitDescriptor,
    sklearn_fit_domain,
)
from opendp.mod import DataFrameDomain, FrameDistance, LazyFrameDomain, SeriesDomain


def _vector_target_domain(series):
    """Domain for a non-null target converted from Polars to Python/NumPy."""
    from opendp.domains import vector_domain

    if series.nullable:
        raise ValueError("Target must be non-nullable for NumPy conversion")
    atom = series.element_domain
    T = str(atom.carrier_type)
    if T not in {*_NUMPY_DTYPES, "String"}:
        raise TypeError("Target must be numeric, boolean, or string")
    return vector_domain(atom)


def _series_to_vector(series):
    import polars as pl  # type: ignore[import-not-found]

    # Polars commonly returns a read-only NumPy view; OpenDP vector membership
    # uses ctypes, which requires a writable buffer.
    if series.dtype == pl.String:
        return series.to_list()
    return series.to_numpy(writable=True)


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
        if isinstance(desc.X, DataFrameDomain)
        else None
    )
    y_conversion = isinstance(desc.y, SeriesDomain)
    y_domain = _vector_target_domain(desc.y) if y_conversion else desc.y
    output_domain = sklearn_fit_domain(
        x_conversion.output_domain if x_conversion else desc.X,
        y_domain,
    )

    def convert(data):
        X = x_conversion(data.X) if x_conversion else data.X
        target = _series_to_vector(data.y) if y_conversion else data.y
        return SklearnFit(X, target)

    return _make_transformation(
        input_domain,
        input_metric,
        output_domain,
        output_metric,
        convert,
        stability,
    )


def _make_dataframe_fit(input_domain, input_metric, y):
    """Select features and an optional target without changing their carriers."""
    from opendp.domains import dataframe_domain

    if y is not None and (not isinstance(y, str) or y not in input_domain.columns):
        raise ValueError("y must name an existing DataFrame column")

    features = [name for name in input_domain.columns if name != y]
    if not features:
        raise ValueError("At least one feature column is required")
    x_domain = dataframe_domain(
        [input_domain.get_series_domain(name) for name in features]
    )
    y_domain = input_domain.get_series_domain(y) if y is not None else None

    if isinstance(input_metric, FrameDistance):
        output_metric, stability = _frame_distance_to_symmetric_distance(input_metric)
    else:
        output_metric, stability = _row_distance(input_metric)

    def convert(frame):
        return SklearnFit(
            frame.select(features),
            frame.get_column(y) if y is not None else None,
        )

    return _make_transformation(
        input_domain,
        input_metric,
        sklearn_fit_domain(x_domain, y_domain),
        output_metric,
        convert,
        stability,
    )


def make_sklearn_fit(input_domain, input_metric, *, y=None):
    """Package a DataFrame or NumPy matrix without converting feature carriers.

    For DataFrames, ``y`` optionally names a target column. For NumPy inputs,
    provide targets separately in ``SklearnFit(X, y)`` before constructing a Context.
    """
    if isinstance(input_domain, LazyFrameDomain):
        raise TypeError("Call .collect() before .sklearn() on a LazyFrame query")
    if isinstance(input_domain, DataFrameDomain):
        return _make_dataframe_fit(input_domain, input_metric, y)

    if y is not None:
        raise TypeError("NumPy targets must be provided explicitly as SklearnFit(X, y)")

    from opendp.extras.numpy import NPArray2Domain

    input_domain.cast(NPArray2Domain)
    output_metric, stability = _row_distance(input_metric)
    return _make_transformation(
        input_domain,
        input_metric,
        sklearn_fit_domain(input_domain),
        output_metric,
        lambda X: SklearnFit(X),
        stability,
    )
