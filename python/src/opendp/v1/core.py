# Auto-generated. Do not edit.
from opendp.v1._convert import *
from opendp.v1._lib import *
from opendp.v1.mod import *
from opendp.v1.typing import *


def _error_free(
    error: FfiError
) -> bool:
    """Internal function. Free the memory associated with `error`.
    
    :param error: 
    :type error: FfiError
    :return: true indicates successful free
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
    
    return c_to_py(function(error))


def _transformation_free(
    transformation: Transformation
):
    """Internal function. Free the memory associated with `transformation`.
    
    :param transformation: 
    :type transformation: Transformation
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
    
    return c_to_py(unwrap(function(transformation), ctypes.c_void_p))


def _measurement_free(
    measurement: Measurement
):
    """Internal function. Free the memory associated with `measurement`.
    
    :param measurement: 
    :type measurement: Measurement
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
    
    return c_to_py(unwrap(function(measurement), ctypes.c_void_p))


def transformation_check(
    transformation: Transformation,
    d_in,
    d_out
) -> bool:
    """Check the stability relation of the `transformation` at the given `d_in`, `d_out`.
    
    :param transformation: Transformation to check the stability relation of.
    :type transformation: Transformation
    :param d_in: Distance in terms of the input metric.
    :param d_out: Distance in terms of the output metric.
    :return: True indicates that the relation passed at the given distance.
    :rtype: bool
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation = py_to_c(transformation, c_type=Transformation)
    d_in = py_to_c(d_in, c_type=AnyMetricDistancePtr, type_name=transformation_input_distance_type(transformation))
    d_out = py_to_c(d_out, c_type=AnyMetricDistancePtr, type_name=transformation_output_distance_type(transformation))
    
    # Call library function.
    function = lib.opendp_core__transformation_check
    function.argtypes = [Transformation, AnyMetricDistancePtr, AnyMetricDistancePtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation, d_in, d_out), BoolPtr))


def measurement_check(
    measurement: Measurement,
    d_in,
    d_out
) -> bool:
    """Check the privacy relation of the `measurement` at the given `d_in`, `d_out`.
    
    :param measurement: Measurement to check the privacy relation of.
    :type measurement: Measurement
    :param d_in: Distance in terms of the input metric.
    :param d_out: Distance in terms of the output measure.
    :return: True indicates that the relation passed at the given distance.
    :rtype: bool
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement)
    d_in = py_to_c(d_in, c_type=AnyMetricDistancePtr, type_name=measurement_input_distance_type(measurement))
    d_out = py_to_c(d_out, c_type=AnyMeasureDistancePtr, type_name=measurement_output_distance_type(measurement))
    
    # Call library function.
    function = lib.opendp_core__measurement_check
    function.argtypes = [Measurement, AnyMetricDistancePtr, AnyMeasureDistancePtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, d_in, d_out), BoolPtr))


def measurement_invoke(
    measurement: Measurement,
    arg: Any
) -> Any:
    """Invoke the `measurement` with `arg`. Returns a differentially private release.
    
    :param measurement: Measurement to invoke.
    :type measurement: Measurement
    :param arg: Input data to supply to the measurement. A member of the measurement's input domain.
    :type arg: Any
    :return: Differentially private release.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement)
    arg = py_to_c(arg, c_type=AnyObjectPtr, type_name=measurement_input_carrier_type(measurement))
    
    # Call library function.
    function = lib.opendp_core__measurement_invoke
    function.argtypes = [Measurement, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, arg), AnyObjectPtr))


def transformation_invoke(
    transformation: Transformation,
    arg: Any
) -> Any:
    """Invoke the `transformation` with `arg`. 
    The response is not differentially private as it has not been chained with a measurement.
    
    :param transformation: Transformation to invoke.
    :type transformation: Transformation
    :param arg: Input data to supply to the measurement. A member of the transformations's input domain.
    :type arg: Any
    :return: Non-differentially private answer to the query.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation = py_to_c(transformation, c_type=Transformation)
    arg = py_to_c(arg, c_type=AnyObjectPtr, type_name=transformation_input_carrier_type(transformation))
    
    # Call library function.
    function = lib.opendp_core__transformation_invoke
    function.argtypes = [Transformation, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation, arg), AnyObjectPtr))


def make_chain_mt(
    measurement: Measurement,
    transformation: Transformation
) -> Measurement:
    """Construct the functional composition (`measurement` ○ `transformation`). Returns a Measurement.
    
    :param measurement: outer privatizer
    :type measurement: Measurement
    :param transformation: inner query
    :type transformation: Transformation
    :return: Measurement representing the chained computation.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement)
    transformation = py_to_c(transformation, c_type=Transformation)
    
    # Call library function.
    function = lib.opendp_core__make_chain_mt
    function.argtypes = [Measurement, Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, transformation), Measurement))


def make_chain_tt(
    transformation1: Transformation,
    transformation0: Transformation
) -> Transformation:
    """Construct the functional composition (`transformation1` ○ `transformation0`). Returns a Tranformation.
    
    :param transformation1: outer transformation
    :type transformation1: Transformation
    :param transformation0: inner transformation
    :type transformation0: Transformation
    :return: Transformation representing the chained computation.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation1 = py_to_c(transformation1, c_type=Transformation)
    transformation0 = py_to_c(transformation0, c_type=Transformation)
    
    # Call library function.
    function = lib.opendp_core__make_chain_tt
    function.argtypes = [Transformation, Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation1, transformation0), Transformation))


def make_basic_composition(
    measurement0: Measurement,
    measurement1: Measurement
) -> Measurement:
    """Construct the DP composition (`measurement0`, `measurement1`). Returns a Measurement.
    
    :param measurement0: The left member of the resulting 2-tuple.
    :type measurement0: Measurement
    :param measurement1: The right member of the resulting 2-tuple.
    :type measurement1: Measurement
    :return: Measurement representing the composed transformations.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement0 = py_to_c(measurement0, c_type=Measurement)
    measurement1 = py_to_c(measurement1, c_type=Measurement)
    
    # Call library function.
    function = lib.opendp_core__make_basic_composition
    function.argtypes = [Measurement, Measurement]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement0, measurement1), Measurement))


def transformation_input_carrier_type(
    transformation: Transformation
) -> str:
    """Get the input (carrier) data type of `transformation`.
    
    :param transformation: The transformation to retrieve the type from.
    :type transformation: Transformation
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation = py_to_c(transformation, c_type=Transformation)
    
    # Call library function.
    function = lib.opendp_core__transformation_input_carrier_type
    function.argtypes = [Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation), ctypes.c_char_p))


def measurement_input_carrier_type(
    measurement: Measurement
) -> str:
    """Get the input (carrier) data type of `measurement`.
    
    :param measurement: The measurement to retrieve the type from.
    :type measurement: Measurement
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement)
    
    # Call library function.
    function = lib.opendp_core__measurement_input_carrier_type
    function.argtypes = [Measurement]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement), ctypes.c_char_p))


def transformation_input_distance_type(
    transformation: Transformation
) -> str:
    """Get the input distance type of `transformation`.
    
    :param transformation: The transformation to retrieve the type from.
    :type transformation: Transformation
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation = py_to_c(transformation, c_type=Transformation)
    
    # Call library function.
    function = lib.opendp_core__transformation_input_distance_type
    function.argtypes = [Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation), ctypes.c_char_p))


def transformation_output_distance_type(
    transformation: Transformation
) -> str:
    """Get the output distance type of `transformation`.
    
    :param transformation: The transformation to retrieve the type from.
    :type transformation: Transformation
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation = py_to_c(transformation, c_type=Transformation)
    
    # Call library function.
    function = lib.opendp_core__transformation_output_distance_type
    function.argtypes = [Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation), ctypes.c_char_p))


def measurement_input_distance_type(
    measurement: Measurement
) -> str:
    """Get the input distance type of `measurement`.
    
    :param measurement: The measurement to retrieve the type from.
    :type measurement: Measurement
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement)
    
    # Call library function.
    function = lib.opendp_core__measurement_input_distance_type
    function.argtypes = [Measurement]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement), ctypes.c_char_p))


def measurement_output_distance_type(
    measurement: Measurement
) -> str:
    """Get the output distance type of `measurement`.
    
    :param measurement: The measurement to retrieve the type from.
    :type measurement: Measurement
    :rtype: str
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement)
    
    # Call library function.
    function = lib.opendp_core__measurement_output_distance_type
    function.argtypes = [Measurement]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement), ctypes.c_char_p))
