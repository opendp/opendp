"""Stable conversion of a homogeneous Polars DataFrame to a NumPy matrix."""

from opendp._internal import _make_transformation
from opendp.context import register
from opendp.mod import DataFrameDomain

# Polars primitive types that NumPy can represent exactly.
_NUMPY_DTYPES = {
    "bool": "bool",
    "i8": "int8", "i16": "int16", "i32": "int32", "i64": "int64",
    "u8": "uint8", "u16": "uint16", "u32": "uint32", "u64": "uint64",
    "f32": "float32", "f64": "float64",
}


def _numpy_dtype(T):
    """NumPy dtype for a supported OpenDP primitive."""
    import numpy as np

    try:
        return np.dtype(_NUMPY_DTYPES[str(T)])
    except KeyError as exc:
        raise TypeError(f"Unsupported NumPy dtype: {T}") from exc


def _polars_dtype(T):
    """Exact Polars dtype for a supported OpenDP scalar type."""
    import polars as pl

    dtypes = {
        "bool": pl.Boolean,
        "i8": pl.Int8, "i16": pl.Int16, "i32": pl.Int32, "i64": pl.Int64,
        "u8": pl.UInt8, "u16": pl.UInt16, "u32": pl.UInt32, "u64": pl.UInt64,
        "f32": pl.Float32, "f64": pl.Float64, "String": pl.String,
    }
    # Also accept NumPy dtypes from composed column metadata.
    name = str(T)
    if name not in dtypes:
        name = next((t for t, np_name in _NUMPY_DTYPES.items()
                     if np_name == name), name)
    try:
        return dtypes[name]
    except KeyError as exc:
        raise TypeError(f"Unsupported Polars dtype: {T}") from exc


def _row_distance(metric):
    """Turn a global frame contribution bound into a row distance."""
    from opendp.metrics import (
        _get_bound, symmetric_distance, insert_delete_distance,
        change_one_distance, hamming_distance,
    )
    from opendp.mod import FrameDistance

    if isinstance(metric, FrameDistance):
        if metric.inner_metric != symmetric_distance():
            raise ValueError("Only FrameDistance<SymmetricDistance> is supported")

        def stability(bounds):
            bound = _get_bound(bounds, []).per_group
            if bound is None:
                raise ValueError("A global row-contribution bound is required")
            return bound

        return symmetric_distance(), stability

    if metric not in (
        symmetric_distance(), insert_delete_distance(),
        change_one_distance(), hamming_distance(),
    ):
        raise ValueError("Conversion requires a row-adjacency metric")
    return metric, lambda d: d


def make_dataframe_to_numpy(input_domain, input_metric):
    """Convert an eager, non-null, homogeneous numeric DataFrame to NPArray2Domain.

    Column types are checked against the public domain; there is no implicit
    promotion, materialization of lazy plans, or sklearn-specific wrapping.
    Numeric bounds are discarded, and non-NaN guarantees are preserved.
    """
    import numpy as np
    from opendp.extras.numpy import array2_domain

    if not isinstance(input_domain, DataFrameDomain):
        raise TypeError("Expected DataFrameDomain")
    columns = [
        input_domain.get_series_domain(name) for name in input_domain.columns
    ]
    if not columns:
        raise ValueError("At least one feature column is required")
    if any(col.nullable for col in columns):
        raise ValueError("NumPy conversion requires non-nullable columns")
    types = [str(col.element_domain.carrier_type) for col in columns]
    if len(set(types)) != 1 or types[0] not in _NUMPY_DTYPES:
        raise TypeError("Columns must have one uniform numeric dtype")

    T = types[0]
    output_domain = array2_domain(
        T=T,
        num_columns=len(columns),
        nan=(any(col.element_domain.nan for col in columns)
             if T in {"f32", "f64"} else False),
    )
    dtype = _numpy_dtype(T)

    def convert(frame):
        array = frame.to_numpy()
        if array.dtype != dtype:
            raise TypeError("Frame dtype differs from its public domain")
        return array

    output_metric, stability = _row_distance(input_metric)

    return _make_transformation(
        input_domain, input_metric, output_domain, output_metric,
        convert, stability,
    )


register(make_dataframe_to_numpy, name="to_numpy")
