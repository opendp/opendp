# Auto-generated. Do not edit.
import ctypes
from typing import Type, Union
from opendp.v1.convert import py_to_ptr, py_to_c, py_to_object
from opendp.v1.mod import lib, unwrap, FfiTransformationPtr, FfiMeasurementPtr, FfiResult, FfiObject, FfiSlice, FfiError, FfiObjectPtr, FfiSlicePtr
from opendp.v1.typing import RuntimeType, RuntimeTypeDescriptor, DatasetMetric, SensitivityMetric


def error_free(
    error
) -> bool:
    """
    :param error: 
    :rtype: bool
    """
    # parse type args
    
    
    # translate arguments to c types
    error = py_to_c(error, c_type=ctypes.POINTER(FfiError))
    
    # call library function
    function = lib.opendp_core__error_free
    function.argtypes = [ctypes.POINTER(FfiError)]
    function.restype = ctypes.c_bool
    
    return function(error)


def transformation_check(
    transformation,
    d_in,
    d_out
):
    """
    :param transformation: 
    :param d_in: 
    :param d_out: 
    """
    # parse type args
    
    
    # translate arguments to c types
    transformation = py_to_c(transformation, c_type=FfiMeasurementPtr)
    d_in = py_to_object(d_in)
    d_out = py_to_object(d_out)
    
    # call library function
    function = lib.opendp_core__transformation_check
    function.argtypes = [FfiMeasurementPtr, FfiObjectPtr, FfiObjectPtr]
    function.restype = FfiResult
    
    return unwrap(function(transformation, d_in, d_out), ctypes.POINTER(ctypes.c_bool))


def measurement_check(
    measurement,
    d_in,
    d_out
):
    """
    :param measurement: 
    :param d_in: 
    :param d_out: 
    """
    # parse type args
    
    
    # translate arguments to c types
    measurement = py_to_c(measurement, c_type=FfiMeasurementPtr)
    d_in = py_to_object(d_in)
    d_out = py_to_object(d_out)
    
    # call library function
    function = lib.opendp_core__measurement_check
    function.argtypes = [FfiMeasurementPtr, FfiObjectPtr, FfiObjectPtr]
    function.restype = FfiResult
    
    return unwrap(function(measurement, d_in, d_out), ctypes.POINTER(ctypes.c_bool))


def measurement_invoke(
    measurement,
    arg
):
    """
    :param measurement: 
    :param arg: 
    """
    # parse type args
    
    
    # translate arguments to c types
    measurement = py_to_c(measurement, c_type=FfiMeasurementPtr)
    arg = py_to_object(arg)
    
    # call library function
    function = lib.opendp_core__measurement_invoke
    function.argtypes = [FfiMeasurementPtr, FfiObjectPtr]
    function.restype = FfiResult
    
    return unwrap(function(measurement, arg), FfiObjectPtr)


def measurement_free(
    measurement
):
    """
    :param measurement: 
    """
    # parse type args
    
    
    # translate arguments to c types
    measurement = py_to_c(measurement, c_type=FfiMeasurementPtr)
    
    # call library function
    function = lib.opendp_core__measurement_free
    function.argtypes = [FfiMeasurementPtr]
    function.restype = FfiResult
    
    return unwrap(function(measurement), ctypes.c_void_p)


def transformation_invoke(
    transformation,
    arg
):
    """
    :param transformation: 
    :param arg: 
    """
    # parse type args
    
    
    # translate arguments to c types
    transformation = py_to_c(transformation, c_type=FfiTransformationPtr)
    arg = py_to_object(arg)
    
    # call library function
    function = lib.opendp_core__transformation_invoke
    function.argtypes = [FfiTransformationPtr, FfiObjectPtr]
    function.restype = FfiResult
    
    return unwrap(function(transformation, arg), FfiObjectPtr)


def transformation_free(
    transformation
):
    """
    :param transformation: 
    """
    # parse type args
    
    
    # translate arguments to c types
    transformation = py_to_c(transformation, c_type=FfiTransformationPtr)
    
    # call library function
    function = lib.opendp_core__transformation_free
    function.argtypes = [FfiTransformationPtr]
    function.restype = FfiResult
    
    return unwrap(function(transformation), ctypes.c_void_p)


def make_chain_mt(
    measurement,
    transformation
):
    """
    :param measurement: 
    :param transformation: 
    """
    # parse type args
    
    
    # translate arguments to c types
    measurement = py_to_c(measurement, c_type=FfiMeasurementPtr)
    transformation = py_to_c(transformation, c_type=FfiTransformationPtr)
    
    # call library function
    function = lib.opendp_core__make_chain_mt
    function.argtypes = [FfiMeasurementPtr, FfiTransformationPtr]
    function.restype = FfiResult
    
    return unwrap(function(measurement, transformation), FfiMeasurementPtr)


def make_chain_tt(
    transformation1,
    transformation0
):
    """
    :param transformation1: 
    :param transformation0: 
    """
    # parse type args
    
    
    # translate arguments to c types
    transformation1 = py_to_c(transformation1, c_type=FfiTransformationPtr)
    transformation0 = py_to_c(transformation0, c_type=FfiTransformationPtr)
    
    # call library function
    function = lib.opendp_core__make_chain_tt
    function.argtypes = [FfiTransformationPtr, FfiTransformationPtr]
    function.restype = FfiResult
    
    return unwrap(function(transformation1, transformation0), FfiTransformationPtr)


def make_composition(
    transformation0,
    transformation1
):
    """
    :param transformation0: 
    :param transformation1: 
    """
    # parse type args
    
    
    # translate arguments to c types
    transformation0 = py_to_c(transformation0, c_type=FfiMeasurementPtr)
    transformation1 = py_to_c(transformation1, c_type=FfiMeasurementPtr)
    
    # call library function
    function = lib.opendp_core__make_composition
    function.argtypes = [FfiMeasurementPtr, FfiMeasurementPtr]
    function.restype = FfiResult
    
    return unwrap(function(transformation0, transformation1), FfiMeasurementPtr)
