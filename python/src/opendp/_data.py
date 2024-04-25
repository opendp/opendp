# Auto-generated. Do not edit!
'''
TODO!
'''
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "arrow_array_free",
    "bool_free",
    "extrinsic_object_free",
    "ffislice_of_anyobjectptrs",
    "fill_bytes",
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


def arrow_array_free(
    this
):
    r"""Internal function. Free the memory associated with `this`, a slice containing an Arrow array, schema, and name.

    :param this: 
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this

    # Call library function.
    lib_function = lib.opendp_data__arrow_array_free
    lib_function.argtypes = [ctypes.c_void_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))

    return output


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


def ffislice_of_anyobjectptrs(
    raw
):
    r"""Internal function. Converts an FfiSlice of AnyObjects to an FfiSlice of AnyObjectPtrs.

    :param raw: A pointer to the slice to free.
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


def new_arrow_array(
    name: str
):
    r"""Allocate an empty ArrowArray and ArrowSchema that Rust owns the memory for.
    The ArrowArray and ArrowSchema are initialized empty, and are populated by the bindings language.

    :param name: The name of the ArrowArray. A clone of this string owned by Rust will be returned in the slice.
    :type name: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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


def object_as_slice(
    obj
):
    r"""Internal function. Unload data from an AnyObject into an FfiSlicePtr.

    :param obj: A pointer to the AnyObject to unpack.
    :return: An FfiSlice that contains the data in FfiObject, but in a format readable in bindings languages.
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


def object_free(
    this
):
    r"""Internal function. Free the memory associated with `this`, an AnyObject.

    :param this: A pointer to the AnyObject to free.
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


def object_type(
    this
) -> str:
    r"""Internal function. Retrieve the type descriptor string of an AnyObject.

    :param this: A pointer to the AnyObject.
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


def slice_as_object(
    raw: FfiSlicePtr,
    T: str
):
    r"""Internal function. Load data from a `slice` into an AnyObject

    :param raw: A pointer to the slice with data.
    :type raw: FfiSlicePtr
    :param T: The type of the data in the slice.
    :type T: str
    :return: An AnyObject that contains the data in `slice`.
    The AnyObject also captures rust type information.
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


def slice_free(
    this
):
    r"""Internal function. Free the memory associated with `this`, an FfiSlicePtr.
    Used to clean up after object_as_slice.
    Frees the slice, but not what the slice references!

    :param this: A pointer to the FfiSlice to free.
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


def smd_curve_epsilon(
    curve,
    delta
):
    r"""Internal function. Use an SMDCurve to find epsilon at a given `delta`.

    :param curve: The SMDCurve.
    :param delta: What to fix delta to compute epsilon.
    :return: Epsilon at a given `delta`.
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


def to_string(
    this
) -> str:
    r"""Internal function. Convert the AnyObject to a string representation.

    :param this: The AnyObject to convert to a string representation.
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
