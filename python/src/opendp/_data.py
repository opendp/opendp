# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "to_string",
    "slice_as_object",
    "object_type",
    "object_as_slice",
    "ffislice_of_anyobjectptrs",
    "object_free",
    "slice_free",
    "str_free",
    "bool_free",
    "smd_curve_epsilon"
]


def to_string(
    this: Any
) -> str:
    """Internal function. Convert the AnyObject to a string representation.
    
    :param this: 
    :type this: Any
    :return: String representation of `this` AnyObject.
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=AnyObjectPtr)
    
    # Call library function.
    function = lib.opendp_data__to_string
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def slice_as_object(
    slice: FfiSlicePtr,
    T: RuntimeTypeDescriptor = None
) -> Any:
    """Internal function. Load data from a `slice` into an AnyObject
    
    :param slice: 
    :type slice: FfiSlicePtr
    :param T: 
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: An AnyObject that contains the data in `slice`. The AnyObject also captures rust type information.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=slice)
    
    # Convert arguments to c types.
    slice = py_to_c(slice, c_type=FfiSlicePtr, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_data__slice_as_object
    function.argtypes = [FfiSlicePtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(slice, T), AnyObjectPtr)


def object_type(
    this: Any
) -> str:
    """Internal function. Retrieve the type descriptor string of an AnyObject.
    
    :param this: 
    :type this: Any
    :return: The rust type associated with `this` AnyObject.
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=AnyObjectPtr)
    
    # Call library function.
    function = lib.opendp_data__object_type
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def object_as_slice(
    this: Any
) -> Any:
    """Internal function. Unload data from an AnyObject into an FfiSlicePtr.
    
    :param this: 
    :type this: Any
    :return: An FfiSlice that contains the data in FfiObject, but in a format readable in bindings languages.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=AnyObjectPtr)
    
    # Call library function.
    function = lib.opendp_data__object_as_slice
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return unwrap(function(this), FfiSlicePtr)


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
    raw = py_to_c(raw, c_type=FfiSlicePtr)
    
    # Call library function.
    function = lib.opendp_data__ffislice_of_anyobjectptrs
    function.argtypes = [FfiSlicePtr]
    function.restype = FfiResult
    
    return unwrap(function(raw), FfiSlicePtr)


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


def slice_free(
    this: Any
):
    """Internal function. Free the memory associated with `this`, an FfiSlicePtr. 
    Used to clean up after object_as_slice.
    
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
    curve = py_to_c(curve, c_type=AnyObjectPtr)
    delta = py_to_c(delta, c_type=AnyObjectPtr, type_name=get_atom(object_type(curve)))
    
    # Call library function.
    function = lib.opendp_data__smd_curve_epsilon
    function.argtypes = [AnyObjectPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(curve, delta), AnyObjectPtr))
