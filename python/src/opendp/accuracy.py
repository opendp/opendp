# Auto-generated. Do not edit!
'''
The ``accuracy`` module provides functions for converting between accuracy and scale parameters.
For more context, see :ref:`accuracy in the User Guide <accuracy-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp
'''
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "accuracy_to_discrete_gaussian_scale",
    "accuracy_to_discrete_laplacian_scale",
    "accuracy_to_gaussian_scale",
    "accuracy_to_laplacian_scale",
    "discrete_gaussian_scale_to_accuracy",
    "discrete_laplacian_scale_to_accuracy",
    "gaussian_scale_to_accuracy",
    "laplacian_scale_to_accuracy"
]


def accuracy_to_discrete_gaussian_scale(
    accuracy,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Any:
    r"""Convert a desired `accuracy` (tolerance) into a discrete gaussian noise scale at a statistical significance level `alpha`.

    [accuracy_to_discrete_gaussian_scale in Rust documentation.](https://docs.rs/opendp/latest/opendp/accuracy/fn.accuracy_to_discrete_gaussian_scale.html)

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/accuracy/accuracy_to_discrete_gaussian_scale.pdf)

    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `accuracy` and `alpha`
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)

    # Convert arguments to c types.
    c_accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__accuracy_to_discrete_gaussian_scale
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_accuracy, c_alpha, c_T), AnyObjectPtr))

    return output


def accuracy_to_discrete_laplacian_scale(
    accuracy,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Any:
    r"""Convert a desired `accuracy` (tolerance) into a discrete Laplacian noise scale at a statistical significance level `alpha`.

    [accuracy_to_discrete_laplacian_scale in Rust documentation.](https://docs.rs/opendp/latest/opendp/accuracy/fn.accuracy_to_discrete_laplacian_scale.html)

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/accuracy/accuracy_to_discrete_laplacian_scale.pdf)

    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `accuracy` and `alpha`
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :return: Discrete laplacian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)

    # Convert arguments to c types.
    c_accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__accuracy_to_discrete_laplacian_scale
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_accuracy, c_alpha, c_T), AnyObjectPtr))

    return output


def accuracy_to_gaussian_scale(
    accuracy,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Any:
    r"""Convert a desired `accuracy` (tolerance) into a gaussian noise scale at a statistical significance level `alpha`.

    [accuracy_to_gaussian_scale in Rust documentation.](https://docs.rs/opendp/latest/opendp/accuracy/fn.accuracy_to_gaussian_scale.html)

    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `accuracy` and `alpha`
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)

    # Convert arguments to c types.
    c_accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__accuracy_to_gaussian_scale
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_accuracy, c_alpha, c_T), AnyObjectPtr))

    return output


def accuracy_to_laplacian_scale(
    accuracy,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Any:
    r"""Convert a desired `accuracy` (tolerance) into a Laplacian noise scale at a statistical significance level `alpha`.

    [accuracy_to_laplacian_scale in Rust documentation.](https://docs.rs/opendp/latest/opendp/accuracy/fn.accuracy_to_laplacian_scale.html)

    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `accuracy` and `alpha`
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :return: Laplacian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)

    # Convert arguments to c types.
    c_accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__accuracy_to_laplacian_scale
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_accuracy, c_alpha, c_T), AnyObjectPtr))

    return output


def discrete_gaussian_scale_to_accuracy(
    scale,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Any:
    r"""Convert a discrete gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

    [discrete_gaussian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/latest/opendp/accuracy/fn.discrete_gaussian_scale_to_accuracy.html)

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/accuracy/discrete_gaussian_scale_to_accuracy.pdf)

    :param scale: Gaussian noise scale.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `scale` and `alpha`
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)

    # Convert arguments to c types.
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__discrete_gaussian_scale_to_accuracy
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_scale, c_alpha, c_T), AnyObjectPtr))

    return output


def discrete_laplacian_scale_to_accuracy(
    scale,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Any:
    r"""Convert a discrete Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

    $\alpha = P[Y \ge accuracy]$, where $Y = | X - z |$, and $X \sim \mathcal{L}_{Z}(0, scale)$.
    That is, $X$ is a discrete Laplace random variable and $Y$ is the distribution of the errors.

    This function returns a float accuracy.
    You can take the floor without affecting the coverage probability.

    [discrete_laplacian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/latest/opendp/accuracy/fn.discrete_laplacian_scale_to_accuracy.html)

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/accuracy/discrete_laplacian_scale_to_accuracy.pdf)

    :param scale: Discrete Laplacian noise scale.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `scale` and `alpha`
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)

    # Convert arguments to c types.
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__discrete_laplacian_scale_to_accuracy
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_scale, c_alpha, c_T), AnyObjectPtr))

    return output


def gaussian_scale_to_accuracy(
    scale,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Any:
    r"""Convert a gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

    [gaussian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/latest/opendp/accuracy/fn.gaussian_scale_to_accuracy.html)

    :param scale: Gaussian noise scale.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `scale` and `alpha`
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)

    # Convert arguments to c types.
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__gaussian_scale_to_accuracy
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_scale, c_alpha, c_T), AnyObjectPtr))

    return output


def laplacian_scale_to_accuracy(
    scale,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Any:
    r"""Convert a Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

    [laplacian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/latest/opendp/accuracy/fn.laplacian_scale_to_accuracy.html)

    :param scale: Laplacian noise scale.
    :param alpha: Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
    :param T: Data type of `scale` and `alpha`
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)

    # Convert arguments to c types.
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__laplacian_scale_to_accuracy
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_scale, c_alpha, c_T), AnyObjectPtr))

    return output
