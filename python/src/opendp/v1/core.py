# Auto-generated. Do not edit.
from opendp.v1.convert import _py_to_c, _c_to_py
from opendp.v1.mod import *
from opendp.v1.typing import *


def _error_free(
    error: FfiError
) -> bool:
    """
    Internal function. Free the memory associated with `error`.
    :param error: 
    :type error: FfiError
    :return: true indicates successful free
    :rtype: bool
    """
    
    # translate arguments to c types
    error = _py_to_c(error, c_type=ctypes.POINTER(FfiError))
    
    # call library function
    function = lib.opendp_core___error_free
    function.argtypes = [ctypes.POINTER(FfiError)]
    function.restype = ctypes.c_bool
    
    return _c_to_py(function(error))


def _transformation_free(
    transformation: AnyTransformationPtr
):
    """
    Internal function. Free the memory associated with `transformation`.
    :param transformation: 
    :type transformation: AnyTransformationPtr
    """
    
    # translate arguments to c types
    transformation = _py_to_c(transformation, c_type=AnyTransformationPtr)
    
    # call library function
    function = lib.opendp_core___transformation_free
    function.argtypes = [AnyTransformationPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(transformation), ctypes.c_void_p))


def _measurement_free(
    measurement: AnyMeasurementPtr
):
    """
    Internal function. Free the memory associated with `measurement`.
    :param measurement: 
    :type measurement: AnyMeasurementPtr
    """
    
    # translate arguments to c types
    measurement = _py_to_c(measurement, c_type=AnyMeasurementPtr)
    
    # call library function
    function = lib.opendp_core___measurement_free
    function.argtypes = [AnyMeasurementPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(measurement), ctypes.c_void_p))


def transformation_check(
    transformation: AnyTransformationPtr,
    d_in,
    d_out
) -> bool:
    """
    Check the stability relation of the `transformation` at the given `d_in`, `d_out`.
    :param transformation: 
    :type transformation: AnyTransformationPtr
    :param d_in: 
    :param d_out: 
    :return: True indicates that the relation passed at the given distance.
    :rtype: bool
    """
    
    # translate arguments to c types
    transformation = _py_to_c(transformation, c_type=AnyTransformationPtr)
    d_in = _py_to_c(d_in, c_type=AnyMetricDistancePtr, type_name=transformation_input_distance_type(transformation))
    d_out = _py_to_c(d_out, c_type=AnyMetricDistancePtr, type_name=transformation_output_distance_type(transformation))
    
    # call library function
    function = lib.opendp_core__transformation_check
    function.argtypes = [AnyTransformationPtr, AnyMetricDistancePtr, AnyMetricDistancePtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(transformation, d_in, d_out), BoolPtr))


def measurement_check(
    measurement: AnyMeasurementPtr,
    d_in,
    d_out
) -> bool:
    """
    Check the privacy relation of the `measurement` at the given `d_in`, `d_out`.
    :param measurement: 
    :type measurement: AnyMeasurementPtr
    :param d_in: 
    :param d_out: 
    :return: True indicates that the relation passed at the given distance.
    :rtype: bool
    """
    
    # translate arguments to c types
    measurement = _py_to_c(measurement, c_type=AnyMeasurementPtr)
    d_in = _py_to_c(d_in, c_type=AnyMetricDistancePtr, type_name=measurement_input_distance_type(measurement))
    d_out = _py_to_c(d_out, c_type=AnyMeasureDistancePtr, type_name=measurement_output_distance_type(measurement))
    
    # call library function
    function = lib.opendp_core__measurement_check
    function.argtypes = [AnyMeasurementPtr, AnyMetricDistancePtr, AnyMeasureDistancePtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(measurement, d_in, d_out), BoolPtr))


def measurement_invoke(
    measurement: AnyMeasurementPtr,
    arg: Any
) -> Any:
    """
    Invoke the `measurement` with `arg`. Returns a differentially private release.
    :param measurement: 
    :type measurement: AnyMeasurementPtr
    :param arg: 
    :type arg: Any
    :return: Differentially private release.
    :rtype: Any
    """
    
    # translate arguments to c types
    measurement = _py_to_c(measurement, c_type=AnyMeasurementPtr)
    arg = _py_to_c(arg, c_type=AnyObjectPtr, type_name=measurement_input_carrier_type(measurement))
    
    # call library function
    function = lib.opendp_core__measurement_invoke
    function.argtypes = [AnyMeasurementPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(measurement, arg), AnyObjectPtr))


def transformation_invoke(
    transformation: AnyTransformationPtr,
    arg: Any
) -> Any:
    """
    Invoke the `transformation` with `arg`. 
    The response is not differentially private as it has not been chained with a measurement.
    :param transformation: 
    :type transformation: AnyTransformationPtr
    :param arg: 
    :type arg: Any
    :return: Non-differentially private answer to the query.
    :rtype: Any
    """
    
    # translate arguments to c types
    transformation = _py_to_c(transformation, c_type=AnyTransformationPtr)
    arg = _py_to_c(arg, c_type=AnyObjectPtr, type_name=transformation_input_carrier_type(transformation))
    
    # call library function
    function = lib.opendp_core__transformation_invoke
    function.argtypes = [AnyTransformationPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(transformation, arg), AnyObjectPtr))


def make_chain_mt(
    measurement: AnyMeasurementPtr,
    transformation: AnyTransformationPtr
) -> AnyMeasurementPtr:
    """
    Construct the functional composition (`measurement` ○ `transformation`). Returns a Measurement.
    :param measurement: outer privatizer
    :type measurement: AnyMeasurementPtr
    :param transformation: inner query
    :type transformation: AnyTransformationPtr
    :return: Measurement representing the chained computation.
    :rtype: AnyMeasurementPtr
    """
    
    # translate arguments to c types
    measurement = _py_to_c(measurement, c_type=AnyMeasurementPtr)
    transformation = _py_to_c(transformation, c_type=AnyTransformationPtr)
    
    # call library function
    function = lib.opendp_core__make_chain_mt
    function.argtypes = [AnyMeasurementPtr, AnyTransformationPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(measurement, transformation), AnyMeasurementPtr))


def make_chain_tt(
    transformation1: AnyTransformationPtr,
    transformation0: AnyTransformationPtr
) -> AnyTransformationPtr:
    """
    Construct the functional composition (`transformation1` ○ `transformation0`). Returns a Tranformation.
    :param transformation1: outer transformation
    :type transformation1: AnyTransformationPtr
    :param transformation0: inner transformation
    :type transformation0: AnyTransformationPtr
    :return: Transformation representing the chained computation.
    :rtype: AnyTransformationPtr
    """
    
    # translate arguments to c types
    transformation1 = _py_to_c(transformation1, c_type=AnyTransformationPtr)
    transformation0 = _py_to_c(transformation0, c_type=AnyTransformationPtr)
    
    # call library function
    function = lib.opendp_core__make_chain_tt
    function.argtypes = [AnyTransformationPtr, AnyTransformationPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(transformation1, transformation0), AnyTransformationPtr))


def make_basic_composition(
    measurement0: AnyMeasurementPtr,
    measurement1: AnyMeasurementPtr
) -> AnyMeasurementPtr:
    """
    Construct the DP composition (`measurement0`, `measurement1`). Returns a Measurement.
    :param measurement0: 
    :type measurement0: AnyMeasurementPtr
    :param measurement1: 
    :type measurement1: AnyMeasurementPtr
    :return: Measurement representing the composed transformations.
    :rtype: AnyMeasurementPtr
    """
    
    # translate arguments to c types
    measurement0 = _py_to_c(measurement0, c_type=AnyMeasurementPtr)
    measurement1 = _py_to_c(measurement1, c_type=AnyMeasurementPtr)
    
    # call library function
    function = lib.opendp_core__make_basic_composition
    function.argtypes = [AnyMeasurementPtr, AnyMeasurementPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(measurement0, measurement1), AnyMeasurementPtr))


def transformation_input_carrier_type(
    transformation: AnyTransformationPtr
) -> str:
    """
    Get the input (carrier) data type of `transformation`.
    :param transformation: 
    :type transformation: AnyTransformationPtr
    :rtype: str
    """
    
    # translate arguments to c types
    transformation = _py_to_c(transformation, c_type=AnyTransformationPtr)
    
    # call library function
    function = lib.opendp_core__transformation_input_carrier_type
    function.argtypes = [AnyTransformationPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(transformation), ctypes.c_char_p))


def measurement_input_carrier_type(
    measurement: AnyMeasurementPtr
) -> str:
    """
    Get the input (carrier) data type of `measurement`.
    :param measurement: 
    :type measurement: AnyMeasurementPtr
    :rtype: str
    """
    
    # translate arguments to c types
    measurement = _py_to_c(measurement, c_type=AnyMeasurementPtr)
    
    # call library function
    function = lib.opendp_core__measurement_input_carrier_type
    function.argtypes = [AnyMeasurementPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(measurement), ctypes.c_char_p))


def transformation_input_distance_type(
    transformation: AnyTransformationPtr
) -> str:
    """
    Get the input distance type of `transformation`.
    :param transformation: 
    :type transformation: AnyTransformationPtr
    :rtype: str
    """
    
    # translate arguments to c types
    transformation = _py_to_c(transformation, c_type=AnyTransformationPtr)
    
    # call library function
    function = lib.opendp_core__transformation_input_distance_type
    function.argtypes = [AnyTransformationPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(transformation), ctypes.c_char_p))


def transformation_output_distance_type(
    transformation: AnyTransformationPtr
) -> str:
    """
    Get the output distance type of `transformation`.
    :param transformation: 
    :type transformation: AnyTransformationPtr
    :rtype: str
    """
    
    # translate arguments to c types
    transformation = _py_to_c(transformation, c_type=AnyTransformationPtr)
    
    # call library function
    function = lib.opendp_core__transformation_output_distance_type
    function.argtypes = [AnyTransformationPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(transformation), ctypes.c_char_p))


def measurement_input_distance_type(
    measurement: AnyMeasurementPtr
) -> str:
    """
    Get the input distance type of `measurement`.
    :param measurement: 
    :type measurement: AnyMeasurementPtr
    :rtype: str
    """
    
    # translate arguments to c types
    measurement = _py_to_c(measurement, c_type=AnyMeasurementPtr)
    
    # call library function
    function = lib.opendp_core__measurement_input_distance_type
    function.argtypes = [AnyMeasurementPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(measurement), ctypes.c_char_p))


def measurement_output_distance_type(
    measurement: AnyMeasurementPtr
) -> str:
    """
    Get the output distance type of `measurement`.
    :param measurement: 
    :type measurement: AnyMeasurementPtr
    :rtype: str
    """
    
    # translate arguments to c types
    measurement = _py_to_c(measurement, c_type=AnyMeasurementPtr)
    
    # call library function
    function = lib.opendp_core__measurement_output_distance_type
    function.argtypes = [AnyMeasurementPtr]
    function.restype = FfiResult
    
    return _c_to_py(unwrap(function(measurement), ctypes.c_char_p))
