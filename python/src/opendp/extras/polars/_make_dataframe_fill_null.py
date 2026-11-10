"""Public, stable null replacement in eager Polars DataFrames."""

from opendp.domains import dataframe_domain, series_domain
from opendp.mod import AtomDomain, DataFrameDomain
from opendp.metrics import (
    change_one_distance,
    hamming_distance,
    insert_delete_distance,
    symmetric_distance,
)


def _dtype_for_atom(atom):
    import polars as pl

    types = {
        "bool": pl.Boolean,
        "i8": pl.Int8, "i16": pl.Int16, "i32": pl.Int32, "i64": pl.Int64,
        "u8": pl.UInt8, "u16": pl.UInt16, "u32": pl.UInt32, "u64": pl.UInt64,
        "f32": pl.Float32, "f64": pl.Float64, "String": pl.String,
    }
    try:
        return types[str(atom.carrier_type)]
    except KeyError as exc:
        raise TypeError("fill_null supports primitive numeric, boolean, and string columns") from exc


def make_dataframe_fill_null(input_domain, input_metric, value, *, columns=None):
    """Replace nulls in an eager DataFrame with a public, in-domain value.

    Selected columns become non-nullable. This is a rowwise stable operation;
    unlike a schema inference, it establishes the declared non-null guarantee.
    NaNs are distinct from nulls and are not replaced. Columns not selected
    retain their original domains. For heterogeneous types, select columns
    explicitly and call fill_null separately with appropriately typed values.
    """
    import polars as pl
    from opendp._internal import _make_transformation

    if not isinstance(input_domain, DataFrameDomain):
        raise TypeError("fill_null requires an eager DataFrameDomain")
    if input_metric not in (
        symmetric_distance(), insert_delete_distance(),
        change_one_distance(), hamming_distance(),
    ):
        raise ValueError("fill_null requires a row-adjacency metric")

    names = tuple(input_domain.columns)
    if columns is None:
        selected = names
    elif isinstance(columns, str):
        selected = (columns,)
    else:
        selected = tuple(columns)
    if (
        any(not isinstance(name, str) or name not in names for name in selected)
        or len(selected) != len(set(selected))
    ):
        raise ValueError("columns must be distinct names in the input domain")

    replacements = {}
    output_series = []
    for name in names:
        column_domain = input_domain.get_series_domain(name)
        if name not in selected or not column_domain.nullable:
            output_series.append(column_domain)
            continue
        atom = column_domain.element_domain
        if not isinstance(atom, AtomDomain):
            raise TypeError("fill_null requires atomic columns")
        dtype = _dtype_for_atom(atom)
        # Validate against public dtype and bounds before declaring non-nullability.
        try:
            literal = pl.Series(name, [value], dtype=dtype, strict=True).item()
        except (TypeError, ValueError, OverflowError) as exc:
            raise ValueError(f"Invalid fill value for column {name!r}") from exc
        if not atom.member(literal):
            raise ValueError(f"Fill value falls outside the domain of {name!r}")
        replacements[name] = (literal, dtype)
        output_series.append(series_domain(name, atom))

    output_domain = dataframe_domain(output_series)

    def fill(frame):
        if not replacements:
            return frame.clone()
        return frame.with_columns([
            pl.col(name).fill_null(pl.lit(literal, dtype=dtype))
            for name, (literal, dtype) in replacements.items()
        ])

    return _make_transformation(
        input_domain, input_metric, output_domain, input_metric,
        fill, lambda d: d,
    )

