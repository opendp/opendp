from typing import Sequence, Tuple, List, Union, Dict

from opendp._lib import *

from opendp.mod import UnknownTypeException, OpenDPException, Transformation, Measurement, SMDCurve, Queryable
from opendp.typing import RuntimeType, Vec

try:
    import numpy as np
except ImportError:
    np = None

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

def c_int_limits(type_name):
    c_int_type = ATOM_MAP[type_name]
    signed = c_int_type(-1).value < c_int_type(0).value
    bit_size = ctypes.sizeof(c_int_type) * 8
    signed_limit = 2 ** (bit_size - 1)
    return (-signed_limit, signed_limit - 1) if signed else (0, 2 * signed_limit - 1)

INT_SIZES = {
    ty: c_int_limits(ty) for ty in (
        'u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64', 'usize',
    )
}

def check_c_int_cast(v, type_name):
    lower, upper = INT_SIZES[type_name]
    if not (lower <= v <= upper):
        raise ValueError(f"value is not representable by {type_name}")


def py_to_c(value: Any, c_type, type_name: Union[RuntimeType, str] = None):
    """Map from Python `value` to ctypes `c_type`.

    :param value: value to convert to c_type
    :param c_type: expected ctypes type to convert to
    :param type_name: optional. rust_type to check inferred rust_type of value against, and/or specify bit depth
    :return: value converted to ctypes representation
    """

    if isinstance(type_name, str):
        type_name = RuntimeType.parse(type_name)

    if isinstance(value, c_type):
        return value

    if c_type == CallbackFn:
        return _wrap_py_func(value, type_name)

    # check that the type name is consistent with the value
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
            if rust_type in INT_SIZES:
                check_c_int_cast(value, rust_type)
            return ctypes.byref(ATOM_MAP[rust_type](value))

        if rust_type == "String":
            return ctypes.c_char_p(value.encode())

        raise UnknownTypeException(rust_type)

    if c_type == AnyObjectPtr:
        from opendp._data import slice_as_object
        return slice_as_object(value, type_name)

    if c_type == FfiSlicePtr:
        assert type_name is not None
        return _py_to_slice(value, type_name)

    if isinstance(value, RuntimeType):
        value = str(value)

    if isinstance(value, str):
        value = value.encode()

    if not isinstance(value, c_type):
        # throw an error if the value is already a c_type, but the wrong one
        # (like passing a Metric into an argument expecting a Domain)
        if hasattr(value, "_type_"):
            raise ValueError(f"Cannot convert {value} to {c_type}")
        value = c_type(value)

    return value


def c_to_py(value):
    """Map from ctypes `value` to Python value.
    It is assumed that the C type is simpler than in py_to_c, as the library returns fewer types.

    :param value: data in ctypes format
    :return: copy of data in Python representation
    """
    if isinstance(value, AnyObjectPtr):
        from opendp._data import object_type, object_as_slice, slice_free
        obj_type = object_type(value)
        if "SMDCurve" in obj_type:
            return SMDCurve(value)
        if "Queryable" in obj_type:
            return Queryable(value)
        ffi_slice = object_as_slice(value)
        try:
            return _slice_to_py(ffi_slice, RuntimeType.parse(obj_type))
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


def _slice_to_py(raw: FfiSlicePtr, type_name: Union[RuntimeType, str]) -> Any:
    """Convert from `raw` FfiSlicePtr to Python type.
    This is the postprocessing step after _object_to_slice that unloads data from a ctypes representation.
    External checks allow this function to assume that `raw` is compatible with the type_name type.

    :param raw: raw pointer to an FfiSlice that will be unloaded into a Python type
    :param type_name: Rust type name that determines the Python type to unload into
    :return: a standard Python reference-counted data type
    """
    if isinstance(type_name, str) and type_name in ATOM_MAP:
        return _slice_to_scalar(raw, type_name)
    
    if type_name == "String":
        return _slice_to_string(raw)

    if type_name.origin == "Vec":
        return _slice_to_vector(raw, type_name)

    if type_name.origin == "HashMap":
        return _slice_to_hashmap(raw)

    if type_name.origin == "Tuple":
        return _slice_to_tuple(raw, type_name)

    raise UnknownTypeException(type_name)


def _py_to_slice(value: Any, type_name: Union[RuntimeType, str]) -> FfiSlicePtr:
    """Convert from Python `value` to FfiSlicePtr.
    The initial preprocessing step for _slice_to_object that loads data into a ctypes representation.
    External checks allow this function to assume that `value` is compatible with the type_name type.

    :param value: data to load into an FfiSlice
    :param type_name: Rust type name to load value into.
    :return: pointer to an FfiSlice owned by Python.
    """
    if isinstance(type_name, str) and type_name in ATOM_MAP:
        return _scalar_to_slice(value, type_name)
    
    if type_name == "AnyMeasurement":
        return _wrap_in_slice(value, 1)

    if type_name == "String":
        return _string_to_slice(value)

    if type_name.origin == "Vec":
        return _vector_to_slice(value, type_name)

    if type_name.origin == "HashMap":
        return _hashmap_to_slice(value, type_name)

    if type_name.origin == "Tuple":
        return _tuple_to_slice(value, type_name)

    raise UnknownTypeException(type_name)


def _scalar_to_slice(val, type_name: str) -> FfiSlicePtr:
    if np is not None and isinstance(val, np.ndarray):
        val = val.item()
    if type_name in INT_SIZES:
        check_c_int_cast(val, type_name)
    # ctypes.byref has edge-cases that cause use-after-free errors. ctypes.pointer fixes these edge-cases
    return _wrap_in_slice(ctypes.pointer(ATOM_MAP[type_name](val)), 1)


def _slice_to_scalar(raw: FfiSlicePtr, type_name: str):
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[type_name])).contents.value


def _string_to_slice(val: str) -> FfiSlicePtr:
    if np is not None and isinstance(val, np.ndarray):
        val = val.item()
    return _wrap_in_slice(ctypes.c_char_p(val.encode()), len(val) + 1)


def _slice_to_string(raw: FfiSlicePtr) -> str:
    return ctypes.cast(raw.contents.ptr, ctypes.c_char_p).value.decode()


def _vector_to_slice(val: Sequence[Any], type_name: RuntimeType) -> FfiSlicePtr:
    assert type_name.origin == 'Vec'
    assert len(type_name.args) == 1, "Vec only has one generic argument"
    inner_type_name = type_name.args[0]

    # input is numpy array
    # TODO: can we use the underlying buffer directly?
    if np is not None and isinstance(val, np.ndarray):
        val = val.tolist()

    if not isinstance(val, list):
        raise TypeError(f"Cannot cast a non-list type to a vector")

    if inner_type_name == "String":
        def str_to_slice(val):
            return ctypes.c_char_p(val.encode())
        array = (ctypes.c_char_p * len(val))(*map(str_to_slice, val))
        return _wrap_in_slice(array, len(val))

    if isinstance(inner_type_name, RuntimeType):
        c_repr = [py_to_c(v, c_type=AnyObjectPtr, type_name=inner_type_name) for v in val]
        array = (AnyObjectPtr * len(val))(*c_repr)
        ffislice = _wrap_in_slice(array, len(val))
        ffislice.depends_on(*c_repr)
        return ffislice

    if inner_type_name not in ATOM_MAP:
        raise TypeError(f"Members must be one of {tuple(ATOM_MAP.keys())}. Found {inner_type_name}.")

    if val:
        # check that actual type can be represented by the inner_type_name
        equivalence_class = ATOM_EQUIVALENCE_CLASSES[str(RuntimeType.infer(val[0]))]
        if inner_type_name not in equivalence_class:
            raise TypeError("Data cannot be represented by the suggested type_name")

    if inner_type_name in INT_SIZES:
        for elem in val:
            check_c_int_cast(elem, inner_type_name)

    array = (ATOM_MAP[inner_type_name] * len(val))(*val)
    return _wrap_in_slice(array, len(val))


def _slice_to_vector(raw: FfiSlicePtr, type_name: RuntimeType) -> List[Any]:
    assert type_name.origin == 'Vec'
    assert len(type_name.args) == 1, "Vec only has one generic argument"
    inner_type_name = type_name.args[0]

    if inner_type_name == 'AnyObject':
        from opendp._data import ffislice_of_anyobjectptrs
        raw = ffislice_of_anyobjectptrs(raw)
        array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(AnyObjectPtr))[0:raw.contents.len]
        res = list(map(c_to_py, array))
        # when the top-level AnyObject is freed, it recursively frees all anyobjects inside of it
        # adjust the type of constituent AnyObjects so that __delete__ is not called when they are dropped
        for elem in array:
            elem.__class__ = ctypes.POINTER(AnyObject)
        return res

    if inner_type_name == 'String':
        array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_char_p))[0:raw.contents.len]
        return list(map(lambda v: v.decode(), array))

    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[inner_type_name]))[0:raw.contents.len]


def _tuple_to_slice(val: Tuple[Any, ...], type_name: RuntimeType) -> FfiSlicePtr:
    inner_type_names = type_name.args
    if not isinstance(val, tuple):
        raise TypeError("Cannot coerce a non-tuple type to a tuple")
    # TODO: temporary check
    if len(inner_type_names) != 2:
        raise OpenDPException("Only 2-tuples are currently supported.")
    # TODO: temporary check
    if len(set(inner_type_names)) > 1:
        raise OpenDPException("Only homogeneously-typed tuples are currently supported.")

    if len(inner_type_names) != len(val):
        raise TypeError("type_name members must have same length as tuple")

    for t in inner_type_names:
        if t not in ATOM_MAP:
            raise TypeError(f"Tuple members must be one of {ATOM_MAP.keys()}. Got {t}")

    # check that actual type can be represented by the inner_type_name
    for v, inner_type_name in zip(val, inner_type_names):
        equivalence_class = ATOM_EQUIVALENCE_CLASSES[str(RuntimeType.infer(v))]
        if inner_type_name not in equivalence_class:
            raise TypeError("Data cannot be represented by the suggested type_name")
        
        if inner_type_name in INT_SIZES:
            check_c_int_cast(v, inner_type_name)
    
    # ctypes.byref has edge-cases that cause use-after-free errors. ctypes.pointer fixes these edge-cases
    ptr_data = (ctypes.cast(ctypes.pointer(ATOM_MAP[name](v)), ctypes.c_void_p)
        for v, name in zip(val, inner_type_names))

    array = (ctypes.c_void_p * len(val))(*ptr_data)
    return _wrap_in_slice(ctypes.pointer(array), len(val))


def _slice_to_tuple(raw: FfiSlicePtr, type_name: RuntimeType) -> Tuple[Any, ...]:
    inner_type_names = type_name.args
    # typed pointer
    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    # list of void*
    ptr_data = void_array_ptr[0:raw.contents.len]
    # tuple of instances of Python types
    return tuple(ctypes.cast(void_p, ctypes.POINTER(ATOM_MAP[name])).contents.value
                 for void_p, name in zip(ptr_data, inner_type_names))


def _hashmap_to_slice(val: Dict[Any, Any], type_name: RuntimeType) -> FfiSlicePtr:
    key_type, val_type = type_name.args
    keys: AnyObjectPtr = py_to_c(list(val.keys()), type_name=Vec[key_type], c_type=AnyObjectPtr)
    vals: AnyObjectPtr = py_to_c(list(val.values()), type_name=Vec[val_type], c_type=AnyObjectPtr)
    ffislice = _wrap_in_slice(ctypes.pointer((AnyObjectPtr * 2)(keys, vals)), 2)

    # The __del__ destructor on `keys` and `vals` is called and memory freed when their refcounts go to zero.
    # ffislice needs keys and vals to have a lifetime at least as long as itself,
    # so we can't allow their refcount to go to zero until ffislice is freed.
    ffislice.depends_on(keys, vals)
    return ffislice


def _slice_to_hashmap(raw: FfiSlicePtr) -> Dict[Any, Any]:
    slice_array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(AnyObjectPtr))
    keys: AnyObjectPtr = slice_array[0]
    vals: AnyObjectPtr = slice_array[1]
    result = dict(zip(c_to_py(keys), c_to_py(vals)))

    # AnyObjectPtr.__del__ would free the memory behind keys and vals when this stack frame is popped.
    # But that memory has a lifetime at least as long as raw, so it cannot be freed yet.
    # Adjust the class to avoid calling AnyObjectPtr.__del__, which would free the backing memory.
    keys.__class__ = ctypes.POINTER(AnyObject)
    vals.__class__ = ctypes.POINTER(AnyObject)
    return result


def _wrap_in_slice(ptr, len_: int) -> FfiSlicePtr:
    return FfiSlicePtr(FfiSlice(ctypes.cast(ptr, ctypes.c_void_p), len_))


# The result type cannot be an `ctypes.POINTER(FfiResult)` due to:
#   https://bugs.python.org/issue5710#msg85731
#                            (output         , input       )
CallbackFn = ctypes.CFUNCTYPE(ctypes.c_void_p, AnyObjectPtr)

def _wrap_py_func(func, TO):
    from opendp._convert import c_to_py, py_to_c

    def wrapper_func(c_arg):
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
                ctypes.c_char_p(f"Continued stack trace from Exception in user-defined function".encode()),
                ctypes.c_char_p(traceback.format_exc().encode()),
            )

    return CallbackFn(wrapper_func)
