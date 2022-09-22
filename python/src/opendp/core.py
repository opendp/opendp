# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "_error_free",
    "_measurement_free",
    "_transformation_free",
    "measurement_check",
    "measurement_input_carrier_type",
    "measurement_input_distance_type",
    "measurement_invoke",
    "measurement_map",
    "measurement_output_distance_type",
    "transformation_check",
    "transformation_input_carrier_type",
    "transformation_input_distance_type",
    "transformation_invoke",
    "transformation_map",
    "transformation_output_distance_type"
]


def _error_free(
    this: FfiError
) -> bool:
    """Internal function. Free the memory associated with `error`.
    
    :param this: 
    :type this: FfiError
    :return: A boolean, where true indicates successful free
    :rtype: bool
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    function = lib.opendp_core___error_free
    function.argtypes = [ctypes.POINTER(FfiError)]
    function.restype = ctypes.c_bool
    
    return c_to_py(function(this))


def _measurement_free(
    this: Measurement
):
    """Internal function. Free the memory associated with `this`.
    
    :param this: 
    :type this: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    function = lib.opendp_core___measurement_free
    function.argtypes = [Measurement]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def _transformation_free(
    this: Transformation
):
    """Internal function. Free the memory associated with `this`.
    
    :param this: 
    :type this: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    function = lib.opendp_core___transformation_free
    function.argtypes = [Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def measurement_check(
    measurement: Measurement,
    distance_in: Any,
    distance_out: Any
):
    """Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
    
    :param measurement: Measurement to check the privacy relation of.
    :type measurement: Measurement
    :param distance_in: 
    :type distance_in: Any
    :param distance_out: 
    :type distance_out: Any
    :return: True indicates that the relation passed at the given distance.
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement, type_name=None)
    distance_in = py_to_c(distance_in, c_type=AnyObjectPtr, type_name=measurement_input_distance_type(measurement))
    distance_out = py_to_c(distance_out, c_type=AnyObjectPtr, type_name=measurement_output_distance_type(measurement))
    
    # Call library function.
    function = lib.opendp_core__measurement_check
    function.argtypes = [Measurement, AnyObjectPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, distance_in, distance_out), BoolPtr))


def measurement_input_carrier_type(
    this: Measurement
) -> str:
    """Get the input (carrier) data type of `this`.
    
    :param this: The measurement to retrieve the type from.
    :type this: Measurement
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=Measurement, type_name=None)
    
    # Call library function.
    function = lib.opendp_core__measurement_input_carrier_type
    function.argtypes = [Measurement]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def measurement_input_distance_type(
    this: Measurement
) -> str:
    """Get the input distance type of `measurement`.
    
    :param this: The measurement to retrieve the type from.
    :type this: Measurement
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=Measurement, type_name=None)
    
    # Call library function.
    function = lib.opendp_core__measurement_input_distance_type
    function.argtypes = [Measurement]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def measurement_invoke(
    this: Measurement,
    arg: Any
) -> Any:
    """Invoke the `measurement` with `arg`. Returns a differentially private release.
    
    :param this: Measurement to invoke.
    :type this: Measurement
    :param arg: Input data to supply to the measurement. A member of the measurement's input domain.
    :type arg: Any
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=Measurement, type_name=None)
    arg = py_to_c(arg, c_type=AnyObjectPtr, type_name=measurement_input_carrier_type(this))
    
    # Call library function.
    function = lib.opendp_core__measurement_invoke
    function.argtypes = [Measurement, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this, arg), AnyObjectPtr))


def measurement_map(
    measurement: Measurement,
    distance_in: Any
) -> Any:
    """Use the `measurement` to map a given `d_in` to `d_out`.
    
    :param measurement: Measurement to check the map distances with.
    :type measurement: Measurement
    :param distance_in: Distance in terms of the input metric.
    :type distance_in: Any
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement, type_name=None)
    distance_in = py_to_c(distance_in, c_type=AnyObjectPtr, type_name=measurement_input_distance_type(measurement))
    
    # Call library function.
    function = lib.opendp_core__measurement_map
    function.argtypes = [Measurement, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, distance_in), AnyObjectPtr))


def measurement_output_distance_type(
    this: Measurement
) -> str:
    """Get the output distance type of `measurement`.
    
    :param this: The measurement to retrieve the type from.
    :type this: Measurement
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=Measurement, type_name=None)
    
    # Call library function.
    function = lib.opendp_core__measurement_output_distance_type
    function.argtypes = [Measurement]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def transformation_check(
    transformation: Transformation,
    distance_in: Any,
    distance_out: Any
):
    """Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
    
    :param transformation: 
    :type transformation: Transformation
    :param distance_in: 
    :type distance_in: Any
    :param distance_out: 
    :type distance_out: Any
    :return: True indicates that the relation passed at the given distance.
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation = py_to_c(transformation, c_type=Transformation, type_name=None)
    distance_in = py_to_c(distance_in, c_type=AnyObjectPtr, type_name=transformation_input_distance_type(transformation))
    distance_out = py_to_c(distance_out, c_type=AnyObjectPtr, type_name=transformation_output_distance_type(transformation))
    
    # Call library function.
    function = lib.opendp_core__transformation_check
    function.argtypes = [Transformation, AnyObjectPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation, distance_in, distance_out), BoolPtr))


def transformation_input_carrier_type(
    this: Transformation
) -> str:
    """Get the input (carrier) data type of `this`.
    
    :param this: The transformation to retrieve the type from.
    :type this: Transformation
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=Transformation, type_name=None)
    
    # Call library function.
    function = lib.opendp_core__transformation_input_carrier_type
    function.argtypes = [Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def transformation_input_distance_type(
    this: Transformation
) -> str:
    """Get the input distance type of `transformation`.
    
    :param this: The transformation to retrieve the type from.
    :type this: Transformation
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=Transformation, type_name=None)
    
    # Call library function.
    function = lib.opendp_core__transformation_input_distance_type
    function.argtypes = [Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def transformation_invoke(
    this: Transformation,
    arg: Any
) -> Any:
    """Invoke the `transformation` with `arg`. Returns a differentially private release.
    
    :param this: Transformation to invoke.
    :type this: Transformation
    :param arg: Input data to supply to the transformation. A member of the transformation's input domain.
    :type arg: Any
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=Transformation, type_name=None)
    arg = py_to_c(arg, c_type=AnyObjectPtr, type_name=transformation_input_carrier_type(this))
    
    # Call library function.
    function = lib.opendp_core__transformation_invoke
    function.argtypes = [Transformation, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this, arg), AnyObjectPtr))


def transformation_map(
    transformation: Transformation,
    distance_in: Any
) -> Any:
    """Use the `transformation` to map a given `d_in` to `d_out`.
    
    :param transformation: Transformation to check the map distances with.
    :type transformation: Transformation
    :param distance_in: Distance in terms of the input metric.
    :type distance_in: Any
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation = py_to_c(transformation, c_type=Transformation, type_name=None)
    distance_in = py_to_c(distance_in, c_type=AnyObjectPtr, type_name=transformation_input_distance_type(transformation))
    
    # Call library function.
    function = lib.opendp_core__transformation_map
    function.argtypes = [Transformation, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation, distance_in), AnyObjectPtr))


def transformation_output_distance_type(
    this: Transformation
) -> str:
    """Get the output distance type of `transformation`.
    
    :param this: The transformation to retrieve the type from.
    :type this: Transformation
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    this = py_to_c(this, c_type=Transformation, type_name=None)
    
    # Call library function.
    function = lib.opendp_core__transformation_output_distance_type
    function.argtypes = [Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))
