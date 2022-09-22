# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "bool_free",
    "ffislice_of_anyobjectptrs",
    "object_as_slice",
    "object_free",
    "object_type",
    "slice_as_object",
    "slice_free",
    "smd_curve_epsilon",
    "str_free",
    "to_string"
]


def bool_free(
    this
):
    """Internal function. Free the memory associated with `this`, a bool. 
    Used to clean up after the relation check.
    
    :param this: 
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    function = lib.opendp_data__bool_free
    function.argtypes = [BoolPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def ffislice_of_anyobjectptrs(
    raw: Any
) -> Any:
    """Internal function. Converts an FfiSlice of AnyObjects to an FfiSlice of AnyObjectPtrs.
    
    :param raw: 
    :type raw: Any
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    raw = py_to_c(raw, c_type=FfiSlicePtr, type_name=None)
    
    # Call library function.
    function = lib.opendp_data__ffislice_of_anyobjectptrs
    function.argtypes = [FfiSlicePtr]
    function.restype = FfiResult
    
    return unwrap(function(raw), FfiSlicePtr)


def object_as_slice(
    obj: Any
) -> Any:
    """Internal function. Unload data from an AnyObject into an FfiSlicePtr.
    
    :param obj: 
    :type obj: Any
    :return: An FfiSlice that contains the data in FfiObject, but in a format readable in bindings languages.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    obj = py_to_c(obj, c_type=AnyObjectPtr, type_name=None)
    
    # Call library function.
    function = lib.opendp_data__object_as_slice
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return unwrap(function(obj), FfiSlicePtr)


def object_free(
    this: Any
):
    """Internal function. Free the memory associated with `this`, an AnyObject.
    
    :param this: 
    :type this: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    function = lib.opendp_data__object_free
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def object_type(
    this: Any
) -> str:
    """Internal function. Retrieve the type descriptor string of an AnyObject.
    
    :param this: 
    :type this: Any
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=AnyObjectPtr, type_name=None)
    
    # Call library function.
    function = lib.opendp_data__object_type
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def slice_as_object(
    raw: FfiSlicePtr,
    T: str
) -> Any:
    """Internal function. Load data from a `slice` into an AnyObject
    
    :param raw: 
    :type raw: FfiSlicePtr
    :param T: 
    :type T: str
    :return: An AnyObject that contains the data in `slice`. 
    The AnyObject also captures rust type information.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = parse_or_infer(T, raw)
    
    # Convert arguments to c types.
    raw = py_to_c(raw, c_type=FfiSlicePtr, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p, type_name=None)
    
    # Call library function.
    function = lib.opendp_data__slice_as_object
    function.argtypes = [FfiSlicePtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(raw, T), AnyObjectPtr)


def slice_free(
    this: Any
):
    """Internal function. Free the memory associated with `this`, an FfiSlicePtr. 
    Used to clean up after object_as_slice.
    Frees the slice, but not what the slice references!
    
    :param this: 
    :type this: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    function = lib.opendp_data__slice_free
    function.argtypes = [FfiSlicePtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def smd_curve_epsilon(
    curve: Any,
    delta: Any
) -> Any:
    """Internal function. Use an SMDCurve to find epsilon at a given `delta`.
    
    :param curve: 
    :type curve: Any
    :param delta: 
    :type delta: Any
    :return: Epsilon at a given `delta`.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    curve = py_to_c(curve, c_type=AnyObjectPtr, type_name=None)
    delta = py_to_c(delta, c_type=AnyObjectPtr, type_name=get_atom(object_type(curve)))
    
    # Call library function.
    function = lib.opendp_data__smd_curve_epsilon
    function.argtypes = [AnyObjectPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(curve, delta), AnyObjectPtr))


def str_free(
    this: str
):
    """Internal function. Free the memory associated with `this`, a string. 
    Used to clean up after the type getter functions.
    
    :param this: 
    :type this: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    function = lib.opendp_data__str_free
    function.argtypes = [ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def to_string(
    this: Any
) -> str:
    """Internal function. Convert the AnyObject to a string representation.
    
    :param this: 
    :type this: Any
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=AnyObjectPtr, type_name=None)
    
    # Call library function.
    function = lib.opendp_data__to_string
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))
