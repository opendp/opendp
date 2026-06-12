from typing import Any, Sequence

from opendp._lib import (
    ATOM_EQUIVALENCE_CLASSES,
    AnyFunction,
    ExtrinsicObject,
    FfiSlice,
    FfiSlicePtr,
    ctypes,
    import_optional_dependency,
)
from opendp.mod import (
    ApproximateDivergence,
    ChangeOneIdDistance,
    ExtrinsicDistance,
    ExtrinsicDivergence,
    ExtrinsicDomain,
    LazyFrameDomain,
    FrameDistance,
    SeriesDomain,
    SymmetricIdDistance,
    Transformation,
    Measurement,
    Function,
    AtomDomain,
    OptionDomain,
    VectorDomain,
)
from opendp.typing import RuntimeType

ATOM_MAP = {
    "f32": ctypes.c_float,
    "f64": ctypes.c_double,
    "u8": ctypes.c_uint8,
    "u16": ctypes.c_uint16,
    "u32": ctypes.c_uint32,
    "u64": ctypes.c_uint64,
    "i8": ctypes.c_int8,
    "i16": ctypes.c_int16,
    "i32": ctypes.c_int32,
    "i64": ctypes.c_int64,
    "usize": ctypes.c_size_t,
    "bool": ctypes.c_bool,
    "AnyMeasurementPtr": Measurement,
    "AnyTransformationPtr": Transformation,
}

_NUMPY_COMPATIBLE_ATOM_TYPES = frozenset(ATOM_MAP) - {
    "AnyMeasurementPtr",
    "AnyTransformationPtr",
}

_ERROR_URL_298 = "https://github.com/opendp/opendp/discussions/298"


def c_int_limits(type_name):
    c_int_type = ATOM_MAP[type_name]
    signed = c_int_type(-1).value < c_int_type(0).value
    bit_size = ctypes.sizeof(c_int_type) * 8
    signed_limit = 2 ** (bit_size - 1)
    return (-signed_limit, signed_limit - 1) if signed else (0, 2 * signed_limit - 1)


INT_SIZES = {
    ty: c_int_limits(ty)
    for ty in (
        "u8",
        "u16",
        "u32",
        "u64",
        "i8",
        "i16",
        "i32",
        "i64",
        "usize",
    )
}


def _numpy_dtype_for_rust_type(type_name: str) -> Any:
    np = import_optional_dependency("numpy")
    if type_name not in _NUMPY_COMPATIBLE_ATOM_TYPES:
        raise ValueError(f"unrecognized numpy dtype: {type_name}")
    return np.dtype(ATOM_MAP[type_name])


def _check_and_cast_scalar(expected, value):
    """
    1. Converts integer value to float if expected value is float
    2. Checks that value is roughly a member of the same data type as expected
    3. Checks that integers are representable at the given data type
    """
    # relax checks in the case of an int
    if (
        isinstance(value, int)
        and expected in ["f32", "f64"]
        and not isinstance(value, bool)
    ):
        return float(value)

    inferred = str(RuntimeType.infer(value))

    if expected not in ATOM_EQUIVALENCE_CLASSES.get(inferred, [inferred]):
        raise TypeError(
            f"inferred type is {inferred}, expected {expected}. See {_ERROR_URL_298}"
        )

    if expected in INT_SIZES:
        check_c_int_cast(value, expected)

    return value


def check_c_int_cast(v, type_name):
    lower, upper = INT_SIZES[type_name]
    if not (lower <= v <= upper):
        raise ValueError(f"{v} is not representable by {type_name}")


def _extrinsic_to_slice(val) -> FfiSlicePtr:
    return _wrap_in_slice(ctypes.pointer(ExtrinsicObject(ctypes.py_object(val))), 1)


def _slice_to_extrinsic(raw: FfiSlicePtr):
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ExtrinsicObject)).contents.ptr


def _string_to_slice(val: str) -> FfiSlicePtr:
    np = import_optional_dependency("numpy", raise_error=False)
    if np is not None and isinstance(val, np.ndarray):
        val = val.item()
    return _wrap_in_slice(ctypes.pointer(ctypes.c_char_p(val.encode())), 1)


def _slice_to_string(raw: FfiSlicePtr) -> str:
    value = ctypes.cast(
        raw.contents.ptr, ctypes.POINTER(ctypes.c_char_p)
    ).contents.value
    assert value is not None
    return value.decode()


def _bitvector_to_slice(val: Sequence[Any]) -> FfiSlicePtr:
    np = import_optional_dependency("numpy", raise_error=False)
    if np is not None and isinstance(val, np.ndarray):
        val = val.tobytes()

    if not isinstance(val, (bytes, bytearray)):
        raise TypeError(
            "Expected type is BitVector but input data is not bytes or bytearray."
        )  # pragma: no cover

    array = (ctypes.c_uint8 * len(val)).from_buffer_copy(val)  # type: ignore[operator]
    return _wrap_in_slice(array, len(val) * 8)


def _slice_to_bitvector(raw: FfiSlicePtr) -> bytes:
    # raw.contents.len is the number of valid bits.
    # Division by -8 is ceiling rather than floor: the number of bytes in the buffer
    n_bytes = -(raw.contents.len // -8)
    buffer = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_uint8))[0:n_bytes]  # type: ignore
    return bytes(buffer)


def _numpy_to_slice(val, type_name: RuntimeType) -> FfiSlicePtr:
    np = import_optional_dependency("numpy")
    if type_name.origin != "NDArray" or len(type_name.args) != 1:
        raise ValueError(
            f"type_name must be NDArray<T> with one type argument, found {type_name}"
        )  # pragma: no cover

    inner_type_name = type_name.args[0]
    if not isinstance(inner_type_name, str):
        raise ValueError(
            f"inner type must be atomic, found {inner_type_name}"
        )  # pragma: no cover

    np_dtype = _numpy_dtype_for_rust_type(inner_type_name)
    if not isinstance(val, np.ndarray):
        raise TypeError(f"Expected type is {type_name}.")

    if val.ndim != 1:
        raise TypeError("Only 1d arrays are currently supported. Flatten first.")
    if val.dtype != np.dtype(np_dtype):
        raise TypeError(f"Expected dtype {np.dtype(np_dtype)}, got {val.dtype}.")

    contiguous = np.ascontiguousarray(val)
    array = np.ctypeslib.as_ctypes(contiguous)
    ffi_slice = _wrap_in_slice(array, len(contiguous))
    ffi_slice.depends_on(contiguous, array)
    return ffi_slice


def _slice_to_numpy(raw: FfiSlicePtr, type_name: RuntimeType):
    np = import_optional_dependency("numpy")
    if type_name.origin != "NDArray" or len(type_name.args) != 1:
        raise ValueError(
            f"type_name must be NDArray<T> with one type argument, found {type_name}"
        )  # pragma: no cover

    inner_type_name = type_name.args[0]
    if not isinstance(inner_type_name, str):
        raise ValueError(
            f"inner type must be atomic, found {inner_type_name}"
        )  # pragma: no cover

    _numpy_dtype_for_rust_type(
        inner_type_name
    )  # validate numpy compatibility before casting

    array_ptr: Any = ctypes.cast(
        raw.contents.ptr, ctypes.POINTER(ATOM_MAP[inner_type_name])
    )
    return np.ctypeslib.as_array(array_ptr, shape=(raw.contents.len,)).copy()


def _slice_to_function(raw: FfiSlicePtr) -> Function:
    # for ε(α)-RDP curves
    function = ctypes.cast(raw.contents.ptr, ctypes.POINTER(AnyFunction)).contents
    # put the contents behind a new, python pointer
    return ctypes.cast(ctypes.pointer(function), Function)


def _function_to_slice(raw: Function, type_name: RuntimeType) -> FfiSlicePtr:
    if not isinstance(raw, Function):
        from opendp.core import new_function

        raw = new_function(raw, TO=type_name.args[1])
    return _wrap_in_slice(raw, 1)


def _lazyframe_to_slice(val) -> FfiSlicePtr:
    pl = import_optional_dependency("polars")
    if not isinstance(val, pl.LazyFrame):
        raise ValueError("expected Polars LazyFrame")

    state = val.__getstate__()
    raw = _wrap_in_slice(state, len(state))
    raw.depends_on(state)
    return raw


def _slice_to_lazyframe(raw: FfiSlicePtr):
    pl = import_optional_dependency("polars")
    lf = pl.LazyFrame()
    slice_array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_uint8))
    lf.__setstate__(bytes(slice_array[0 : raw.contents.len]))
    return lf


def _expr_to_slice(val) -> FfiSlicePtr:
    pl = import_optional_dependency("polars")
    if isinstance(val, str):
        val = pl.col(val)

    if not isinstance(val, pl.Expr):
        raise ValueError("expected Polars Expr")

    state = val.__getstate__()
    raw = _wrap_in_slice(state, len(state))
    raw.depends_on(state)
    return raw


def _slice_to_expr(raw: FfiSlicePtr):
    pl = import_optional_dependency("polars")
    expr = pl.all()
    slice_array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_uint8))
    expr.__setstate__(bytes(slice_array[0 : raw.contents.len]))
    return expr


def _slice_to_exprplan(raw: FfiSlicePtr):
    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    ptr_data: list[ctypes.c_void_p] = void_array_ptr[0 : raw.contents.len]

    plan = _slice_to_lazyframe(ctypes.cast(ptr_data[0], FfiSlicePtr))
    expr = _slice_to_expr(ctypes.cast(ptr_data[1], FfiSlicePtr))
    fill = (
        _slice_to_expr(ctypes.cast(ptr_data[2], FfiSlicePtr))
        if raw.contents.len == 3
        else None
    )

    from collections import namedtuple

    ExprPlan = namedtuple("ExprPlan", ["plan", "expr", "fill"])
    return ExprPlan(plan, expr, fill)


def _check_polars_by(by):
    if isinstance(by, str):
        raise ValueError(f'by ({by}) must be a sequence type; Did you mean ["{by}"]?')

    if not isinstance(by, Sequence):
        raise ValueError(f"by ({by}) must be a sequence type")


def _dataframe_to_slice(val) -> FfiSlicePtr:
    pl = import_optional_dependency("polars")
    if not isinstance(val, pl.DataFrame):
        raise ValueError("expected Polars DataFrame")  # pragma: no cover

    slices = list(_series_to_slice(s) for s in val.get_columns())
    raw = _wrap_in_slice(ctypes.pointer((FfiSlicePtr * val.width)(*slices)), val.width)
    # extend the lifetime of each series' slice to that of the frame slice
    raw.depends_on(slices)
    return raw


def _slice_to_dataframe(raw: FfiSlicePtr):
    pl = import_optional_dependency("polars")
    slice_array = ctypes.cast(raw.contents.ptr, FfiSlicePtr)
    series = [
        _slice_to_series(FfiSlicePtr(ffislice))
        for ffislice in slice_array[0 : raw.contents.len]
    ]
    return pl.DataFrame(series)


def _series_to_slice(val) -> FfiSlicePtr:
    from opendp._data import new_arrow_array, arrow_array_free

    pl = import_optional_dependency("polars")
    if not isinstance(val, pl.Series):
        raise ValueError("expected Polars Series")

    raw = new_arrow_array(val.name)
    slice_array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    array_ptr, schema_ptr = slice_array[0:2]

    # make the conversion through PyArrow's private API
    # this changes the pointer's memory and is thus unsafe. In particular, `_export_to_c` can go out of bounds
    # NOTE: consider changing to PyCapsule when available. https://github.com/pola-rs/polars/issues/12530
    val.to_arrow()._export_to_c(array_ptr, schema_ptr)

    # when freeing the slice, also free up the memory of what's left behind the slice
    class ArrowArrayFFIBuffer(object):
        def __init__(self, ptr) -> None:
            self.ptr = ptr

        def __del__(self) -> None:
            arrow_array_free(self.ptr)

    raw.depends_on(ArrowArrayFFIBuffer(raw.contents.ptr))
    return raw


def _slice_to_series(raw: FfiSlicePtr):
    pl = import_optional_dependency("polars")
    pyarrow = import_optional_dependency("pyarrow")
    slice_array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    array_ptr, schema_ptr, name_ptr = slice_array[0:3]

    arrow_array = pyarrow.Array._import_from_c(array_ptr, schema_ptr)
    series = pl.from_arrow(arrow_array)

    name_bytes = ctypes.cast(name_ptr, ctypes.c_char_p).value
    if name_bytes is not None:
        series = series.rename(name_bytes.decode())
    return series


def _wrap_in_slice(ptr, len_: int) -> FfiSlicePtr:
    return FfiSlicePtr(FfiSlice(ctypes.cast(ptr, ctypes.c_void_p), len_))


class _ConvertMaps:
    TO_SLICE_FROM_STR = {
        "BitVector": _bitvector_to_slice,
        "ExtrinsicObject": _extrinsic_to_slice,
        "AnyMeasurement": lambda value: _wrap_in_slice(value, 1),
        "String": _string_to_slice,
        "Series": _series_to_slice,
        "LazyFrame": _lazyframe_to_slice,
        "DslPlan": _lazyframe_to_slice,
        "DataFrame": _dataframe_to_slice,
        "Expr": _expr_to_slice,
    }

    TO_SLICE_FROM_ORIGIN = {
        "NDArray": _numpy_to_slice,
        "Function": _function_to_slice,
    }

    FROM_SLICE_STR = {
        "BitVector": _slice_to_bitvector,
        "String": _slice_to_string,
        "LazyFrame": _slice_to_lazyframe,
        "DataFrame": _slice_to_dataframe,
        "Series": _slice_to_series,
        "Expr": _slice_to_expr,
        "ExprPlan": _slice_to_exprplan,
        "ExtrinsicObject": _slice_to_extrinsic,
    }

    FROM_SLICE_RT_TYPE = {
        "NDArray": _slice_to_numpy,
        "Function": lambda raw, _: _slice_to_function(raw),
    }

    DOMAIN_CLASS_FROM_RT_TYPE = {
        OptionDomain.__name__: OptionDomain,
        AtomDomain.__name__: AtomDomain,
        VectorDomain.__name__: VectorDomain,
        SeriesDomain.__name__: SeriesDomain,
        LazyFrameDomain.__name__: LazyFrameDomain,
        ExtrinsicDomain.__name__: ExtrinsicDomain,
    }

    DISTANCE_CLASS_FROM_RT_TYPE = {
        FrameDistance.__name__: FrameDistance,
        SymmetricIdDistance.__name__: SymmetricIdDistance,
        ChangeOneIdDistance.__name__: ChangeOneIdDistance,
        ExtrinsicDistance.__name__: ExtrinsicDistance,
    }

    MEASURE_CLASS_FROM_RT_TYPE = {
        "Approximate": ApproximateDivergence,
        ExtrinsicDivergence.__name__: ExtrinsicDivergence,
    }