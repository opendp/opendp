# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "bool_free",
    "ffislice_of_anyobjectptrs",
    "new_arrow_array",
    "object_as_slice",
    "object_free",
    "object_type",
    "slice_as_object",
    "slice_free",
    "smd_curve_epsilon",
    "str_free",
    "to_string"
]


@versioned
def bool_free(
    this
):
    """Internal function. Free the memory associated with `this`, a bool.
    Used to clean up after the relation check.
    
    [bool_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.bool_free.html)
    
    :param this: 
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this
    
    # Call library function.
    lib_function = lib.opendp_data__bool_free
    lib_function.argtypes = [BoolPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))
    
    return output


@versioned
def ffislice_of_anyobjectptrs(
    raw: Any
) -> Any:
    """Internal function. Converts an FfiSlice of AnyObjects to an FfiSlice of AnyObjectPtrs.
    
    [ffislice_of_anyobjectptrs in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.ffislice_of_anyobjectptrs.html)
    
    :param raw: 
    :type raw: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_raw = py_to_c(raw, c_type=FfiSlicePtr, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_data__ffislice_of_anyobjectptrs
    lib_function.argtypes = [FfiSlicePtr]
    lib_function.restype = FfiResult
    
    output = unwrap(lib_function(c_raw), FfiSlicePtr)
    
    return output


@versioned
def new_arrow_array(
    name: str
) -> Any:
    """[new_arrow_array in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.new_arrow_array.html)
    
    :param name: 
    :type name: str
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_name = py_to_c(name, c_type=ctypes.c_char_p, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_data__new_arrow_array
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_name), FfiSlicePtr))
    
    return output


@versioned
def object_as_slice(
    obj: Any
) -> Any:
    """Internal function. Unload data from an AnyObject into an FfiSlicePtr.
    
    [object_as_slice in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.object_as_slice.html)
    
    :param obj: 
    :type obj: Any
    :return: An FfiSlice that contains the data in FfiObject, but in a format readable in bindings languages.
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_obj = py_to_c(obj, c_type=AnyObjectPtr, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_data__object_as_slice
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = unwrap(lib_function(c_obj), FfiSlicePtr)
    
    return output


@versioned
def object_free(
    this: Any
):
    """Internal function. Free the memory associated with `this`, an AnyObject.
    
    [object_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.object_free.html)
    
    :param this: 
    :type this: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this
    
    # Call library function.
    lib_function = lib.opendp_data__object_free
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))
    
    return output


@versioned
def object_type(
    this: Any
) -> str:
    """Internal function. Retrieve the type descriptor string of an AnyObject.
    
    [object_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.object_type.html)
    
    :param this: 
    :type this: Any
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=AnyObjectPtr, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_data__object_type
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def slice_as_object(
    raw: FfiSlicePtr,
    T: str
) -> Any:
    """Internal function. Load data from a `slice` into an AnyObject
    
    [slice_as_object in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.slice_as_object.html)
    
    :param raw: 
    :type raw: FfiSlicePtr
    :param T: 
    :type T: str
    :return: An AnyObject that contains the data in `slice`.
    The AnyObject also captures rust type information.
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = parse_or_infer(T, raw)
    
    # Convert arguments to c types.
    c_raw = py_to_c(raw, c_type=FfiSlicePtr, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_data__slice_as_object
    lib_function.argtypes = [FfiSlicePtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = unwrap(lib_function(c_raw, c_T), AnyObjectPtr)
    
    return output


@versioned
def slice_free(
    this: Any
):
    """Internal function. Free the memory associated with `this`, an FfiSlicePtr.
    Used to clean up after object_as_slice.
    Frees the slice, but not what the slice references!
    
    [slice_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.slice_free.html)
    
    :param this: 
    :type this: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this
    
    # Call library function.
    lib_function = lib.opendp_data__slice_free
    lib_function.argtypes = [FfiSlicePtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))
    
    return output


@versioned
def smd_curve_epsilon(
    curve: Any,
    delta: Any
) -> Any:
    """Internal function. Use an SMDCurve to find epsilon at a given `delta`.
    
    [smd_curve_epsilon in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.smd_curve_epsilon.html)
    
    :param curve: 
    :type curve: Any
    :param delta: 
    :type delta: Any
    :return: Epsilon at a given `delta`.
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_curve = py_to_c(curve, c_type=AnyObjectPtr, type_name=None)
    c_delta = py_to_c(delta, c_type=AnyObjectPtr, type_name=get_atom(object_type(curve)))
    
    # Call library function.
    lib_function = lib.opendp_data__smd_curve_epsilon
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_curve, c_delta), AnyObjectPtr))
    
    return output


@versioned
def str_free(
    this: str
):
    """Internal function. Free the memory associated with `this`, a string.
    Used to clean up after the type getter functions.
    
    [str_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.str_free.html)
    
    :param this: 
    :type this: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this
    
    # Call library function.
    lib_function = lib.opendp_data__str_free
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))
    
    return output


@versioned
def to_string(
    this: Any
) -> str:
    """Internal function. Convert the AnyObject to a string representation.
    
    [to_string in Rust documentation.](https://docs.rs/opendp/latest/opendp/data/fn.to_string.html)
    
    :param this: 
    :type this: Any
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=AnyObjectPtr, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_data__to_string
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output
