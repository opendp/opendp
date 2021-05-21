# Auto-generated. Do not edit.
import ctypes
from typing import Type, Union
from opendp.v1.convert import py_to_ptr, py_to_c, py_to_object
from opendp.v1.mod import lib, unwrap, FfiTransformationPtr, FfiMeasurementPtr, FfiResult, FfiObject, FfiSlice, FfiError, FfiObjectPtr, FfiSlicePtr
from opendp.v1.typing import RuntimeType, RuntimeTypeDescriptor, DatasetMetric, SensitivityMetric


def make_base_laplace(
    scale,
    T: RuntimeTypeDescriptor = None
):
    """
    Create a Measurement that adds noise from the laplace(scale) distribution.
    :param scale: noise scale parameter of the laplace distribution
    :param T: data type to be privatized
    """
    # parse type args
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # translate arguments to c types
    scale = py_to_ptr(scale, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_meas__make_base_laplace
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(scale, T), FfiMeasurementPtr)


def make_base_laplace_vec(
    scale,
    T: RuntimeTypeDescriptor = None
):
    """
    Create a Measurement that adds noise from the multivariate laplace(scale) distribution.
    :param scale: noise scale parameter of the laplace distribution
    :param T: data type to be privatized
    """
    # parse type args
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # translate arguments to c types
    scale = py_to_ptr(scale, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_meas__make_base_laplace_vec
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(scale, T), FfiMeasurementPtr)


def make_base_gaussian(
    scale,
    T: RuntimeTypeDescriptor = None
):
    """
    Create a Measurement that adds noise from the gaussian(scale) distribution.
    :param scale: noise scale parameter to the gaussian distribution
    :param T: data type to be privatized
    """
    # parse type args
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # translate arguments to c types
    scale = py_to_ptr(scale, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_meas__make_base_gaussian
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(scale, T), FfiMeasurementPtr)


def make_base_gaussian_vec(
    scale,
    T: RuntimeTypeDescriptor = None
):
    """
    Create a Measurement that adds noise from the multivariate gaussian(scale) distribution.
    :param scale: noise scale parameter to the gaussian distribution
    :param T: data type to be privatized
    """
    # parse type args
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # translate arguments to c types
    scale = py_to_ptr(scale, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_meas__make_base_gaussian_vec
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(scale, T), FfiMeasurementPtr)


def make_base_geometric(
    scale,
    min,
    max,
    T: RuntimeTypeDescriptor = None,
    QO: RuntimeTypeDescriptor = None
):
    """
    Create a Measurement that adds noise from the geometric(scale) distribution.
    :param scale: noise scale parameter to the geometric distribution
    :param min: 
    :param max: 
    :param T: data type to be privatized
    :param QO: data type of the sensitivity space
    """
    # parse type args
    T = RuntimeType.parse_or_infer(type_name=T, public_example=min)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    
    # translate arguments to c types
    scale = py_to_ptr(scale, type_name=QO)
    min = py_to_ptr(min, type_name=T)
    max = py_to_ptr(max, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_meas__make_base_geometric
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(scale, min, max, T, QO), FfiMeasurementPtr)


def make_base_stability(
    n: int,
    scale,
    threshold,
    MI: RuntimeTypeDescriptor,
    TIK: RuntimeTypeDescriptor,
    TIC: RuntimeTypeDescriptor = int
):
    """
    Create a Measurement that implements a stability-based filtering and noising.
    :param n: 
    :param scale: noise scale parameter
    :param threshold: exclude counts that are less than this minimum value
    :param MI: input metric space
    :param TIK: type of input key- hashable/categorical data type
    :param TIC: type of input count- integral
    """
    # parse type args
    MI = RuntimeType.parse(type_name=MI)
    TIK = RuntimeType.parse(type_name=TIK)
    TIC = RuntimeType.parse(type_name=TIC)
    
    # translate arguments to c types
    n = py_to_c(n, c_type=ctypes.c_uint)
    scale = py_to_ptr(scale, type_name=MI.args[0])
    threshold = py_to_ptr(threshold, type_name=MI.args[0])
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    TIK = py_to_c(TIK, c_type=ctypes.c_char_p)
    TIC = py_to_c(TIC, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_meas__make_base_stability
    function.argtypes = [ctypes.c_uint, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(n, scale, threshold, MI, TIK, TIC), FfiMeasurementPtr)
