from typing import Sequence, Union, cast, MutableMapping
from inspect import signature

from opendp._lib import *
from opendp.mod import Domain, UnknownTypeException, Transformation, Measurement, PrivacyProfile, Queryable, Function
from opendp.typing import RuntimeType, RuntimeTypeDescriptor, Vec

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
_ERROR_URL_298 = "https://github.com/opendp/opendp/discussions/298"


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


def py_to_c(value: Any, c_type, type_name: RuntimeTypeDescriptor = None) -> Any:
    """Map from Python `value` to ctypes `c_type`.

    :param value: value to convert to c_type
    :param c_type: expected ctypes type to convert to. Untyped because there is no public superclass.
    :param type_name: optional. rust_type to check inferred rust_type of value against, and/or specify bit depth
    :return: value converted to ctypes representation
    """
    if isinstance(type_name, str):
        type_name = RuntimeType.parse(type_name)

    if isinstance(value, c_type):
        return value

    if c_type == CallbackFnPtr:
        return _wrap_py_func(value, type_name)
    
    if c_type == TransitionFn:
        return _wrap_py_transition(value, type_name)

    if c_type == ExtrinsicObjectPtr:
        # since the memory is allocated by python, 
        #    don't actually return an ExtrinsicObjectPtr, 
        #    which would call rust to free the Python-allocated ExtrinsicObject
        return ctypes.pointer(ExtrinsicObject(ctypes.py_object(value), c_counter))

    # check that the type name is consistent with the value
    if type_name is not None:
        # exit early with a null pointer if trying to load an Option type with a None value
        if isinstance(type_name, RuntimeType) and type_name.origin == "Option":
            if value is None:
                return
            type_name = type_name.args[0]

    if c_type == ctypes.c_void_p:
        if type_name is None:
            raise ValueError("type_name must be known")  # pragma: no cover

        rust_type = str(type_name)
        
        value = _check_and_cast_scalar(rust_type, value)

        if rust_type in ATOM_MAP:
            return ctypes.byref(ATOM_MAP[rust_type](value))

        if rust_type == "String":
            return ctypes.c_char_p(value.encode())

        raise UnknownTypeException(rust_type)  # pragma: no cover

    if c_type == AnyObjectPtr:
        from opendp._data import slice_as_object
        return slice_as_object(value, type_name) # type: ignore[arg-type]

    if c_type == FfiSlicePtr:
        if type_name is None:
            raise ValueError("type_name must be known")  # pragma: no cover
        return _py_to_slice(value, RuntimeType.parse(type_name))

    if isinstance(value, RuntimeType):
        value = str(value)

    if isinstance(value, str):
        value = value.encode()

    if not isinstance(value, c_type):
        # throw an error if the value is already a c_type, but the wrong one
        # (like passing a Metric into an argument expecting a Domain)
        if hasattr(value, "_type_"):
            raise ValueError(f"Cannot convert {value} to {c_type}")  # pragma: no cover
        value = c_type(value)

    return value


def c_to_py(value: Any) -> Any:
    """Map from ctypes `value` to Python value.
    It is assumed that the C type is simpler than in py_to_c, as the library returns fewer types.

    :param value: data in ctypes format
    :return: copy of data in Python representation
    """
    if isinstance(value, AnyObjectPtr):
        from opendp._data import object_type, object_as_slice, slice_free
        obj_type = object_type(value)
        
        if "PrivacyProfile" == obj_type:
            return PrivacyProfile(value)
        
        if "Queryable" in obj_type:
            from opendp.core import queryable_query_type
            query_type = RuntimeType.parse(queryable_query_type(value))
            
            if query_type == "OnceFrameQuery":
                from opendp.extras.polars import OnceFrame
                return OnceFrame(value)
            
            return Queryable(value, query_type)
        
        ffi_slice = object_as_slice(value)
        try:
            return _slice_to_py(ffi_slice, RuntimeType.parse(obj_type))
        finally:
            slice_free(ffi_slice)

    if isinstance(value, ctypes.c_char_p):
        from opendp._data import str_free
        assert value.value is not None
        value_contents = value.value.decode()
        str_free(value)
        return value_contents

    if isinstance(value, BoolPtr):
        from opendp._data import bool_free
        value_contents = value.contents.value
        bool_free(value)
        return value_contents

    if isinstance(value, ctypes.POINTER(ExtrinsicObject)):
        return value.contents.ptr

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

    if isinstance(type_name, str):
        if type_name in ATOM_MAP:
            return _slice_to_scalar(raw, type_name)
        
        if type_name == "BitVector":
            return _slice_to_bitvector(raw)
        
        if type_name == "ExtrinsicObject":
            return _slice_to_extrinsic(raw)
        
        if type_name == "String":
            return _slice_to_string(raw)
        
        if type_name == "LazyFrame":
            return _slice_to_lazyframe(raw)
        
        if type_name == "DataFrame":
            return _slice_to_dataframe(raw)
            
        if type_name == "Series":
            return _slice_to_series(raw)
        
        if type_name == "Expr":
            return _slice_to_expr(raw)
        
        if type_name == "ExprPlan":
            return _slice_to_exprplan(raw)

    if isinstance(type_name, RuntimeType):
        if type_name.origin == "Vec":
            return _slice_to_vector(raw, type_name)
        
        if type_name.origin == "Function":
            return _slice_to_function(raw)

        if type_name.origin == "HashMap":
            return _slice_to_hashmap(raw)

        if type_name.origin == "Tuple":
            return _slice_to_tuple(raw, type_name)
        
        if type_name.origin == "Option":
            return _slice_to_option(raw, type_name)

    raise UnknownTypeException(type_name)  # pragma: no cover


def _py_to_slice(value: Any, type_name: Union[RuntimeType, str]) -> FfiSlicePtr:
    """Convert from Python `value` to FfiSlicePtr.
    The initial preprocessing step for _slice_to_object that loads data into a ctypes representation.
    External checks allow this function to assume that `value` is compatible with the type_name type.

    :param value: data to load into an FfiSlice
    :param type_name: Rust type name to load value into.
    :return: pointer to an FfiSlice owned by Python.
    """
    if isinstance(type_name, str):
        if type_name in ATOM_MAP:
            return _scalar_to_slice(value, type_name)
        
        if type_name == "BitVector":
            return _bitvector_to_slice(value)
    
        if type_name == "ExtrinsicObject":
            return _extrinsic_to_slice(value)
        
        if type_name == "AnyMeasurement":
            return _wrap_in_slice(value, 1)

        if type_name == "String":
            return _string_to_slice(value)
        
        if type_name == "Series":
            return _series_to_slice(value)
        
        if type_name in {"LazyFrame", "DslPlan"}:
            return _lazyframe_to_slice(value)
        
        if type_name == "DataFrame":
            return _dataframe_to_slice(value)
            
        if type_name == "Expr":
            return _expr_to_slice(value)
    
    if isinstance(type_name, RuntimeType):
        if type_name.origin == "Vec":
            return _vector_to_slice(value, type_name)

        if type_name.origin == "HashMap":
            return _hashmap_to_slice(value, type_name)
        
        if type_name.origin == "Function":
            return _function_to_slice(value, type_name)

        if type_name.origin == "Tuple":
            return _tuple_to_slice(value, type_name)

    raise UnknownTypeException(type_name)  # pragma: no cover


def _scalar_to_slice(val, type_name: str) -> FfiSlicePtr:
    np = import_optional_dependency('numpy', raise_error=False)
    if np is not None and isinstance(val, np.ndarray):
        val = val.item()
    val = _check_and_cast_scalar(type_name, val)
    # ctypes.byref has edge-cases that cause use-after-free errors. ctypes.pointer fixes these edge-cases
    return _wrap_in_slice(ctypes.pointer(ATOM_MAP[type_name](val)), 1)


def _slice_to_scalar(raw: FfiSlicePtr, type_name: str):
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[type_name])).contents.value # type: ignore[attr-defined]


def _refcounter(ptr, increment):
    try:
        if increment:
            ctypes.pythonapi.Py_IncRef(ctypes.py_object(ptr))
        else:
            ctypes.pythonapi.Py_DecRef(ctypes.py_object(ptr))
    except Exception: # pragma: no cover
        return False
    return True

c_counter = ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.py_object, ctypes.c_bool)(_refcounter)

def _extrinsic_to_slice(val) -> FfiSlicePtr:
    return _wrap_in_slice(ctypes.pointer(ExtrinsicObject(ctypes.py_object(val), c_counter)), 1)

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
        raise ValueError("type_name must be a Vec<_>")  # pragma: no cover

    inner_type_name = type_name.args[0]
    # when input is numpy array
    # TODO: can we use the underlying buffer directly?
    np = import_optional_dependency('numpy', raise_error=False)
    if np is not None and isinstance(val, np.ndarray):
        val = val.tolist()

    if not isinstance(val, list):
        raise TypeError(f"Expected type is {type_name} but input data is not a list.")

    inner_type_name = type_name.args[0]

    if isinstance(inner_type_name, RuntimeType):
        c_repr = [py_to_c(v, c_type=AnyObjectPtr, type_name=inner_type_name) for v in val]
        array = (AnyObjectPtr * len(val))(*c_repr) # type: ignore[operator]
        ffislice = _wrap_in_slice(array, len(val))
        ffislice.depends_on(*c_repr)
        return ffislice
    
    if inner_type_name == "ExtrinsicObject":
        c_repr = [ExtrinsicObject(ctypes.py_object(v), c_counter) for v in val]
        array = (ExtrinsicObject * len(val))(*c_repr)
        ffi_slice = _wrap_in_slice(array, len(val))
        ffi_slice.depends_on(c_repr)
        return ffi_slice

    val = [_check_and_cast_scalar(inner_type_name, v) for v in val]

    if inner_type_name == "String":
        def str_to_slice(val):
            return ctypes.c_char_p(val.encode())
        array = (ctypes.c_char_p * len(val))(*map(str_to_slice, val))
        return _wrap_in_slice(array, len(val))
    
    if inner_type_name == "SeriesDomain":
        # define the ctype of an array of domains
        domain_array_type = (Domain * len(val)) # type: ignore[operator]
        # create an instance of a ctype array of domains
        array = domain_array_type(*val) # type: ignore[operator]
        return _wrap_in_slice(array, len(val))

    if inner_type_name not in ATOM_MAP:
        raise TypeError(f"Members must be one of {tuple(ATOM_MAP.keys())}. Found {inner_type_name}.")  # pragma: no cover

    array = (ATOM_MAP[inner_type_name] * len(val))(*val) # type: ignore[operator]
    return _wrap_in_slice(array, len(val))


def _slice_to_vector(raw: FfiSlicePtr, type_name: RuntimeType) -> list[Any]:
    if type_name.origin != 'Vec' or len(type_name.args) != 1:
        raise ValueError("type_name must be a Vec<_>")  # pragma: no cover
    
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

    if inner_type_name == 'ExtrinsicObject':
        array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ExtrinsicObject))[0:raw.contents.len]
        return list(map(lambda v: v.ptr, array))

    if inner_type_name == 'String':
        array = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_char_p))[0:raw.contents.len]
        return list(map(lambda v: v.decode(), array))

    assert isinstance(inner_type_name, str)
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(ATOM_MAP[inner_type_name]))[0:raw.contents.len]


def _tuple_to_slice(val: tuple[Any, ...], type_name: Union[RuntimeType, str]) -> FfiSlicePtr:
    type_name = cast(RuntimeType, type_name)
    inner_type_names = type_name.args
    if not isinstance(val, tuple):
        raise TypeError("Cannot coerce a non-tuple type to a tuple")  # pragma: no cover

    if len(inner_type_names) != len(val):
        raise TypeError("type_name members must have same length as tuple")  # pragma: no cover

    if inner_type_names == ['f64', 'ExtrinsicObject']:
        score_ptr = ctypes.pointer(ctypes.c_double(val[0]))
        ext_obj = ctypes.pointer(ExtrinsicObject(ctypes.py_object(val[1]), c_counter))

        cand_ptr = py_to_c(ext_obj, c_type=AnyObjectPtr, type_name="ExtrinsicObject")
        array = (ctypes.c_void_p * 2)(
            ctypes.cast(score_ptr, ctypes.c_void_p), 
            ctypes.cast(cand_ptr, ctypes.c_void_p), 
        )
        return _wrap_in_slice(ctypes.pointer(array), 2)

    for t in inner_type_names:
        if t not in ATOM_MAP:
            raise TypeError(f"Tuple members must be one of {ATOM_MAP.keys()}. Got {t}")  # pragma: no cover

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
    # typed pointer
    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    # list of void*
    ptr_data = void_array_ptr[0:raw.contents.len]

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
    
    keys: AnyObjectPtr = py_to_c(list(val.keys()), type_name=Vec[key_type], c_type=AnyObjectPtr)
    vals: AnyObjectPtr = py_to_c(list(val.values()), type_name=Vec[val_type], c_type=AnyObjectPtr)
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
    # typed pointer
    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    # list of void*
    ptr_data = void_array_ptr[0:raw.contents.len]

    plan = _slice_to_lazyframe(ctypes.cast(ptr_data[0], FfiSlicePtr))
    expr = _slice_to_expr(ctypes.cast(ptr_data[1], FfiSlicePtr))
    fill = _slice_to_expr(ctypes.cast(ptr_data[2], FfiSlicePtr)) if raw.contents.len == 3 else None

    from collections import namedtuple

    ExprPlan = namedtuple("ExprPlan", ["plan", "expr", "fill"])
    return ExprPlan(plan, expr, fill)

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


def _wrap_in_slice(ptr, len_: int) -> FfiSlicePtr:
    return FfiSlicePtr(FfiSlice(ctypes.cast(ptr, ctypes.c_void_p), len_))


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
                ctypes.c_char_p("Continued stack trace from Exception in user-defined function".encode()),
                ctypes.c_char_p(traceback.format_exc().encode()),
            )

    c_wrapper_func = CallbackFnValue(wrapper_func)
    lifeline = ExtrinsicObject(ctypes.py_object(c_wrapper_func), c_counter)

    return ctypes.pointer(CallbackFn(c_wrapper_func, lifeline))


# The output type cannot be an `ctypes.POINTER(FfiResult)` due to:
#   https://bugs.python.org/issue5710#msg85731
#                              (answer         , query       , is_internal  )
TransitionFn = ctypes.CFUNCTYPE(ctypes.c_void_p, AnyObjectPtr, ctypes.c_bool)

def _wrap_py_transition(py_transition, A):
    from opendp._convert import c_to_py, py_to_c

    # the indicator that a query is internal is oftentimes not needed
    if len(signature(py_transition).parameters) == 1:
        py_transition_old = py_transition
        py_transition = lambda q, _=None: py_transition_old(q)

    def wrapper_func(c_query, c_is_internal: ctypes.c_bool):
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

    return TransitionFn(wrapper_func)
