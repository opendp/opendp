# Auto-generated. Do not edit.
from opendp.v1._convert import *
from opendp.v1._lib import *
from opendp.v1.mod import *
from opendp.v1.typing import *


def _to_string(
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
    function = lib.opendp_data___to_string
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def _slice_as_object(
    slice: FfiSlicePtr,
    T: RuntimeTypeDescriptor = None
) -> Any:
    """Internal function. Load data from a `slice` into an AnyObject
    
    :param slice: 
    :type slice: FfiSlicePtr
    :param T: 
    :type T: RuntimeTypeDescriptor
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
    function = lib.opendp_data___slice_as_object
    function.argtypes = [FfiSlicePtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(slice, T), AnyObjectPtr)


def _slice_as_metric_distance(
    slice: FfiSlicePtr,
    T: RuntimeTypeDescriptor = None
):
    """Internal function. Load data from a `slice` into an AnyMetricDistance
    
    :param slice: 
    :type slice: FfiSlicePtr
    :param T: 
    :type T: RuntimeTypeDescriptor
    :return: An AnyMetricDistance that contains the data in `slice`. The AnyMetricDistance also captures rust type information.
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
    function = lib.opendp_data___slice_as_metric_distance
    function.argtypes = [FfiSlicePtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(slice, T), AnyMetricDistancePtr)


def _slice_as_measure_distance(
    slice: FfiSlicePtr,
    T: RuntimeTypeDescriptor = None
):
    """Internal function. Load data from a `slice` into an AnyMeasureDistance
    
    :param slice: 
    :type slice: FfiSlicePtr
    :param T: 
    :type T: RuntimeTypeDescriptor
    :return: An AnyMeasureDistance that contains the data in `slice`. The AnyMeasureDistance also captures rust type information.
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
    function = lib.opendp_data___slice_as_measure_distance
    function.argtypes = [FfiSlicePtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(slice, T), AnyMeasureDistancePtr)


def _object_type(
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
    function = lib.opendp_data___object_type
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def _object_as_slice(
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
    function = lib.opendp_data___object_as_slice
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return unwrap(function(this), FfiSlicePtr)


def _object_free(
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
    function = lib.opendp_data___object_free
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def _slice_free(
    this: Any
):
    """Internal function. Free the memory associated with `this`, an FfiSlicePtr. 
    Used to clean up after _object_as_slice.
    
    :param this: 
    :type this: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    function = lib.opendp_data___slice_free
    function.argtypes = [FfiSlicePtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def _str_free(
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
    function = lib.opendp_data___str_free
    function.argtypes = [ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def _bool_free(
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
    function = lib.opendp_data___bool_free
    function.argtypes = [BoolPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))
