from typing import Sequence, Tuple, List, Union, Dict

from opendp._lib import *

from opendp.mod import UnknownTypeException, OpenDPException, Transformation, Measurement
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
    'bool': ctypes.c_bool,
}


def py_to_c(value: Any, c_type, type_name: Union[RuntimeType, str] = None):
    """Map from python `value` to ctypes `c_type`.

    :param value: value to convert to c_type
    :param c_type: expected ctypes type to convert to
    :param type_name: optional. rust_type to check inferred rust_type of value against, and/or specify bit depth
    :return: value converted to ctypes representation
    """

    if isinstance(value, c_type):
        return value

    if type_name is not None:
        RuntimeType.assert_is_similar(RuntimeType.parse(type_name), RuntimeType.infer(value))

        # exit early with a null pointer if trying to load an Option type with a None value
        if isinstance(type_name, RuntimeType) and type_name.origin == "Option":
            if value is None:
                return
            type_name = type_name.args[0]

    if c_type == ctypes.c_void_p:
        assert type_name is not None

        rust_type = str(type_name)
        if rust_type in ATOM_MAP:
            return ctypes.byref(ATOM_MAP[rust_type](value))

        if rust_type == "String":
            return ctypes.c_char_p(value.encode())

        raise UnknownTypeException(rust_type)

    if c_type == AnyObjectPtr:
        from opendp._data import slice_as_object
        return slice_as_object(value, type_name)

    if c_type == AnyMeasureDistancePtr:
        from opendp._data import slice_as_measure_distance
        return slice_as_measure_distance(value, type_name)

    if c_type == AnyMetricDistancePtr:
        from opendp._data import slice_as_metric_distance
        return slice_as_metric_distance(value, type_name)

    if c_type == FfiSlicePtr:
        assert type_name is not None
        return _py_to_slice(value, str(type_name))

    if isinstance(value, RuntimeType):
        value = str(value)

    if isinstance(value, str):
        value = value.encode()

    if not isinstance(value, c_type):
        value = c_type(value)

    return value


def c_to_py(value):
    """Map from ctypes `value` to python value.
    It is assumed that the c type is simpler than in py_to_c, as the library returns fewer types.

    :param value: data in ctypes format
    :return: copy of data in python representation
    """
    if isinstance(value, AnyObjectPtr):
        from opendp._data import object_type, object_as_slice, to_string, slice_free
        ffi_slice = object_as_slice(value)
        try:
            return _slice_to_py(ffi_slice, object_type(value))
        except UnknownTypeException:
            raise
        except Exception as err:
            print("MASKED ERROR:", err)
            print("using string fallback")
            # raise err
            # If we fail, resort to string representation.
            # TODO: Remove this fallback once we have composition and/or tuples sorted out.
            return to_string(value)
        finally:
            slice_free(ffi_slice)

    if isinstance(value, ctypes.c_char_p):
        from opendp._data import str_free
        value_contents = value.value.decode()
        str_free(value)
        return value_contents

    if isinstance(value, BoolPtr):
        from opendp._data import bool_free
        value_contents = value.contents.value
        bool_free(value)
        return value_contents

    if isinstance(value, (Transformation, Measurement)):
        # these types are meant to pass through
        return value

    if isinstance(value, ctypes.c_void_p):
        # returned void pointers are interpreted as None
        return

    return value


def _slice_to_py(raw: FfiSlicePtr, type_name: str) -> Any:
    """Convert from `raw` FfiSlicePtr to python type.
    This is the postprocessing step after _object_to_slice that unloads data from a ctypes representation.
    External checks allow this function to assume that `raw` is compatible with the type_name type.

    :param raw: raw pointer to an FfiSlice that will be unloaded into a python type
    :param type_name: rust type name that determines the python type to unload into
    :return: a standard python reference-counted data type
    """
    if type_name in ATOM_MAP:
        return _slice_to_scalar(raw, type_name)

    if type_name.startswith("Vec<") and type_name.endswith('>'):
        return _slice_to_vector(raw, type_name)

    if type_name.startswith("HashMap<") and type_name.endswith('>'):
        return _slice_to_hashmap(raw)

    if type_name.startswith('(') and type_name.endswith(')'):
        return _slice_to_tuple(raw, type_name)

    if type_name == "String":
        return _slice_to_string(raw)

    raise UnknownTypeException(type_name)


def _py_to_slice(value: Any, type_name: str) -> FfiSlicePtr:
    """Convert from python `value` to FfiSlicePtr.
    The initial preprocessing step for _slice_to_object that loads data into a ctypes representation.
    External checks allow this function to assume that `value` is compatible with the type_name type.

    :param value: data to load into an FfiSlice
    :param type_name: rust type name to load value into.
    :return: pointer to an FfiSlice owned by python.
    """
    if type_name in ATOM_MAP:
        return _scalar_to_slice(value, type_name)

    if type_name.startswith("Vec<") and type_name.endswith('>'):
        return _vector_to_slice(value, type_name)

    if type_name.startswith("HashMap<") and type_name.endswith('>'):
        return _hashmap_to_slice(value, type_name)

    if type_name.startswith('(') and type_name.endswith(')'):
        return _tuple_to_slice(value, type_name)

    if type_name == "String":
        return _string_to_slice(value)

    raise UnknownTypeException(type_name)


def _scalar_to_slice(val, type_name: str) -> FfiSlicePtr:
    # ctypes.byref has edge-cases that cause use-after-free errors. ctypes.pointer fixes these edge-cases
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
        raise OpenDPException(f"Cannot cast a non-list type to a vector")

    if inner_type_name == "String":
        def str_to_slice(val):
            return ctypes.c_char_p(val.encode())
        array = (ctypes.c_char_p * len(val))(*map(str_to_slice, val))
        return _wrap_in_slice(array, len(val))

    if inner_type_name not in ATOM_MAP:
        raise OpenDPException(f"Members must be one of {ATOM_MAP.keys()}. Found {inner_type_name}.")

    if val:
        # check that actual type can be represented by the inner_type_name
        equivalence_class = ATOM_EQUIVALENCE_CLASSES[str(RuntimeType.infer(val[0]))]
        if inner_type_name not in equivalence_class:
            raise OpenDPException("Data cannot be represented by the suggested type_name")

    array = (ATOM_MAP[inner_type_name] * len(val))(*val)
    return _wrap_in_slice(array, len(val))


def _slice_to_vector(raw: FfiSlicePtr, type_name: str) -> List[Any]:
    assert type_name[:3] == 'Vec'
    inner_type_name = type_name[4:-1]

    if inner_type_name == 'String':
        array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_char_p))[0:raw.contents.len]
        return list(map(lambda v: v.decode(), array))

    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[inner_type_name]))[0:raw.contents.len]


def _tuple_to_slice(val: Tuple[Any, ...], type_name: str) -> FfiSlicePtr:
    inner_type_names = [i.strip() for i in type_name[1:-1].split(",")]
    if not isinstance(val, tuple):
        raise OpenDPException("Cannot cast a non-tuple type to a tuple")
    # TODO: temporary check
    if len(inner_type_names) != 2:
        raise OpenDPException("Only 2-tuples are currently supported.")
    # TODO: temporary check
    if len(set(inner_type_names)) > 1:
        raise OpenDPException("Only homogeneously-typed tuples are currently supported.")

    if len(inner_type_names) != len(val):
        raise OpenDPException("type_name members must have same length as tuple")

    if any(t not in ATOM_MAP for t in inner_type_names):
        raise OpenDPException(f"Tuple members must be one of {ATOM_MAP.keys()}")

    # check that actual type can be represented by the inner_type_name
    for v, inner_type_name in zip(val, inner_type_names):
        equivalence_class = ATOM_EQUIVALENCE_CLASSES[str(RuntimeType.infer(v))]
        if inner_type_name not in equivalence_class:
            raise OpenDPException("Data cannot be represented by the suggested type_name")

    # ctypes.byref has edge-cases that cause use-after-free errors. ctypes.pointer fixes these edge-cases
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


def _hashmap_to_slice(val: Dict[Any, Any], type_name: str) -> FfiSlicePtr:
    key_type, val_type = [i.strip() for i in type_name[8:-1].split(",")]
    keys: AnyObjectPtr = py_to_c(list(val.keys()), type_name=f"Vec<{key_type}>", c_type=AnyObjectPtr)
    vals: AnyObjectPtr = py_to_c(list(val.values()), type_name=f"Vec<{val_type}>", c_type=AnyObjectPtr)
    return _wrap_in_slice(ctypes.pointer((AnyObjectPtr * 2)(keys, vals)), 2)


def _slice_to_hashmap(raw: FfiSlicePtr) -> Dict[Any, Any]:
    keys_obj, vals_obj = ctypes.cast(raw.contents.ptr, ctypes.POINTER(AnyObjectPtr))[0:2]
    return dict(zip(c_to_py(keys_obj), c_to_py(vals_obj)))


def _wrap_in_slice(ptr, len_: int) -> FfiSlicePtr:
    return FfiSlicePtr(FfiSlice(ctypes.cast(ptr, ctypes.c_void_p), len_))
