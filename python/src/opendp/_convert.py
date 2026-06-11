from typing import Any, Union
from inspect import signature

from opendp._lib import *
from opendp.mod import (
    UnknownTypeException,
    Domain,
    Measure,
    Metric,
    PrivacyProfile,
    Queryable,
    OdometerQueryable,
)
from opendp.typing import RuntimeType, RuntimeTypeDescriptor
from opendp._convert_maps import (
    ATOM_MAP,
    check_c_int_cast,
    _scalar_to_slice,
    _slice_to_scalar,
    _TO_SLICE_FROM_STR,
    _TO_SLICE_FROM_ORIGIN,
    _FROM_SLICE_STR,
    _FROM_SLICE_RT_TYPE,
    _DOMAIN_CLASS_FROM_RT_TYPE,
    _DISTANCE_CLASS_FROM_RT_TYPE,
    _MEASURE_CLASS_FROM_RT_TYPE,
    INT_SIZES,
)

_ERROR_URL_298 = "https://github.com/opendp/opendp/discussions/298"


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

    if c_type == TransitionFnPtr:
        return _wrap_py_transition(value, type_name)

    if c_type == ExtrinsicObjectPtr:
        # since the memory is allocated by python,
        #    don't actually return an ExtrinsicObjectPtr,
        #    which would call rust to free the Python-allocated ExtrinsicObject
        return ctypes.pointer(ExtrinsicObject(ctypes.py_object(value)))

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
        if isinstance(value, ctypes.POINTER(AnyObject)):
            return value

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
    if isinstance(value, ctypes.POINTER(AnyObject)):
        from opendp._data import object_type, object_as_slice, slice_free

        obj_type = object_type(value)

        if obj_type == PrivacyProfile.__name__:
            return PrivacyProfile(value)

        if obj_type == "AnyOdometerQueryable":
            return OdometerQueryable(value)

        if obj_type == "AnyQueryable":
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

    if isinstance(value, Domain):
        from opendp.domains import domain_type
        rt_type = RuntimeType.parse(domain_type(value))

        if isinstance(rt_type, RuntimeType):
            rt_type = rt_type.origin

        if rt_type in _DOMAIN_CLASS_FROM_RT_TYPE:
            value.__class__ = _DOMAIN_CLASS_FROM_RT_TYPE[rt_type]

        # if you fall through these cases, then it is just treated as a generic Domain

    if isinstance(value, Metric):
        from opendp.metrics import metric_type

        rt_type = RuntimeType.parse(metric_type(value))

        if isinstance(rt_type, RuntimeType):
            rt_type = rt_type.origin

        if rt_type in _DISTANCE_CLASS_FROM_RT_TYPE:
            value.__class__ = _DISTANCE_FROM_RT_TYPE

        # if you fall through these cases, then it is just treated as a generic Metric

    if isinstance(value, Measure):
        from opendp.measures import measure_type

        rt_type = RuntimeType.parse(measure_type(value))

        if isinstance(rt_type, RuntimeType):
            rt_type = rt_type.origin

        if rt_type in _MEASURE_CLASS_FROM_RT_TYPE:
            value.__class__ = _MEASURE_CLASS_FROM_RT_TYPE

        # if you fall through these cases, then it is just treated as a generic Measure

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

        if type_name in _FROM_SLICE_STR:
            return _FROM_SLICE_STR[type_name](raw)

    elif isinstance(type_name, RuntimeType):
        if type_name.origin in _FROM_SLICE_RT_TYPE:
            return _FROM_SLICE_RT_TYPE[type_name.origin](raw, type_name)

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

        if type_name in _TO_SLICE_FROM_STR:
            return _TO_SLICE_FROM_STR[type_name](value)

    if isinstance(type_name, RuntimeType):
        if type_name.origin in _TO_SLICE_FROM_ORIGIN:
            return _TO_SLICE_FROM_ORIGIN[type_name.origin](value, type_name)

    raise UnknownTypeException(type_name)  # pragma: no cover

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


def _wrap_py_transition(py_transition, A):
    # the indicator that a query is internal is oftentimes not needed
    if len(signature(py_transition).parameters) == 1:
        py_transition_old = py_transition
        py_transition = lambda q, _=None: py_transition_old(q)

    userdata = ExtrinsicObject(ctypes.py_object((py_transition, A)))
    return ctypes.pointer(TransitionFn(_TRANSITION_DISPATCH, userdata))
