from typing import Any, Sequence, Union, cast, MutableMapping

from opendp._lib import *
from opendp.mod import (
    ApproximateDivergence,
    ChangeOneIdDistance,
    Domain,
    ExtrinsicDistance,
    ExtrinsicDivergence,
    ExtrinsicDomain,
    LazyFrameDomain,
    FrameDistance,
    SeriesDomain,
    SymmetricIdDistance,
    Transformation,
    Measurement,
    PrivacyProfile,
    Function,
    AtomDomain,
    OptionDomain,
    VectorDomain,
)
from opendp.typing import RuntimeType

ATOM_MAP = {
    'f32': ctypes.c_float,
    'f64': ctypes.c_double,
    'u8': ctypes.c_uint8,
    'u16': ctypes.c_uint16,
    'u32': ctypes.c_uint32,
    'u64': ctypes.c_uint64,
    'i8': ctypes.c_int8,
    'i16': ctypes.c_int16,
    'i32': ctypes.c_int32,
    'i64': ctypes.c_int64,
    'usize': ctypes.c_size_t,
    'bool': ctypes.c_bool,
    'AnyMeasurementPtr': Measurement,
    'AnyTransformationPtr': Transformation,
}
_ERROR_URL_298 = "https://github.com/opendp/opendp/discussions/298"

def _numpy_dtype_for_rust_type(type_name: str) -> Any:
    np = import_optional_dependency('numpy')
    if type_name not in _NUMPY_COMPATIBLE_ATOM_TYPES:
        raise ValueError(f"unrecognized numpy dtype: {type_name}")
    return np.dtype(ATOM_MAP[type_name])


def c_int_limits(type_name):
    c_int_type = ATOM_MAP[type_name]
    signed = c_int_type(-1).value < c_int_type(0).value
    bit_size = ctypes.sizeof(c_int_type) * 8
    signed_limit = 2 ** (bit_size - 1)
    return (-signed_limit, signed_limit - 1) if signed else (0, 2 * signed_limit - 1)

def _check_and_cast_scalar(expected, value):
    '''
    1. Converts integer value to float if expected value is float
    2. Checks that value is roughly a member of the same data type as expected
    3. Checks that integers are representable at the given data type
    '''
    # relax checks in the case of an int
    if isinstance(value, int) and expected in ["f32", "f64"] and not isinstance(value, bool):
        return float(value)

    inferred = str(RuntimeType.infer(value))

    if expected not in ATOM_EQUIVALENCE_CLASSES.get(inferred, [inferred]):
        raise TypeError(f"inferred type is {inferred}, expected {expected}. See {_ERROR_URL_298}")

    if expected in INT_SIZES:
        check_c_int_cast(value, expected)

    return value


def check_c_int_cast(v, type_name):
    lower, upper = INT_SIZES[type_name]
    if not (lower <= v <= upper):
        raise ValueError(f"{v} is not representable by {type_name}")


def _scalar_to_slice(val, type_name: str) -> FfiSlicePtr:
    np = import_optional_dependency('numpy', raise_error=False)
    if np is not None and isinstance(val, np.ndarray):
        val = val.item()
    val = _check_and_cast_scalar(type_name, val)
    # ctypes.byref has edge-cases that cause use-after-free errors. ctypes.pointer fixes these edge-cases
    return _wrap_in_slice(ctypes.pointer(ATOM_MAP[type_name](val)), 1)


def _slice_to_scalar(raw: FfiSlicePtr, type_name: str):
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[type_name])).contents.value # type: ignore[attr-defined]

def _extrinsic_to_slice(val) -> FfiSlicePtr:
    return _wrap_in_slice(ctypes.pointer(ExtrinsicObject(ctypes.py_object(val))), 1)

def _slice_to_extrinsic(raw: FfiSlicePtr):
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ExtrinsicObject)).contents.ptr

def _string_to_slice(val: str) -> FfiSlicePtr:
    np = import_optional_dependency('numpy', raise_error=False)
    if np is not None and isinstance(val, np.ndarray):
        val = val.item()
    return _wrap_in_slice(ctypes.pointer(ctypes.c_char_p(val.encode())), 1)


def _slice_to_string(raw: FfiSlicePtr) -> str:
    value = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_char_p)).contents.value
    assert value is not None
    return value.decode()


def _bitvector_to_slice(val: Sequence[Any]) -> FfiSlicePtr:
    np = import_optional_dependency('numpy', raise_error=False)
    if np is not None and isinstance(val, np.ndarray):
        val = val.tobytes()

    if not isinstance(val, (bytes, bytearray)):
        raise TypeError("Expected type is BitVector but input data is not bytes or bytearray.")  # pragma: no cover

    array = (ctypes.c_uint8 * len(val)).from_buffer_copy(val) # type: ignore[operator]
    return _wrap_in_slice(array, len(val) * 8)


def _slice_to_bitvector(raw: FfiSlicePtr) -> bytes:
    # raw.contents.len is the number of valid bits.
    # Division by -8 is ceiling rather than floor: the number of bytes in the buffer
    n_bytes = -(raw.contents.len // -8)
    buffer = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_uint8))[0:n_bytes] # type: ignore
    return bytes(buffer)


def _vector_to_slice(val: Sequence[Any], type_name: RuntimeType) -> FfiSlicePtr:
    if type_name.origin != 'Vec' or len(type_name.args) != 1:
        raise ValueError("type_name must be Vec<_>")  # pragma: no cover

    inner_type_name = type_name.args[0]

    np = import_optional_dependency("numpy", raise_error=False)
    if (
        np is not None
        and isinstance(val, np.ndarray)
        and isinstance(inner_type_name, str)
        and inner_type_name in _NUMPY_COMPATIBLE_ATOM_TYPES
    ):
        type_name = RuntimeType("NDArray", [inner_type_name])
        return _numpy_to_slice(val, type_name)

    if not isinstance(val, list):
        try:
            val = list(val)
        except TypeError:
            raise TypeError(f"Expected type is {type_name} but input data is not a list.")

    if isinstance(inner_type_name, RuntimeType) or inner_type_name in {"Expr", "Bound", "BitVector"}:
        c_repr = [py_to_c(v, c_type=AnyObjectPtr, type_name=inner_type_name) for v in val]
        array = (AnyObjectPtr * len(val))(*c_repr) # type: ignore[operator]
        ffislice = _wrap_in_slice(array, len(val))
        ffislice.depends_on(*c_repr)
        return ffislice

    if inner_type_name == "ExtrinsicObject":
        c_repr = [ExtrinsicObject(ctypes.py_object(v)) for v in val]
        array = (ExtrinsicObject * len(val))(*c_repr)
        ffi_slice = _wrap_in_slice(array, len(val))
        ffi_slice.depends_on(c_repr)
        return ffi_slice

    if inner_type_name == "SeriesDomain":
        # define the ctype of an array of domains
        domain_array_type = (Domain * len(val)) # type: ignore[operator]
        # create an instance of a ctype array of domains
        array = domain_array_type(*val) # type: ignore[operator]
        return _wrap_in_slice(array, len(val))

    # remaining inner types should be atomic
    val = [_check_and_cast_scalar(inner_type_name, v) for v in val]

    if inner_type_name == "String":
        def str_to_slice(val):
            return ctypes.c_char_p(val.encode())
        array = (ctypes.c_char_p * len(val))(*map(str_to_slice, val))
        return _wrap_in_slice(array, len(val))

    if inner_type_name not in ATOM_MAP:
        raise TypeError(f"Members must be one of {tuple(ATOM_MAP.keys())}. Found {inner_type_name}.")  # pragma: no cover

    array = (ATOM_MAP[inner_type_name] * len(val))(*val)  # type: ignore[operator]
    return _wrap_in_slice(array, len(val))


def _slice_to_vector(raw: FfiSlicePtr, type_name: RuntimeType) -> Sequence[Any]:
    if type_name.origin != 'Vec' or len(type_name.args) != 1:
        raise ValueError("type_name must be Vec<_>")  # pragma: no cover

    inner_type_name = type_name.args[0]

    if inner_type_name in {'AnyObject', 'Expr', 'Bound'}:
        from opendp._data import ffislice_of_anyobjectptrs
        raw = ffislice_of_anyobjectptrs(raw)
        array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(AnyObjectPtr))[0:raw.contents.len]
        res = list(map(c_to_py, array))
        # when the top-level AnyObject is freed, it recursively frees all anyobjects inside of it
        # adjust the type of constituent AnyObjects so that __delete__ is not called when they are dropped
        for elem in array:
            elem.__class__ = ctypes.POINTER(AnyObject)
        return res

    if inner_type_name == 'ExtrinsicObject':
        array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ExtrinsicObject))[0:raw.contents.len]
        return list(map(lambda v: v.ptr, array))

    if inner_type_name == 'String':
        array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_char_p))[0:raw.contents.len]
        return list(map(lambda v: v.decode(), array))

    if not isinstance(inner_type_name, str):
        raise ValueError(f"inner type must be atomic, found {inner_type_name}")  # pragma: no cover

    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[inner_type_name]))[0:raw.contents.len]


def _numpy_to_slice(val, type_name: RuntimeType) -> FfiSlicePtr:
    np = import_optional_dependency("numpy")
    if type_name.origin != 'NDArray' or len(type_name.args) != 1:
        raise ValueError(f"type_name must be NDArray<T> with one type argument, found {type_name}")  # pragma: no cover

    inner_type_name = type_name.args[0]
    if not isinstance(inner_type_name, str):
        raise ValueError(f"inner type must be atomic, found {inner_type_name}")  # pragma: no cover

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
    if type_name.origin != 'NDArray' or len(type_name.args) != 1:
        raise ValueError(f"type_name must be NDArray<T> with one type argument, found {type_name}")  # pragma: no cover

    inner_type_name = type_name.args[0]
    if not isinstance(inner_type_name, str):
        raise ValueError(f"inner type must be atomic, found {inner_type_name}")  # pragma: no cover

    _numpy_dtype_for_rust_type(inner_type_name)  # validate numpy compatibility before casting

    array_ptr: Any = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[inner_type_name]))
    return np.ctypeslib.as_array(array_ptr, shape=(raw.contents.len,)).copy()


def _tuple_to_slice(val: tuple[Any, ...], type_name: Union[RuntimeType, str]) -> FfiSlicePtr:
    type_name = cast(RuntimeType, type_name)
    inner_type_names = type_name.args
    if not isinstance(val, tuple):
        raise TypeError("Cannot coerce a non-tuple type to a tuple")  # pragma: no cover

    if len(inner_type_names) != len(val):
        raise TypeError("type_name members must have same length as tuple")  # pragma: no cover

    if inner_type_names == ['f64', 'ExtrinsicObject']:
        score_ptr = ctypes.pointer(ctypes.c_double(val[0]))
        ext_obj = ctypes.pointer(ExtrinsicObject(ctypes.py_object(val[1])))

        cand_ptr = py_to_c(ext_obj, c_type=AnyObjectPtr, type_name="ExtrinsicObject")
        array = (ctypes.c_void_p * 2)(
            ctypes.cast(score_ptr, ctypes.c_void_p), 
            ctypes.cast(cand_ptr, ctypes.c_void_p), 
        )
        return _wrap_in_slice(ctypes.pointer(array), 2)

    for t in inner_type_names:
        if t not in ATOM_MAP:
            raise TypeError(f"Tuple members must be one of {set(ATOM_MAP.keys())}. Got {t}")  # pragma: no cover

    # check that actual type can be represented by the inner_type_name
    val = tuple(
        _check_and_cast_scalar(inner_type_name, val[i])
        for i, inner_type_name in zip(range(len(val)), inner_type_names)
    )

    # ctypes.byref has edge-cases that cause use-after-free errors. ctypes.pointer fixes these edge-cases
    ptr_data = (
        ctypes.cast(ctypes.pointer(ATOM_MAP[name](v)), ctypes.c_void_p) # type: ignore[index]
        for v, name in zip(val, inner_type_names))

    array = (ctypes.c_void_p * len(val))(*ptr_data)
    return _wrap_in_slice(ctypes.pointer(array), len(val))


def _slice_to_tuple(raw: FfiSlicePtr, type_name: RuntimeType) -> tuple[Any, ...]:
    inner_type_names = type_name.args
    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    ptr_data: list[ctypes.c_void_p] = void_array_ptr[0:raw.contents.len]

    if inner_type_names == ['PrivacyProfile', 'f64']:
        curve = ctypes.cast(ptr_data[0], AnyObjectPtr)
        delta = ctypes.cast(ptr_data[1], ctypes.POINTER(ctypes.c_double))
        return PrivacyProfile(curve), delta.contents.value

    if inner_type_names == ['f64', 'AnyObject']:
        score = ctypes.cast(ptr_data[0], ctypes.POINTER(ctypes.c_double))
        candidate_obj = ctypes.cast(ptr_data[1], AnyObjectPtr)
        candidate = c_to_py(c_to_py(candidate_obj))
        candidate_obj.__class__ = ctypes.POINTER(AnyObject) # type: ignore[assignment]
        return score.contents.value, candidate

    # tuple of instances of Python types
    return tuple(ctypes.cast(void_p, ctypes.POINTER(ATOM_MAP[name])).contents.value # type: ignore[index,attr-defined]
                 for void_p, name in zip(ptr_data, inner_type_names))


def _slice_to_option(raw: FfiSlicePtr, type_name: RuntimeType) -> Optional[Any]:
    if raw.contents.len == 0:
        return None
    return _slice_to_py(raw, type_name.args[0])


def _hashmap_to_slice(val: MutableMapping, type_name: RuntimeType) -> FfiSlicePtr:
    key_type, val_type = type_name.args
    if not isinstance(val, MutableMapping):
        raise TypeError(f"Expected type is {type_name} but input data is not a dict.")  # pragma: no cover

    val = {
        _check_and_cast_scalar(key_type, k):
            _check_and_cast_scalar(val_type, v) if val_type != "ExtrinsicObject" else v
        for k, v in val.items()
    }

    keys: AnyObjectPtr = py_to_c(list(val.keys()), type_name=f"Vec<{key_type}>", c_type=AnyObjectPtr)
    vals: AnyObjectPtr = py_to_c(list(val.values()), type_name=f"Vec<{val_type}>", c_type=AnyObjectPtr)
    ffislice = _wrap_in_slice(ctypes.pointer((AnyObjectPtr * 2)(keys, vals)), 2) # type: ignore[operator]

    # The __del__ destructor on `keys` and `vals` is called and memory freed when their refcounts go to zero.
    # ffislice needs keys and vals to have a lifetime at least as long as itself,
    # so we can't allow their refcount to go to zero until ffislice is freed.
    ffislice.depends_on(keys, vals)
    return ffislice


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


def _slice_to_hashmap(raw: FfiSlicePtr) -> MutableMapping:
    slice_array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(AnyObjectPtr))
    keys: AnyObjectPtr = slice_array[0]
    vals: AnyObjectPtr = slice_array[1]
    result = dict(zip(c_to_py(keys), c_to_py(vals)))

    # AnyObjectPtr.__del__ would free the memory behind keys and vals when this stack frame is popped.
    # But that memory has a lifetime at least as long as raw, so it cannot be freed yet.
    # Adjust the class to avoid calling AnyObjectPtr.__del__, which would free the backing memory.
    keys.__class__ = ctypes.POINTER(AnyObject) # type: ignore[assignment]
    vals.__class__ = ctypes.POINTER(AnyObject) # type: ignore[assignment]
    return result


def _lazyframe_to_slice(val) -> FfiSlicePtr:
    pl = import_optional_dependency('polars')
    if not isinstance(val, pl.LazyFrame):
        raise ValueError("expected Polars LazyFrame")

    state = val.__getstate__()
    raw = _wrap_in_slice(state, len(state))
    raw.depends_on(state)
    return raw


def _slice_to_lazyframe(raw: FfiSlicePtr):
    pl = import_optional_dependency('polars')
    lf = pl.LazyFrame()
    slice_array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_uint8))
    lf.__setstate__(bytes(slice_array[0:raw.contents.len]))
    return lf


def _expr_to_slice(val) -> FfiSlicePtr:
    pl = import_optional_dependency('polars')
    if isinstance(val, str):
        val = pl.col(val)

    if not isinstance(val, pl.Expr):
        raise ValueError("expected Polars Expr")

    state = val.__getstate__()
    raw = _wrap_in_slice(state, len(state))
    raw.depends_on(state)
    return raw


def _slice_to_expr(raw: FfiSlicePtr):
    pl = import_optional_dependency('polars')
    expr = pl.all()
    slice_array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_uint8))
    expr.__setstate__(bytes(slice_array[0:raw.contents.len]))
    return expr


def _slice_to_exprplan(raw: FfiSlicePtr):
    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    ptr_data: list[ctypes.c_void_p] = void_array_ptr[0:raw.contents.len]

    plan = _slice_to_lazyframe(ctypes.cast(ptr_data[0], FfiSlicePtr))
    expr = _slice_to_expr(ctypes.cast(ptr_data[1], FfiSlicePtr))
    fill = _slice_to_expr(ctypes.cast(ptr_data[2], FfiSlicePtr)) if raw.contents.len == 3 else None

    from collections import namedtuple

    ExprPlan = namedtuple("ExprPlan", ["plan", "expr", "fill"])
    return ExprPlan(plan, expr, fill)

def _to_optional_u32_void_ptr(v):
    if v is None:
        return
    check_c_int_cast(v, "u32")
    return ctypes.cast(ctypes.pointer(ctypes.c_uint32(v)), ctypes.c_void_p)


def _from_optional_u32_void_ptr(ptr):
    u32_ptr = ctypes.cast(ptr, ctypes.POINTER(ctypes.c_uint32))
    return u32_ptr[0] if u32_ptr else None


def _slice_to_margin(raw: FfiSlicePtr):
    from opendp.extras.polars import Margin

    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    ptr_data: list[ctypes.c_void_p] = void_array_ptr[0: raw.contents.len]

    invariant = ctypes.cast(ptr_data[3], ctypes.c_char_p)
    return Margin(
        by=c_to_py(ctypes.cast(ptr_data[0], AnyObjectPtr)),
        max_length=_from_optional_u32_void_ptr(ptr_data[1]),
        max_groups=_from_optional_u32_void_ptr(ptr_data[2]),
        invariant=invariant.value.decode() if invariant.value else None, # type: ignore[arg-type]
    )

def _check_polars_by(by):
    if isinstance(by, str):
        raise ValueError(f"by ({by}) must be a sequence type; Did you mean [\"{by}\"]?")

    if not isinstance(by, Sequence):
        raise ValueError(f"by ({by}) must be a sequence type")


def _margin_to_slice(val) -> FfiSlicePtr:
    from opendp.extras.polars import Margin
    if not isinstance(val, Margin):
        raise ValueError(f"expected Polars Margin, got {val}") # pragma: no cover

    _check_polars_by(val.by)
    by = ctypes.cast(py_to_c(val.by, c_type=AnyObjectPtr, type_name="Vec<Expr>"), ctypes.c_void_p)

    max_length = _to_optional_u32_void_ptr(val.max_length)
    max_groups = _to_optional_u32_void_ptr(val.max_groups)

    def str_to_nullptr(s):
        return ctypes.cast(ctypes.c_char_p(s.encode()), ctypes.c_void_p)

    invariant = str_to_nullptr(val.invariant) if val.invariant else None

    array = (ctypes.c_void_p * 4)(by, max_length, max_groups, invariant)
    return _wrap_in_slice(ctypes.pointer(array), 4)

def _slice_to_group_bound(raw: FfiSlicePtr):
    from opendp.extras.polars import Bound

    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    ptr_data: list[ctypes.c_void_p] = void_array_ptr[0: raw.contents.len]

    return Bound(
        by=c_to_py(ctypes.cast(ptr_data[0], AnyObjectPtr)),
        per_group=_from_optional_u32_void_ptr(ptr_data[1]),
        num_groups=_from_optional_u32_void_ptr(ptr_data[2]),
    )


def _bound_to_slice(val) -> FfiSlicePtr:
    from opendp.extras.polars import Bound
    assert isinstance(val, Bound)

    _check_polars_by(val.by)
    by = ctypes.cast(py_to_c(val.by, c_type=AnyObjectPtr, type_name="Vec<Expr>"), ctypes.c_void_p)

    num_groups = _to_optional_u32_void_ptr(val.num_groups)
    per_group = _to_optional_u32_void_ptr(val.per_group)

    array = (ctypes.c_void_p * 3)(by, per_group, num_groups)
    return _wrap_in_slice(ctypes.pointer(array), 3)



def _dataframe_to_slice(val) -> FfiSlicePtr:
    pl = import_optional_dependency('polars')
    if not isinstance(val, pl.DataFrame):
        raise ValueError("expected Polars DataFrame")  # pragma: no cover

    slices = list(_series_to_slice(s) for s in val.get_columns())
    raw = _wrap_in_slice(ctypes.pointer((FfiSlicePtr * val.width)(*slices)), val.width)
    # extend the lifetime of each series' slice to that of the frame slice
    raw.depends_on(slices)
    return raw

def _slice_to_dataframe(raw: FfiSlicePtr):
    pl = import_optional_dependency('polars')
    slice_array = ctypes.cast(raw.contents.ptr, FfiSlicePtr)
    series = [_slice_to_series(FfiSlicePtr(ffislice)) for ffislice in slice_array[0:raw.contents.len]]
    return pl.DataFrame(series)


def _series_to_slice(val) -> FfiSlicePtr:
    from opendp._data import new_arrow_array, arrow_array_free

    pl = import_optional_dependency('polars')
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
    pl = import_optional_dependency('polars')
    pyarrow = import_optional_dependency('pyarrow')
    slice_array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    array_ptr, schema_ptr, name_ptr = slice_array[0:3]

    arrow_array = pyarrow.Array._import_from_c(array_ptr, schema_ptr)
    series = pl.from_arrow(arrow_array)

    name_bytes = ctypes.cast(name_ptr, ctypes.c_char_p).value
    if name_bytes is not None:
        series = series.rename(name_bytes.decode())
    return series


def _slice_to_anyobject(raw: FfiSlicePtr):
    """Unpack an AnyObject from a slice.

    Used to return an Option<AnyObject> from Rust to Python.
    """

    obj = ctypes.cast(raw.contents.ptr, AnyObjectPtr)
    ret = c_to_py(obj)
    # don't free obj, because it is owned by Rust
    # c_to_py cares that the type is AnyObject, so this needs to happen after c_to_py
    obj.__class__ = ctypes.POINTER(AnyObject) # type: ignore[assignment]
    
    return ret


def _wrap_in_slice(ptr, len_: int) -> FfiSlicePtr:
    return FfiSlicePtr(FfiSlice(ctypes.cast(ptr, ctypes.c_void_p), len_))


def _invoke_py_callback(c_arg, userdata):
    from opendp._convert import c_to_py, py_to_c
    func, TO = userdata

    try:
        # 1. convert AnyObject to Python type
        py_arg = c_to_py(c_arg)
        # don't free c_arg, because it is owned by Rust
        c_arg.__class__ = ctypes.POINTER(AnyObject)

        # 2. invoke the user-supplied function
        py_out = func(py_arg)

        # 3. convert back to an AnyObject
        c_out = py_to_c(py_out, c_type=AnyObjectPtr, type_name=TO)
        # don't free c_out, because we are giving ownership to Rust
        c_out.__class__ = ctypes.POINTER(AnyObject)

        # 4. pack up into an FfiResult
        lib.ffiresult_ok.argtypes = [ctypes.c_void_p]
        lib.ffiresult_ok.restype = ctypes.c_void_p
        return lib.ffiresult_ok(ctypes.addressof(c_out.contents))

    except Exception:
        import traceback
        lib.ffiresult_err.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        lib.ffiresult_err.restype = ctypes.c_void_p
        return lib.ffiresult_err(
            ctypes.c_char_p("Continued stack trace from Exception in user-defined function".encode()),
            ctypes.c_char_p(traceback.format_exc().encode()),
        )


_CALLBACK_DISPATCH = CallbackFnValue(_invoke_py_callback)


def _wrap_py_func(func, TO):
    # `userdata` is the opaque host-owned payload carried alongside the shared
    # callback trampoline. Here it stores the Python callable and the declared
    # output type so the dispatcher can recover the closure environment later.
    userdata = ExtrinsicObject(ctypes.py_object((func, TO)))
    return ctypes.pointer(CallbackFn(_CALLBACK_DISPATCH, userdata))


# The output type cannot be an `ctypes.POINTER(FfiResult)` due to:
#   https://bugs.python.org/issue5710#msg85731
#                                   (answer         , query       , is_internal  , userdata        )
TransitionFnValue = ctypes.CFUNCTYPE(ctypes.c_void_p, AnyObjectPtr, ctypes.c_bool, ctypes.py_object)

class TransitionFn(ctypes.Structure):
    _fields_ = [
        ("callback", TransitionFnValue),
        ("userdata", ExtrinsicObject)
    ]

class TransitionFnPtr(ctypes.POINTER(TransitionFn)): # type: ignore[misc]
    _type_ = TransitionFn


def _invoke_py_transition(c_query, c_is_internal: ctypes.c_bool, userdata):
    from opendp._convert import c_to_py, py_to_c
    py_transition, A = userdata

    try:
        # 1. convert to Python type
        py_query = c_to_py(c_query)
        py_is_internal = c_is_internal
        # don't free c_arg, because it is owned by Rust
        c_query.__class__ = ctypes.POINTER(AnyObject)

        # 2. invoke the user-supplied function
        py_out = py_transition(py_query, py_is_internal)

        # 3. convert back to an AnyObject
        c_out = py_to_c(py_out, c_type=AnyObjectPtr, type_name=A)
        # don't free c_out, because we are giving ownership to Rust
        c_out.__class__ = ctypes.POINTER(AnyObject)

        # 4. pack up into an FfiResult
        lib.ffiresult_ok.argtypes = [ctypes.c_void_p]
        lib.ffiresult_ok.restype = ctypes.c_void_p
        return lib.ffiresult_ok(ctypes.addressof(c_out.contents))

    except Exception:
        import traceback
        lib.ffiresult_err.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        lib.ffiresult_err.restype = ctypes.c_void_p
        return lib.ffiresult_err(
            ctypes.c_char_p("Continued stack trace from Exception in user-defined function".encode()),
            ctypes.c_char_p(traceback.format_exc().encode()),
        )


_TRANSITION_DISPATCH = TransitionFnValue(_invoke_py_transition)


_TO_SLICE_FROM_STR = {
    "BitVector": _bitvector_to_slice,
    "ExtrinsicObject": _extrinsic_to_slice,
    "AnyMeasurement": lambda value: _wrap_in_slice(value, 1),
    "String": _string_to_slice,
    "Series": _series_to_slice,
    "LazyFrame": _lazyframe_to_slice,
    "DslPlan": _lazyframe_to_slice,
    "DataFrame": _dataframe_to_slice,
    "Margin": _margin_to_slice,
    "Expr": _expr_to_slice,
    "Bound": _bound_to_slice,
    "Bounds": lambda v: _vector_to_slice(v, RuntimeType("Vec", ["Bound"])),
}

_TO_SLICE_FROM_ORIGIN = {
    "Vec": _vector_to_slice,
    "NDArray": _numpy_to_slice,
    "HashMap": _hashmap_to_slice,
    "Function": _function_to_slice,
    "Tuple": _tuple_to_slice,
}

_FROM_SLICE_STR = {
    "BitVector": _slice_to_bitvector,
    "String": _slice_to_string,
    "LazyFrame": _slice_to_lazyframe,
    "DataFrame": _slice_to_dataframe,
    "Series": _slice_to_series,
    "Expr": _slice_to_expr,
    "ExprPlan": _slice_to_exprplan,
    "Bound": _slice_to_group_bound,
    "Margin": _slice_to_margin,
    "Bounds": lambda raw: _slice_to_vector(raw, RuntimeType("Vec", ["Bound"])),
    "ExtrinsicObject": _slice_to_extrinsic,
}

_FROM_SLICE_RT_TYPE = {
    "Vec": _slice_to_vector,
    "AnyObject": _slice_to_anyobject,
    "NDArray": _slice_to_numpy,
    "Function": lambda raw, _: _slice_to_function(raw),
    "HashMap": lambda raw, _: _slice_to_hashmap(raw),
    "Tuple": _slice_to_tuple,
    "Option": _slice_to_option,
}

_DOMAIN_CLASS_FROM_RT_TYPE = {
    OptionDomain.__name__: OptionDomain,
    AtomDomain.__name__: AtomDomain,
    VectorDomain.__name__: VectorDomain,
    SeriesDomain.__name__: SeriesDomain,
    LazyFrameDomain.__name__: LazyFrameDomain,
    ExtrinsicDomain.__name__: ExtrinsicDomain,
}
_DISTANCE_CLASS_FROM_RT_TYPE = {
    FrameDistance.__name__: FrameDistance,
    SymmetricIdDistance.__name__: SymmetricIdDistance,
    ChangeOneIdDistance.__name__: ChangeOneIdDistance,
    ExtrinsicDistance.__name__: ExtrinsicDistance,
}

_MEASURE_CLASS_FROM_RT_TYPE = {
    "Approximate": ApproximateDivergence,
    ExtrinsicDivergence.__name__: ExtrinsicDivergence,
}
_NUMPY_COMPATIBLE_ATOM_TYPES = frozenset(ATOM_MAP) - {'AnyMeasurementPtr', 'AnyTransformationPtr'}

INT_SIZES = {
    ty: c_int_limits(ty) for ty in (
        'u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64', 'usize',
    )
}


