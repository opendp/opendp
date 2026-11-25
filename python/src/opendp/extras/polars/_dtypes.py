"""Shared exact Polars dtypes for OpenDP primitive and NumPy scalar types."""


def _polars_dtype(T):
    import polars as pl  # type: ignore[import-not-found]

    names = {
        "bool": pl.Boolean,
        "i8": pl.Int8,
        "i16": pl.Int16,
        "i32": pl.Int32,
        "i64": pl.Int64,
        "u8": pl.UInt8,
        "u16": pl.UInt16,
        "u32": pl.UInt32,
        "u64": pl.UInt64,
        "f32": pl.Float32,
        "f64": pl.Float64,
        "String": pl.String,
    }
    aliases = {
        "int8": "i8",
        "int16": "i16",
        "int32": "i32",
        "int64": "i64",
        "uint8": "u8",
        "uint16": "u16",
        "uint32": "u32",
        "uint64": "u64",
        "float32": "f32",
        "float64": "f64",
    }
    name = str(T)
    try:
        return names[aliases.get(name, name)]
    except KeyError as exc:
        raise TypeError(f"Unsupported Polars dtype: {T}") from exc
