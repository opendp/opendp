# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "make_base_laplace",
    "make_base_gaussian",
    "make_base_geometric",
    "make_base_stability"
]


def make_base_laplace(
    scale,
    D: RuntimeTypeDescriptor = "AllDomain<T>"
) -> Measurement:
    """Make a Measurement that adds noise from the laplace(`scale`) distribution to a scalar value.
    Adjust D to noise vector-valued data.
    
    :param scale: Noise scale parameter of the laplace distribution.
    :param D: Domain of the data type to be privatized. Valid values are VectorDomain<AllDomain<T>> or AllDomain<T>
    :type D: RuntimeTypeDescriptor
    :return: A base_laplace step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("floating-point", "contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D, generics=["T"])
    T = get_domain_atom_or_infer(D, scale)
    D = D.substitute(T=T)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_laplace
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, D), Measurement))


def make_base_gaussian(
    scale,
    D: RuntimeTypeDescriptor = "AllDomain<T>"
) -> Measurement:
    """Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
    Adjust D to noise vector-valued data.
    
    :param scale: noise scale parameter to the gaussian distribution
    :param D: Domain of the data type to be privatized. Valid values are VectorDomain<AllDomain<T>> or AllDomain<T>
    :type D: RuntimeTypeDescriptor
    :return: A base_gaussian step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("floating-point", "contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D, generics=["T"])
    T = get_domain_atom_or_infer(D, scale)
    D = D.substitute(T=T)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_gaussian
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, D), Measurement))


def make_base_geometric(
    scale,
    bounds: Any = None,
    D: RuntimeTypeDescriptor = "AllDomain<i32>",
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the geometric(`scale`) distribution to the input.
    Adjust D to noise vector-valued data.
    
    :param scale: noise scale parameter to the geometric distribution
    :param bounds: Set bounds on the count to make the algorithm run in constant-time.
    :type bounds: Any
    :param D: Domain of the data type to be privatized. Valid values are VectorDomain<AllDomain<T>> or AllDomain<T>
    :type D: RuntimeTypeDescriptor
    :param QO: Data type of the sensitivity, scale, and budget.
    :type QO: RuntimeTypeDescriptor
    :return: A base_geometric step.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    T = get_domain_atom(D)
    OptionT = RuntimeType(origin='Option', args=[RuntimeType(origin='Tuple', args=[T, T])])
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=OptionT)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_geometric
    function.argtypes = [ctypes.c_void_p, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, bounds, D, QO), Measurement))


def make_base_stability(
    size: int,
    scale,
    threshold,
    MI: SensitivityMetric,
    TIK: RuntimeTypeDescriptor,
    TIC: RuntimeTypeDescriptor = "i32"
) -> Measurement:
    """Make a Measurement that implements a stability-based filtering and noising.
    
    :param size: Number of records in the input vector.
    :type size: int
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
    assert_features("floating-point", "contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    TIK = RuntimeType.parse(type_name=TIK)
    TIC = RuntimeType.parse(type_name=TIC)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=MI.args[0])
    threshold = py_to_c(threshold, c_type=ctypes.c_void_p, type_name=MI.args[0])
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    TIK = py_to_c(TIK, c_type=ctypes.c_char_p)
    TIC = py_to_c(TIC, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_meas__make_base_stability
    function.argtypes = [ctypes.c_uint, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, scale, threshold, MI, TIK, TIC), Measurement))
