# Auto-generated. Do not edit.
import ctypes
from typing import Type, Union
from opendp.v1.convert import py_to_ptr, py_to_c, py_to_object
from opendp.v1.mod import lib, unwrap, FfiTransformationPtr, FfiMeasurementPtr, FfiResult, FfiObject, FfiSlice, FfiError, FfiObjectPtr, FfiSlicePtr
from opendp.v1.typing import RuntimeType, RuntimeTypeDescriptor, DatasetMetric, SensitivityMetric


def make_identity(
    M: DatasetMetric,
    T: RuntimeTypeDescriptor
):
    """
    :param M: metric space
    :param T: type of data passed to the identity function
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    T = RuntimeType.parse(type_name=T)
    
    # translate arguments to c types
    M = py_to_c(M, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_identity
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(M, T), FfiTransformationPtr)


def make_split_lines(
    
):
    """
    
    """
    # parse type args
    
    
    # translate arguments to c types
    
    
    # call library function
    function = lib.opendp_trans__make_split_lines
    function.argtypes = []
    function.restype = FfiResult
    
    return unwrap(function(), FfiTransformationPtr)


def make_parse_series(
    impute: bool,
    M: DatasetMetric
):
    """
    :param impute: 
    :param M: 
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    
    # translate arguments to c types
    impute = py_to_c(impute, c_type=ctypes.c_bool)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_parse_series
    function.argtypes = [ctypes.c_bool, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(impute, M), FfiTransformationPtr)


def make_split_records(
    separator: str,
    M: DatasetMetric
):
    """
    :param separator: 
    :param M: 
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    
    # translate arguments to c types
    separator = py_to_c(separator, c_type=ctypes.c_char_p)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_split_records
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(separator, M), FfiTransformationPtr)


def make_create_dataframe(
    col_names,
    M: DatasetMetric,
    K: RuntimeTypeDescriptor = None
):
    """
    :param col_names: 
    :param M: 
    :param K: 
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    K = RuntimeType.parse_or_infer(type_name=K, public_example=col_names)
    
    # translate arguments to c types
    col_names = py_to_object(col_names, type_name=K)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    K = py_to_c(K, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_create_dataframe
    function.argtypes = [FfiObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(col_names, M, K), FfiTransformationPtr)


def make_split_dataframe(
    separator: str,
    col_names,
    M: DatasetMetric,
    K: RuntimeTypeDescriptor
):
    """
    :param separator: 
    :param col_names: 
    :param M: 
    :param K: 
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    K = RuntimeType.parse(type_name=K)
    
    # translate arguments to c types
    separator = py_to_c(separator, c_type=ctypes.c_char_p)
    col_names = py_to_object(col_names)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    K = py_to_c(K, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_split_dataframe
    function.argtypes = [ctypes.c_char_p, FfiObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(separator, col_names, M, K), FfiTransformationPtr)


def make_parse_column(
    key,
    impute: bool,
    M: DatasetMetric,
    T: RuntimeTypeDescriptor,
    K: RuntimeTypeDescriptor = None
):
    """
    :param key: name of column to select from dataframe and parse
    :param impute: if false, raise an error if parsing fails
    :param M: 
    :param K: data type of the key
    :param T: data type to parse into
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    K = RuntimeType.parse_or_infer(type_name=K, public_example=key)
    T = RuntimeType.parse(type_name=T)
    
    # translate arguments to c types
    key = py_to_ptr(key, type_name=K)
    impute = py_to_c(impute, c_type=ctypes.c_bool)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    K = py_to_c(K, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_parse_column
    function.argtypes = [ctypes.c_void_p, ctypes.c_bool, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(key, impute, M, K, T), FfiTransformationPtr)


def make_select_column(
    key,
    M: DatasetMetric,
    T: RuntimeTypeDescriptor,
    K: RuntimeTypeDescriptor = None
):
    """
    :param key: 
    :param M: 
    :param K: data type of the key
    :param T: data type to downcast to
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    K = RuntimeType.parse_or_infer(type_name=K, public_example=key)
    T = RuntimeType.parse(type_name=T)
    
    # translate arguments to c types
    key = py_to_ptr(key, type_name=K)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    K = py_to_c(K, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_select_column
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(key, M, K, T), FfiTransformationPtr)


def make_clamp_vec(
    lower,
    upper,
    M: DatasetMetric,
    T: RuntimeTypeDescriptor = None
):
    """
    :param lower: 
    :param upper: 
    :param M: 
    :param T: type of data being clamped
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    
    # translate arguments to c types
    lower = py_to_ptr(lower, type_name=T)
    upper = py_to_ptr(upper, type_name=T)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_clamp_vec
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(lower, upper, M, T), FfiTransformationPtr)


def make_clamp_scalar(
    lower,
    upper,
    M: DatasetMetric,
    T: RuntimeTypeDescriptor = None
):
    """
    :param lower: 
    :param upper: 
    :param M: 
    :param T: type of data being clamped
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    
    # translate arguments to c types
    lower = py_to_ptr(lower, type_name=T)
    upper = py_to_ptr(upper, type_name=T)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_clamp_scalar
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(lower, upper, M, T), FfiTransformationPtr)


def make_cast_vec(
    M: DatasetMetric,
    TI: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor
):
    """
    :param M: 
    :param TI: input data type
    :param TO: output data type
    """
    # parse type args
    M = RuntimeType.parse(type_name=M)
    TI = RuntimeType.parse(type_name=TI)
    TO = RuntimeType.parse(type_name=TO)
    
    # translate arguments to c types
    M = py_to_c(M, c_type=ctypes.c_char_p)
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_cast_vec
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(M, TI, TO), FfiTransformationPtr)


def make_bounded_covariance(
    lower,
    upper,
    length: int,
    ddof: int,
    MI: DatasetMetric,
    MO: SensitivityMetric
):
    """
    :param lower: 
    :param upper: 
    :param length: 
    :param ddof: 
    :param MI: 
    :param MO: 
    """
    # parse type args
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    T = MO.args[0]
    
    # translate arguments to c types
    lower = py_to_object(lower, type_name=(T, T))
    upper = py_to_object(upper, type_name=(T, T))
    length = py_to_c(length, c_type=ctypes.c_uint)
    ddof = py_to_c(ddof, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_bounded_covariance
    function.argtypes = [FfiObjectPtr, FfiObjectPtr, ctypes.c_uint, ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(lower, upper, length, ddof, MI, MO), FfiTransformationPtr)


def make_bounded_mean(
    lower,
    upper,
    length: int,
    MI: DatasetMetric,
    MO: SensitivityMetric
):
    """
    :param lower: 
    :param upper: 
    :param length: 
    :param MI: 
    :param MO: 
    """
    # parse type args
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    T = MO.args[0]
    
    # translate arguments to c types
    lower = py_to_ptr(lower, type_name=T)
    upper = py_to_ptr(upper, type_name=T)
    length = py_to_c(length, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_bounded_mean
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(lower, upper, length, MI, MO), FfiTransformationPtr)


def make_bounded_sum(
    lower,
    upper,
    MI: DatasetMetric,
    MO: SensitivityMetric
):
    """
    :param lower: 
    :param upper: 
    :param MI: 
    :param MO: 
    """
    # parse type args
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    T = MO.args[0]
    
    # translate arguments to c types
    lower = py_to_ptr(lower, type_name=T)
    upper = py_to_ptr(upper, type_name=T)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_bounded_sum
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(lower, upper, MI, MO), FfiTransformationPtr)


def make_bounded_sum_n(
    lower,
    upper,
    n: int,
    MI: DatasetMetric,
    MO: SensitivityMetric
):
    """
    :param lower: 
    :param upper: 
    :param n: 
    :param MI: 
    :param MO: 
    """
    # parse type args
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    T = MO.args[0]
    
    # translate arguments to c types
    lower = py_to_ptr(lower, type_name=T)
    upper = py_to_ptr(upper, type_name=T)
    n = py_to_c(n, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_bounded_sum_n
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(lower, upper, n, MI, MO), FfiTransformationPtr)


def make_bounded_variance(
    lower,
    upper,
    length: int,
    ddof: int,
    MI: DatasetMetric,
    MO: SensitivityMetric
):
    """
    :param lower: 
    :param upper: 
    :param length: 
    :param ddof: 
    :param MI: 
    :param MO: 
    """
    # parse type args
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    T = MO.args[0]
    
    # translate arguments to c types
    lower = py_to_ptr(lower, type_name=T)
    upper = py_to_ptr(upper, type_name=T)
    length = py_to_c(length, c_type=ctypes.c_uint)
    ddof = py_to_c(ddof, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_bounded_variance
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint, ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(lower, upper, length, ddof, MI, MO), FfiTransformationPtr)


def make_count(
    MI: DatasetMetric,
    MO: SensitivityMetric,
    TI: RuntimeTypeDescriptor
):
    """
    :param MI: 
    :param MO: 
    :param TI: 
    """
    # parse type args
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    TI = RuntimeType.parse(type_name=TI)
    
    # translate arguments to c types
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_count
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(MI, MO, TI), FfiTransformationPtr)


def make_count_by(
    n: int,
    MI: DatasetMetric,
    MO: SensitivityMetric,
    TI: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor
):
    """
    :param n: 
    :param MI: 
    :param MO: 
    :param TI: 
    :param TO: 
    """
    # parse type args
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    TI = RuntimeType.parse(type_name=TI)
    TO = RuntimeType.parse(type_name=TO)
    
    # translate arguments to c types
    n = py_to_c(n, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_count_by
    function.argtypes = [ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(n, MI, MO, TI, TO), FfiTransformationPtr)


def make_count_by_categories(
    categories,
    MI: DatasetMetric,
    MO: SensitivityMetric,
    TI: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor
):
    """
    :param categories: 
    :param MI: 
    :param MO: 
    :param TI: 
    :param TO: 
    """
    # parse type args
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    TI = RuntimeType.parse(type_name=TI)
    TO = RuntimeType.parse(type_name=TO)
    
    # translate arguments to c types
    categories = py_to_object(categories)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # call library function
    function = lib.opendp_trans__make_count_by_categories
    function.argtypes = [FfiObjectPtr, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return unwrap(function(categories, MI, MO, TI, TO), FfiTransformationPtr)
