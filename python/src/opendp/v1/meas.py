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


def make_base_vector_laplace(
    scale,
    T: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the multivariate laplace(`scale`) distribution to a vector value.
    
    :param scale: Noise scale parameter of the laplace distribution.
    :param T: Data type to be privatized.
    :type T: RuntimeTypeDescriptor
    :return: A base_vector_laplace step.
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
    function = lib.opendp_meas__make_base_vector_laplace
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


def make_base_vector_gaussian(
    scale,
    T: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the multivariate gaussian(`scale`) distribution to a vector value.
    
    :param scale: noise scale parameter to the gaussian distribution
    :param T: data type to be privatized
    :type T: RuntimeTypeDescriptor
    :return: A base_vector_gaussian step.
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
    function = lib.opendp_meas__make_base_vector_gaussian
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, T), Measurement))


def make_base_geometric(
    scale,
    T: RuntimeTypeDescriptor = "i32",
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the geometric(`scale`) distribution to a scalar value.
    
    :param scale: noise scale parameter to the geometric distribution
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
    T = RuntimeType.parse(type_name=T)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_geometric
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, T, QO), Measurement))


def make_base_vector_geometric(
    scale,
    T: RuntimeTypeDescriptor = "i32",
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the geometric(`scale`) distribution to a vector value.
    
    :param scale: noise scale parameter to the geometric distribution
    :param T: Data type to be privatized.
    :type T: RuntimeTypeDescriptor
    :param QO: Data type of the sensitivity space.
    :type QO: RuntimeTypeDescriptor
    :return: A base_vector_geometric step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_vector_geometric
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, T, QO), Measurement))


def make_constant_time_base_geometric(
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
    :return: A constant_time_base_geometric step.
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
    function = lib.opendp_meas__make_constant_time_base_geometric
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, lower, upper, T, QO), Measurement))


def make_constant_time_base_vector_geometric(
    scale,
    lower,
    upper,
    T: RuntimeTypeDescriptor = None,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the geometric(`scale`) distribution to a vector value.
    `lower` and `upper` are used to derive the max number of trials necessary when sampling from the geometric distribution.
    
    :param scale: noise scale parameter to the geometric distribution
    :param lower: Expected lower bound of data.
    :param upper: Expected upper bound of data.
    :param T: Data type to be privatized.
    :type T: RuntimeTypeDescriptor
    :param QO: Data type of the sensitivity space.
    :type QO: RuntimeTypeDescriptor
    :return: A constant_time_base_vector_geometric step.
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
    function = lib.opendp_meas__make_constant_time_base_vector_geometric
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, lower, upper, T, QO), Measurement))


def make_base_stability(
    n: int,
    scale,
    threshold,
    MI: SensitivityMetric,
    TIK: RuntimeTypeDescriptor,
    TIC: RuntimeTypeDescriptor = "i32"
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


def make_shuffle_amplification(
    step_epsilon: float,
    step_delta: float,
    num_steps: int,
    MI: DatasetMetric
) -> Measurement:
    """Make a Measurement that estimates privacy usage under shuffle amplification.
    
    :param step_epsilon: Epsilon usage of each disjoint step.
    :type step_epsilon: float
    :param step_delta: Epsilon usage of each disjoint step.
    :type step_delta: float
    :param num_steps: Number of disjoint steps taken.
    :type num_steps: int
    :param MI: input dataset metric
    :type MI: DatasetMetric
    :return: A shuffle_amplification step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    
    # Convert arguments to c types.
    step_epsilon = py_to_c(step_epsilon, c_type=ctypes.c_double)
    step_delta = py_to_c(step_delta, c_type=ctypes.c_double)
    num_steps = py_to_c(num_steps, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_shuffle_amplification
    function.argtypes = [ctypes.c_double, ctypes.c_double, ctypes.c_uint, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(step_epsilon, step_delta, num_steps, MI), Measurement))
