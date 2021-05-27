# Auto-generated. Do not edit.
import ctypes
from typing import Type, Union
from opendp.v1.convert import py_to_ptr, py_to_c, py_to_object, c_to_py
from opendp.v1.mod import lib, unwrap, AnyTransformationPtr, AnyMeasurementPtr, FfiResult, AnyObject, FfiSlice, FfiError, AnyObjectPtr, FfiSlicePtr, BoolPtr
from opendp.v1.typing import RuntimeType, RuntimeTypeDescriptor, DatasetMetric, SensitivityMetric


def to_string(
    this
):
    """
    :param this: 
    """
    # parse type args
    
    
    # translate arguments to c types
    this = py_to_object(this)
    
    # call library function
    function = lib.opendp_data__to_string
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def slice_as_object(
    raw: FfiSlicePtr,
    T: RuntimeTypeDescriptor = None
):
    """
    :param raw: 
    :type raw: FfiSlicePtr
    :param T: 
    :type T: RuntimeTypeDescriptor
    """
    # parse type args
    T = RuntimeType.parse_or_infer(type_name=T, public_example=raw)
    
    # translate arguments to c types
    raw = py_to_c(raw, c_type=FfiSlicePtr)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_data__slice_as_object
    function.argtypes = [FfiSlicePtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(raw, T), AnyObjectPtr)


def object_type(
    this
):
    """
    :param this: 
    """
    # parse type args
    
    
    # translate arguments to c types
    this = py_to_object(this)
    
    # call library function
    function = lib.opendp_data__object_type
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_char_p))


def object_as_slice(
    this
):
    """
    :param this: 
    """
    # parse type args
    
    
    # translate arguments to c types
    this = py_to_object(this)
    
    # call library function
    function = lib.opendp_data__object_as_slice
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), FfiSlicePtr))


def object_free(
    this
):
    """
    :param this: 
    """
    # parse type args
    
    
    # translate arguments to c types
    this = py_to_object(this)
    
    # call library function
    function = lib.opendp_data__object_free
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def slice_free(
    this
):
    """
    :param this: 
    """
    # parse type args
    
    
    # translate arguments to c types
    this = py_to_c(this, c_type=FfiSlicePtr)
    
    # call library function
    function = lib.opendp_data__slice_free
    function.argtypes = [FfiSlicePtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def str_free(
    this: str
):
    """
    :param this: 
    :type this: str
    """
    # parse type args
    
    
    # translate arguments to c types
    this = py_to_c(this, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_data__str_free
    function.argtypes = [ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))


def bool_free(
    this
):
    """
    :param this: 
    """
    # parse type args
    
    
    # translate arguments to c types
    this = py_to_c(this, c_type=BoolPtr)
    
    # call library function
    function = lib.opendp_data__bool_free
    function.argtypes = [BoolPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(this), ctypes.c_void_p))
