import ctypes
from typing import Any, Sequence, Tuple, List

from opendp.v1.mod import lib, UnknownTypeException, FfiSlice, OdpException, FfiObject, FfiObjectPtr, FfiSlicePtr
from opendp.v1.typing import RuntimeType, RuntimeTypeDescriptor

# list all acceptable alternative types for each default type
ATOM_EQUIVALENCE_CLASSES = {
    'i32': ['u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64'],
    'f64': ['f32', 'f64'],
    'bool': ['bool']
}

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


def _scalar_to_slice(val: Any, type_name: str) -> FfiSlicePtr:
    return _wrap_in_slice(ctypes.byref(ATOM_MAP[type_name](val)), 1)


def _slice_to_scalar(raw: FfiSlicePtr, type_name: str) -> Any:
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
        raise OdpException(f"Members must be one of {ATOM_MAP.keys()}")

    if val:
        # check that actual type can be represented by the inner_type_name
        equivalence_class = ATOM_EQUIVALENCE_CLASSES[str(RuntimeType.infer(val[0]))]
        if inner_type_name not in equivalence_class:
            raise OdpException("Data cannot be represented by the suggested type_name")

    array = (ATOM_MAP[inner_type_name] * len(val))(*val)
    return _wrap_in_slice(array, len(val))


def _slice_to_vector(raw: FfiSlicePtr, type_name: str) -> List[Any]:
    assert type_name[:4] == 'Vec'
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
    return _wrap_in_slice(ctypes.byref(array), len(val))


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


def py_to_object(val: Any, type_name: RuntimeTypeDescriptor = None) -> FfiObjectPtr:
    if isinstance(val, FfiObjectPtr):
        return val
    type_name = str(RuntimeType.parse_or_infer(type_name=type_name, public_example=val))
    ffi_slice = _py_to_slice(val, type_name)
    from opendp.v1.data import slice_as_object
    return slice_as_object(ffi_slice, type_name)


def object_to_py(obj: FfiObjectPtr) -> Any:
    from opendp.v1.data import object_type, object_as_slice, to_string, slice_free
    type_name_ptr = object_type(obj)
    type_name = type_name_ptr.value.decode()
    ffi_slice = object_as_slice(obj)
    try:
        return _slice_to_py(ffi_slice, type_name)
    except UnknownTypeException:
        raise
    except Exception as err:
        print("MASKED ERROR:", err)
        print("using string fallback")
        # raise err
        # If we fail, resort to string representation.
        # TODO: Remove this fallback once we have composition and/or tuples sorted out.
        return to_string(obj).value.decode()
    finally:
        slice_free(ffi_slice)


def py_to_ptr(val: Any, type_name: str = None):
    """map from python val to void *"""
    # TODO: raise if val is inconsistent with type_name
    if type_name is None:
        type_name = RuntimeType.infer(val)

    print(val, type_name)
    print('TODO: py_to_ptr without type_name check')
    # type_name.validate(val)

    if type_name in ATOM_MAP:
        return ctypes.byref(ATOM_MAP[type_name](val))

    if type_name == "String":
        return ctypes.c_char_p(val.encode())

    raise UnknownTypeException(type_name)


def py_to_c(val: Any, c_type):
    """map from python val to any c type"""
    if isinstance(val, RuntimeType):
        val = str(val)

    if isinstance(val, str):
        val = val.encode()

    if not isinstance(val, c_type):
        val = c_type(val)

    return val
