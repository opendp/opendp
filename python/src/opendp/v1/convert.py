import ctypes
from typing import Any, Sequence, Tuple, List

from opendp.v1.mod import UnknownTypeException, FfiSlice, OdpException, AnyObjectPtr, FfiSlicePtr, BoolPtr, \
    AnyTransformationPtr, AnyMeasurementPtr, ATOM_EQUIVALENCE_CLASSES, AnyMetricDistancePtr, AnyMeasureDistancePtr
from opendp.v1.typing import RuntimeType, RuntimeTypeDescriptor

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
    'bool': ctypes.c_bool,
}


def _wrap_in_slice(ptr, len_: int) -> FfiSlicePtr:
    return FfiSlicePtr(FfiSlice(ctypes.cast(ptr, ctypes.c_void_p), len_))


def _scalar_to_slice(val, type_name: str) -> FfiSlicePtr:
    return _wrap_in_slice(ctypes.pointer(ATOM_MAP[type_name](val)), 1)


def _slice_to_scalar(raw: FfiSlicePtr, type_name: str):
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[type_name])).contents.value


def _string_to_slice(val: str) -> FfiSlicePtr:
    return _wrap_in_slice(ctypes.c_char_p(val.encode()), len(val) + 1)


def _slice_to_string(raw: FfiSlicePtr) -> str:
    return ctypes.cast(raw.contents.ptr, ctypes.c_char_p).value.decode()


def _vector_to_slice(val: Sequence[Any], type_name) -> FfiSlicePtr:
    assert type_name[:4] == 'Vec<'
    inner_type_name = type_name[4:-1]
    if not isinstance(val, list):
        raise OdpException(f"Cannot cast a non-list type to a vector")

    if inner_type_name not in ATOM_MAP:
        raise OdpException(f"Members must be one of {ATOM_MAP.keys()}. Found {inner_type_name}.")

    if val:
        # check that actual type can be represented by the inner_type_name
        equivalence_class = ATOM_EQUIVALENCE_CLASSES[str(RuntimeType.infer(val[0]))]
        if inner_type_name not in equivalence_class:
            raise OdpException("Data cannot be represented by the suggested type_name")

    array = (ATOM_MAP[inner_type_name] * len(val))(*val)
    return _wrap_in_slice(array, len(val))


def _slice_to_vector(raw: FfiSlicePtr, type_name: str) -> List[Any]:
    assert type_name[:3] == 'Vec'
    inner_type_name = type_name[4:-1]
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[inner_type_name]))[0:raw.contents.len]


def _tuple_to_slice(val: Tuple[Any, ...], type_name: str) -> FfiSlicePtr:
    inner_type_names = [i.strip() for i in type_name[1:-1].split(",")]
    if not isinstance(val, tuple):
        raise OdpException("Cannot cast a non-tuple type to a tuple")
    # TODO: temporary check
    if len(inner_type_names) != 2:
        raise OdpException("Only 2-tuples are currently supported.")
    # TODO: temporary check
    if len(set(inner_type_names)) > 1:
        raise OdpException("Only homogeneously-typed tuples are currently supported.")

    if len(inner_type_names) != len(val):
        raise OdpException("type_name members must have same length as tuple")

    if any(t not in ATOM_MAP for t in inner_type_names):
        raise OdpException(f"Tuple members must be one of {ATOM_MAP.keys()}")

    # check that actual type can be represented by the inner_type_name
    for v, inner_type_name in zip(val, inner_type_names):
        equivalence_class = ATOM_EQUIVALENCE_CLASSES[str(RuntimeType.infer(v))]
        if inner_type_name not in equivalence_class:
            raise OdpException("Data cannot be represented by the suggested type_name")

    ptr_data = (ctypes.cast(ctypes.pointer(ATOM_MAP[name](v)), ctypes.c_void_p)
                for v, name in zip(val, inner_type_names))
    array = (ctypes.c_void_p * len(val))(*ptr_data)
    return _wrap_in_slice(ctypes.pointer(array), len(val))


def _slice_to_tuple(raw: FfiSlicePtr, type_name: str) -> Tuple[Any, ...]:
    inner_type_names = [i.strip() for i in type_name[1:-1].split(",")]
    # typed pointer
    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    # list of void*
    ptr_data = void_array_ptr[0:raw.contents.len]
    # tuple of instances of python types
    return tuple(ctypes.cast(void_p, ctypes.POINTER(ATOM_MAP[name])).contents.value
                 for void_p, name in zip(ptr_data, inner_type_names))


def _slice_to_py(raw: FfiSlicePtr, type_name: str) -> Any:
    if type_name in ATOM_MAP:
        return _slice_to_scalar(raw, type_name)

    if type_name.startswith("Vec<") and type_name.endswith('>'):
        return _slice_to_vector(raw, type_name)

    if type_name.startswith('(') and type_name.endswith(')'):
        return _slice_to_tuple(raw, type_name)

    if type_name == "String":
        return _slice_to_string(raw)

    raise UnknownTypeException(type_name)


def _py_to_slice(val: Any, type_name: str) -> FfiSlicePtr:
    if type_name in ATOM_MAP:
        return _scalar_to_slice(val, type_name)

    if type_name.startswith("Vec<") and type_name.endswith('>'):
        return _vector_to_slice(val, type_name)

    if type_name.startswith('(') and type_name.endswith(')'):
        return _tuple_to_slice(val, type_name)

    if type_name == "String":
        return _string_to_slice(val)

    raise UnknownTypeException(type_name)


def py_to_metric_distance(val: Any, type_name: RuntimeTypeDescriptor = None) -> AnyMetricDistancePtr:
    if isinstance(val, AnyMetricDistancePtr):
        return val

    from opendp.v1.data import _slice_as_metric_distance
    return _slice_as_metric_distance(val, type_name)


def py_to_measure_distance(val: Any, type_name: RuntimeTypeDescriptor = None) -> AnyMeasureDistancePtr:
    if isinstance(val, AnyMeasureDistancePtr):
        return val

    from opendp.v1.data import _slice_as_measure_distance
    return _slice_as_measure_distance(val, type_name)


def py_to_object(val: Any, type_name: RuntimeTypeDescriptor = None) -> AnyObjectPtr:
    if isinstance(val, AnyObjectPtr):
        return val

    from opendp.v1.data import _slice_as_object
    return _slice_as_object(val, type_name)


def object_to_py(obj: AnyObjectPtr) -> Any:
    from opendp.v1.data import _object_type, _object_as_slice, _to_string, _slice_free
    ffi_slice = _object_as_slice(obj)
    try:
        return _slice_to_py(ffi_slice, _object_type(obj))
    except UnknownTypeException:
        raise
    except Exception as err:
        print("MASKED ERROR:", err)
        print("using string fallback")
        # raise err
        # If we fail, resort to string representation.
        # TODO: Remove this fallback once we have composition and/or tuples sorted out.
        return _to_string(obj)
    finally:
        _slice_free(ffi_slice)


def py_to_ptr(val: Any, type_name: str = None):
    """map from python val to void *"""
    type_name = RuntimeType.parse_or_infer(type_name=type_name, public_example=val)
    # the explicit type_name must be compatible with the actual data
    RuntimeType.assert_is_similar(type_name, RuntimeType.infer(val))

    if type_name in ATOM_MAP:
        return ctypes.byref(ATOM_MAP[type_name](val))

    if type_name == "String":
        return ctypes.c_char_p(val.encode())

    raise UnknownTypeException(type_name)


def py_to_c(val: Any, c_type, rust_type=None):
    """map from python val to any c type"""
    if isinstance(val, c_type):
        return val

    if rust_type is not None:
        RuntimeType.assert_is_similar(rust_type, RuntimeType.infer(val))

    if c_type == FfiSlicePtr:
        assert rust_type is not None
        return _py_to_slice(val, str(rust_type))

    if isinstance(val, RuntimeType):
        val = str(val)

    if isinstance(val, str):
        val = val.encode()

    if not isinstance(val, c_type):
        val = c_type(val)

    return val


def c_to_py(c_value):
    if isinstance(c_value, ctypes.c_char_p):
        from opendp.v1.data import _str_free
        value = c_value.value.decode()
        _str_free(c_value)
        return value

    if isinstance(c_value, BoolPtr):
        from opendp.v1.data import _bool_free
        value = c_value.contents.value
        _bool_free(c_value)
        return value

    if isinstance(c_value, AnyObjectPtr):
        return object_to_py(c_value)

    if isinstance(c_value, (AnyTransformationPtr, AnyMeasurementPtr)):
        return c_value

    return c_value

