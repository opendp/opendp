# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "fixed_smoothed_max_divergence",
    "max_divergence",
    "measure_debug",
    "measure_distance_type",
    "measure_type",
    "smoothed_max_divergence",
    "zero_concentrated_divergence"
]


def fixed_smoothed_max_divergence(
    T: RuntimeTypeDescriptor
):
    """Construct an instance of the `FixedSmoothedMaxDivergence` measure.
    
    [fixed_smoothed_max_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.fixed_smoothed_max_divergence.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measures__fixed_smoothed_max_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Measure))
    
    return output


def max_divergence(
    T: RuntimeTypeDescriptor
):
    """Construct an instance of the `MaxDivergence` measure.
    
    [max_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.max_divergence.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measures__max_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Measure))
    
    return output


def measure_debug(
    this
) -> str:
    """Debug a `measure`.
    
    [measure_debug in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.measure_debug.html)
    
    :param this: The measure to debug (stringify).
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measure, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_measures__measure_debug
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


def measure_distance_type(
    this
) -> str:
    """Get the distance type of a `measure`.
    
    [measure_distance_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.measure_distance_type.html)
    
    :param this: The measure to retrieve the distance type from.
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measure, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_measures__measure_distance_type
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


def measure_type(
    this
) -> str:
    """Get the type of a `measure`.
    
    [measure_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.measure_type.html)
    
    :param this: The measure to retrieve the type from.
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measure, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_measures__measure_type
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


def smoothed_max_divergence(
    T: RuntimeTypeDescriptor
):
    """Construct an instance of the `SmoothedMaxDivergence` measure.
    
    [smoothed_max_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.smoothed_max_divergence.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measures__smoothed_max_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Measure))
    
    return output


def zero_concentrated_divergence(
    T: RuntimeTypeDescriptor
):
    """Construct an instance of the `ZeroConcentratedDivergence` measure.
    
    [zero_concentrated_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.zero_concentrated_divergence.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measures__zero_concentrated_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Measure))
    
    return output
