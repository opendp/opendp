# Auto-generated. Do not edit.
from opendp.v1._convert import *
from opendp.v1._lib import *
from opendp.v1.mod import *
from opendp.v1.typing import *


def make_base_laplace(
    scale,
    T: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the laplace(`scale`) distribution to a scalar value.
    
    
    `This constructor is supported by the linked proof. <https://www.overleaf.com/read/brvrprjhrhwb>`_
    
    :param scale: Noise scale parameter of the laplace distribution.
    :param T: Data type to be privatized.
    :type T: RuntimeTypeDescriptor
    :return: A base_laplace step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_laplace
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, T), Measurement))


def make_base_laplace_vec(
    scale,
    T: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the multivariate laplace(`scale`) distribution to a vector value.
    
    :param scale: Noise scale parameter of the laplace distribution.
    :param T: Data type to be privatized.
    :type T: RuntimeTypeDescriptor
    :return: A base_laplace_vec step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_laplace_vec
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, T), Measurement))


def make_base_gaussian(
    scale,
    T: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the gaussian(`scale`) distribution to a scalar value.
    
    :param scale: noise scale parameter to the gaussian distribution
    :param T: data type to be privatized
    :type T: RuntimeTypeDescriptor
    :return: A base_gaussian step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_gaussian
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, T), Measurement))


def make_base_gaussian_vec(
    scale,
    T: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the multivariate gaussian(`scale`) distribution to a vector value.
    
    :param scale: noise scale parameter to the gaussian distribution
    :param T: data type to be privatized
    :type T: RuntimeTypeDescriptor
    :return: A base_gaussian_vec step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_gaussian_vec
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, T), Measurement))


def make_base_geometric(
    scale,
    lower,
    upper,
    T: RuntimeTypeDescriptor = None,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the geometric(`scale`) distribution to a scalar value.
    `lower` and `upper` are used to derive the max number of trials necessary when sampling from the geometric distribution.
    
    :param scale: noise scale parameter to the geometric distribution
    :param lower: Expected lower bound of data.
    :param upper: Expected upper bound of data.
    :param T: Data type to be privatized.
    :type T: RuntimeTypeDescriptor
    :param QO: Data type of the sensitivity space.
    :type QO: RuntimeTypeDescriptor
    :return: A base_geometric step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    lower = py_to_c(lower, c_type=ctypes.c_void_p, type_name=T)
    upper = py_to_c(upper, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_geometric
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, lower, upper, T, QO), Measurement))


def make_base_stability(
    n: int,
    scale,
    threshold,
    MI: SensitivityMetric,
    TIK: RuntimeTypeDescriptor,
    TIC: RuntimeTypeDescriptor = int
) -> Measurement:
    """Make a Measurement that implements a stability-based filtering and noising.
    
    :param n: Number of records in the input vector.
    :type n: int
    :param scale: Noise scale parameter.
    :param threshold: Exclude counts that are less than this minimum value.
    :param MI: Input metric.
    :type MI: SensitivityMetric
    :param TIK: Data type of input key- must be hashable/categorical.
    :type TIK: RuntimeTypeDescriptor
    :param TIC: Data type of input count- must be integral.
    :type TIC: RuntimeTypeDescriptor
    :return: A base_stability step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    TIK = RuntimeType.parse(type_name=TIK)
    TIC = RuntimeType.parse(type_name=TIC)
    
    # Convert arguments to c types.
    n = py_to_c(n, c_type=ctypes.c_uint)
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=MI.args[0])
    threshold = py_to_c(threshold, c_type=ctypes.c_void_p, type_name=MI.args[0])
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    TIK = py_to_c(TIK, c_type=ctypes.c_char_p)
    TIC = py_to_c(TIC, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_stability
    function.argtypes = [ctypes.c_uint, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(n, scale, threshold, MI, TIK, TIC), Measurement))
