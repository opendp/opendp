# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "laplacian_scale_to_accuracy",
    "accuracy_to_laplacian_scale",
    "gaussian_scale_to_accuracy",
    "accuracy_to_gaussian_scale"
]


def laplacian_scale_to_accuracy(
    scale,
    alpha,
    T: RuntimeTypeDescriptor = None
) -> Any:
    """Convert a laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
    
    :param scale: Laplacian noise scale.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `scale` and `alpha`
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: Accuracy estimate. Maximum amount a value is expected to diverge at the given level-`alpha`.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_accuracy__laplacian_scale_to_accuracy
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, alpha, T), AnyObjectPtr))


def accuracy_to_laplacian_scale(
    accuracy,
    alpha,
    T: RuntimeTypeDescriptor = None
) -> Any:
    """Convert a desired `accuracy` (tolerance) into a laplacian noise scale at a statistical significance level `alpha`.
    
    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `accuracy` and `alpha`
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: Laplacian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)
    
    # Convert arguments to c types.
    accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_accuracy__accuracy_to_laplacian_scale
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(accuracy, alpha, T), AnyObjectPtr))


def gaussian_scale_to_accuracy(
    scale,
    alpha,
    T: RuntimeTypeDescriptor = None
) -> Any:
    """Convert a gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
    
    :param scale: Gaussian noise scale.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `scale` and `alpha`
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: Accuracy estimate. Maximum amount a value is expected to diverge at the given level-`alpha`.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_accuracy__gaussian_scale_to_accuracy
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, alpha, T), AnyObjectPtr))


def accuracy_to_gaussian_scale(
    accuracy,
    alpha,
    T: RuntimeTypeDescriptor = None
) -> Any:
    """Convert a desired `accuracy` (tolerance) into a gaussian noise scale at a statistical significance level `alpha`.
    
    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1).
    :param T: Data type of `accuracy` and `alpha`
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: Gaussian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
    :rtype: Any
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)
    
    # Convert arguments to c types.
    accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_accuracy__accuracy_to_gaussian_scale
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(accuracy, alpha, T), AnyObjectPtr))
