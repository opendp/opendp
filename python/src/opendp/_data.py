# Auto-generated. Do not edit!
'''
TODO!
'''
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "bool_free",
    "extrinsic_object_free",
    "ffislice_of_anyobjectptrs",
    "fill_bytes",
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
    r"""Internal function. Free the memory associated with `this`, a bool.
    Used to clean up after the relation check.

    :param this: A pointer to the bool to free.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
def extrinsic_object_free(
    this
):
    r"""Internal function. Free the memory associated with `this`, a string.
    Used to clean up after the type getter functions.

    :param this: 
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this

    # Call library function.
    lib_function = lib.opendp_data__extrinsic_object_free
    lib_function.argtypes = [ExtrinsicObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))

    return output


@versioned
def ffislice_of_anyobjectptrs(
    raw: Any
) -> Any:
    r"""Internal function. Converts an FfiSlice of AnyObjects to an FfiSlice of AnyObjectPtrs.

    :param raw: A pointer to the slice to free.
    :type raw: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
def fill_bytes(
    ptr,
    len: int
) -> bool:
    r"""Internal function. Populate the buffer behind `ptr` with `len` random bytes
    sampled from a cryptographically secure RNG.

    :param ptr: 
    :param len: 
    :type len: int
    :rtype: bool
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_ptr = ptr
    c_len = py_to_c(len, c_type=ctypes.c_size_t, type_name=usize)

    # Call library function.
    lib_function = lib.opendp_data__fill_bytes
    lib_function.argtypes = [ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
    lib_function.restype = ctypes.c_bool

    output = c_to_py(lib_function(c_ptr, c_len))

    return output


@versioned
def object_as_slice(
    obj: Any
) -> Any:
    r"""Internal function. Unload data from an AnyObject into an FfiSlicePtr.

    :param obj: A pointer to the AnyObject to unpack.
    :type obj: Any
    :return: An FfiSlice that contains the data in FfiObject, but in a format readable in bindings languages.
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    this
):
    r"""Internal function. Free the memory associated with `this`, an AnyObject.

    :param this: A pointer to the AnyObject to free.
    :type this: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    r"""Internal function. Retrieve the type descriptor string of an AnyObject.

    :param this: A pointer to the AnyObject.
    :type this: Any
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    r"""Internal function. Load data from a `slice` into an AnyObject

    :param raw: A pointer to the slice with data.
    :type raw: FfiSlicePtr
    :param T: The type of the data in the slice.
    :type T: str
    :return: An AnyObject that contains the data in `slice`.
    The AnyObject also captures rust type information.
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = parse_or_infer(T, raw) # type: ignore

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
    this
):
    r"""Internal function. Free the memory associated with `this`, an FfiSlicePtr.
    Used to clean up after object_as_slice.
    Frees the slice, but not what the slice references!

    :param this: A pointer to the FfiSlice to free.
    :type this: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    r"""Internal function. Use an SMDCurve to find epsilon at a given `delta`.

    :param curve: The SMDCurve.
    :type curve: Any
    :param delta: What to fix delta to compute epsilon.
    :type delta: Any
    :return: Epsilon at a given `delta`.
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    this
):
    r"""Internal function. Free the memory associated with `this`, a string.
    Used to clean up after the type getter functions.

    :param this: A pointer to the string to free.
    :type this: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    r"""Internal function. Convert the AnyObject to a string representation.

    :param this: The AnyObject to convert to a string representation.
    :type this: Any
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
