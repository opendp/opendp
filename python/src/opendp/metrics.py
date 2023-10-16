# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "_metric_free",
    "absolute_distance",
    "change_one_distance",
    "discrete_distance",
    "hamming_distance",
    "insert_delete_distance",
    "l1_distance",
    "l2_distance",
    "linf_diff_distance",
    "metric_debug",
    "metric_distance_type",
    "metric_type",
    "symmetric_distance",
    "user_distance"
]


@versioned
def _metric_free(
    this
):
    r"""Internal function. Free the memory associated with `this`.
    
    [_metric_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn._metric_free.html)
    
    :param this: 
    :type this: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this
    
    # Call library function.
    lib_function = lib.opendp_metrics___metric_free
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))
    
    return output


@versioned
def absolute_distance(
    T: RuntimeTypeDescriptor
) -> Metric:
    r"""Construct an instance of the `AbsoluteDistance` metric.
    
    [absolute_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.absolute_distance.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_metrics__absolute_distance
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Metric))
    
    return output


@versioned
def change_one_distance(
    
) -> Metric:
    r"""Construct an instance of the `ChangeOneDistance` metric.
    
    [change_one_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.change_one_distance.html)
    
    
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__change_one_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(), Metric))
    
    return output


@versioned
def discrete_distance(
    
) -> Metric:
    r"""Construct an instance of the `DiscreteDistance` metric.
    
    [discrete_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.discrete_distance.html)
    
    
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__discrete_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(), Metric))
    
    return output


@versioned
def hamming_distance(
    
) -> Metric:
    r"""Construct an instance of the `HammingDistance` metric.
    
    [hamming_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.hamming_distance.html)
    
    
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__hamming_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(), Metric))
    
    return output


@versioned
def insert_delete_distance(
    
) -> Metric:
    r"""Construct an instance of the `InsertDeleteDistance` metric.
    
    [insert_delete_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.insert_delete_distance.html)
    
    
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__insert_delete_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(), Metric))
    
    return output


@versioned
def l1_distance(
    T: RuntimeTypeDescriptor
) -> Metric:
    r"""Construct an instance of the `L1Distance` metric.
    
    [l1_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.l1_distance.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_metrics__l1_distance
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Metric))
    
    return output


@versioned
def l2_distance(
    T: RuntimeTypeDescriptor
) -> Metric:
    r"""Construct an instance of the `L2Distance` metric.
    
    [l2_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.l2_distance.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_metrics__l2_distance
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Metric))
    
    return output


@versioned
def linf_diff_distance(
    T: RuntimeTypeDescriptor
) -> Metric:
    r"""Construct an instance of the `LInfDiffDistance` metric.
    
    [linf_diff_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.linf_diff_distance.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_metrics__linf_diff_distance
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Metric))
    
    return output


@versioned
def metric_debug(
    this: Metric
) -> str:
    r"""Debug a `metric`.
    
    [metric_debug in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.metric_debug.html)
    
    :param this: The metric to debug (stringify).
    :type this: Metric
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Metric, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_metrics__metric_debug
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def metric_distance_type(
    this: Metric
) -> str:
    r"""Get the distance type of a `metric`.
    
    [metric_distance_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.metric_distance_type.html)
    
    :param this: The metric to retrieve the distance type from.
    :type this: Metric
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Metric, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_metrics__metric_distance_type
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def metric_type(
    this: Metric
) -> str:
    r"""Get the type of a `metric`.
    
    [metric_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.metric_type.html)
    
    :param this: The metric to retrieve the type from.
    :type this: Metric
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Metric, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_metrics__metric_type
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def symmetric_distance(
    
) -> Metric:
    r"""Construct an instance of the `SymmetricDistance` metric.
    
    [symmetric_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.symmetric_distance.html)
    
    
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__symmetric_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(), Metric))
    
    return output


@versioned
def user_distance(
    descriptor: str
) -> Metric:
    r"""Construct a new UserDistance.
    Any two instances of an UserDistance are equal if their string descriptors are equal.
    
    [user_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.user_distance.html)
    
    :param descriptor: A string description of the metric.
    :type descriptor: str
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("honest-but-curious")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_descriptor = py_to_c(descriptor, c_type=ctypes.c_char_p, type_name=String)
    
    # Call library function.
    lib_function = lib.opendp_metrics__user_distance
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_descriptor), Metric))
    
    return output
