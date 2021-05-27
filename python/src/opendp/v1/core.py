# Auto-generated. Do not edit.
import ctypes
from typing import Type, Union
from opendp.v1.convert import py_to_ptr, py_to_c, py_to_object, c_to_py
from opendp.v1.mod import lib, unwrap, AnyTransformationPtr, AnyMeasurementPtr, FfiResult, AnyObject, FfiSlice, FfiError, AnyObjectPtr, FfiSlicePtr, BoolPtr
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
    
    return c_to_py(function(error))


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
    transformation = py_to_c(transformation, c_type=AnyMeasurementPtr)
    d_in = py_to_object(d_in)
    d_out = py_to_object(d_out)
    
    # call library function
    function = lib.opendp_core__transformation_check
    function.argtypes = [AnyMeasurementPtr, AnyObjectPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation, d_in, d_out), BoolPtr))


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
    measurement = py_to_c(measurement, c_type=AnyMeasurementPtr)
    d_in = py_to_object(d_in)
    d_out = py_to_object(d_out)
    
    # call library function
    function = lib.opendp_core__measurement_check
    function.argtypes = [AnyMeasurementPtr, AnyObjectPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, d_in, d_out), BoolPtr))


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
    measurement = py_to_c(measurement, c_type=AnyMeasurementPtr)
    arg = py_to_object(arg)
    
    # call library function
    function = lib.opendp_core__measurement_invoke
    function.argtypes = [AnyMeasurementPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, arg), AnyObjectPtr))


def measurement_free(
    measurement
):
    """
    :param measurement: 
    """
    # parse type args
    
    
    # translate arguments to c types
    measurement = py_to_c(measurement, c_type=AnyMeasurementPtr)
    
    # call library function
    function = lib.opendp_core__measurement_free
    function.argtypes = [AnyMeasurementPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement), ctypes.c_void_p))


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
    transformation = py_to_c(transformation, c_type=AnyTransformationPtr)
    arg = py_to_object(arg)
    
    # call library function
    function = lib.opendp_core__transformation_invoke
    function.argtypes = [AnyTransformationPtr, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation, arg), AnyObjectPtr))


def transformation_free(
    transformation
):
    """
    :param transformation: 
    """
    # parse type args
    
    
    # translate arguments to c types
    transformation = py_to_c(transformation, c_type=AnyTransformationPtr)
    
    # call library function
    function = lib.opendp_core__transformation_free
    function.argtypes = [AnyTransformationPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation), ctypes.c_void_p))


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
    measurement = py_to_c(measurement, c_type=AnyMeasurementPtr)
    transformation = py_to_c(transformation, c_type=AnyTransformationPtr)
    
    # call library function
    function = lib.opendp_core__make_chain_mt
    function.argtypes = [AnyMeasurementPtr, AnyTransformationPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, transformation), AnyMeasurementPtr))


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
    transformation1 = py_to_c(transformation1, c_type=AnyTransformationPtr)
    transformation0 = py_to_c(transformation0, c_type=AnyTransformationPtr)
    
    # call library function
    function = lib.opendp_core__make_chain_tt
    function.argtypes = [AnyTransformationPtr, AnyTransformationPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation1, transformation0), AnyTransformationPtr))


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
    transformation0 = py_to_c(transformation0, c_type=AnyMeasurementPtr)
    transformation1 = py_to_c(transformation1, c_type=AnyMeasurementPtr)
    
    # call library function
    function = lib.opendp_core__make_composition
    function.argtypes = [AnyMeasurementPtr, AnyMeasurementPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation0, transformation1), AnyMeasurementPtr))
