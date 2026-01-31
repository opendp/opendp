# Auto-generated. Do not edit!
'''
The ``metrics`` module provides fuctions that measure the distance between two elements of a domain.
For more context, see :ref:`metrics in the User Guide <metrics-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp
'''
from deprecated.sphinx import deprecated # noqa: F401 (Not every file actually has deprecated functions.)

from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.typing import _substitute # noqa: F401

__all__ = [
    "_change_one_id_distance_get_identifier",
    "_extrinsic_metric_descriptor",
    "_frame_distance_get_inner_metric",
    "_get_bound",
    "_metric_equal",
    "_metric_free",
    "_symmetric_id_distance_get_identifier",
    "absolute_distance",
    "change_one_distance",
    "change_one_id_distance",
    "discrete_distance",
    "frame_distance",
    "hamming_distance",
    "insert_delete_distance",
    "l01inf_distance",
    "l02inf_distance",
    "l1_distance",
    "l2_distance",
    "linf_distance",
    "metric_debug",
    "metric_distance_type",
    "metric_type",
    "symmetric_distance",
    "symmetric_id_distance",
    "user_distance"
]


def _change_one_id_distance_get_identifier(
    metric: Metric
):
    r"""Retrieve the identifier of a `ChangeOneIdDistance` metric.

    .. end-markdown

    :param metric: 
    :type metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_metric = py_to_c(metric, c_type=Metric, type_name="AnyMetric")

    # Call library function.
    lib_function = lib.opendp_metrics___change_one_id_distance_get_identifier
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_metric), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_change_one_id_distance_get_identifier',
            '__module__': 'metrics',
            '__kwargs__': {
                'metric': metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _extrinsic_metric_descriptor(
    metric: Metric
):
    r"""Retrieve the descriptor value stored in an extrinsic metric.

    .. end-markdown

    :param metric: The ExtrinsicDistance to extract the descriptor from
    :type metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_metric = py_to_c(metric, c_type=Metric, type_name="AnyMetric")

    # Call library function.
    lib_function = lib.opendp_metrics___extrinsic_metric_descriptor
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_metric), ExtrinsicObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_extrinsic_metric_descriptor',
            '__module__': 'metrics',
            '__kwargs__': {
                'metric': metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _frame_distance_get_inner_metric(
    metric: Metric
) -> Metric:
    r"""Retrieve the inner metric of a `FrameDistance` metric.

    .. end-markdown

    :param metric: 
    :type metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_metric = py_to_c(metric, c_type=Metric, type_name="AnyMetric")

    # Call library function.
    lib_function = lib.opendp_metrics___frame_distance_get_inner_metric
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_metric), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': '_frame_distance_get_inner_metric',
            '__module__': 'metrics',
            '__kwargs__': {
                'metric': metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _get_bound(
    bounds,
    by
):
    r"""Infer a bound when grouping by `by`.

    .. end-markdown

    :param bounds: 
    :param by: 
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name="Bounds")
    c_by = py_to_c(by, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=["Expr"]))

    # Call library function.
    lib_function = lib.opendp_metrics___get_bound
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_bounds, c_by), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_get_bound',
            '__module__': 'metrics',
            '__kwargs__': {
                'bounds': bounds, 'by': by
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _metric_equal(
    left: Metric,
    right: Metric
) -> bool:
    r"""Check whether two metrics are equal.

    .. end-markdown

    :param left: Metric to compare.
    :type left: Metric
    :param right: Metric to compare.
    :type right: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_left = py_to_c(left, c_type=Metric, type_name="AnyMetric")
    c_right = py_to_c(right, c_type=Metric, type_name="AnyMetric")

    # Call library function.
    lib_function = lib.opendp_metrics___metric_equal
    lib_function.argtypes = [Metric, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_left, c_right), BoolPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_metric_equal',
            '__module__': 'metrics',
            '__kwargs__': {
                'left': left, 'right': right
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _metric_free(
    this
):
    r"""Internal function. Free the memory associated with `this`.

    .. end-markdown

    :param this: 
    :type this: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': '_metric_free',
            '__module__': 'metrics',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _symmetric_id_distance_get_identifier(
    metric: Metric
):
    r"""Retrieve the identifier of a `SymmetricIdDistance` metric.

    .. end-markdown

    :param metric: 
    :type metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_metric = py_to_c(metric, c_type=Metric, type_name="AnyMetric")

    # Call library function.
    lib_function = lib.opendp_metrics___symmetric_id_distance_get_identifier
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_metric), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_symmetric_id_distance_get_identifier',
            '__module__': 'metrics',
            '__kwargs__': {
                'metric': metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def absolute_distance(
    T: RuntimeTypeDescriptor
) -> Metric:
    r"""Construct an instance of the `AbsoluteDistance` metric.

    [absolute_distance in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/metrics/struct.AbsoluteDistance.html)

    .. end-markdown

    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'absolute_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def change_one_distance(

) -> Metric:
    r"""Construct an instance of the `ChangeOneDistance` metric.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__change_one_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'change_one_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def change_one_id_distance(
    identifier
) -> ChangeOneIdDistance:
    r"""Construct an instance of the `ChangeOneIdDistance` metric.

    .. end-markdown

    :param identifier: 
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_identifier = py_to_c(identifier, c_type=AnyObjectPtr, type_name="Expr")

    # Call library function.
    lib_function = lib.opendp_metrics__change_one_id_distance
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_identifier), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'change_one_id_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'identifier': identifier
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def discrete_distance(

) -> Metric:
    r"""Construct an instance of the `DiscreteDistance` metric.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__discrete_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'discrete_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def frame_distance(
    inner_metric: Metric
) -> Metric:
    r"""`frame_distance` is a higher-order metric that contains multiple distance bounds for different groupings of data.

    .. end-markdown

    :param inner_metric: 
    :type inner_metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_inner_metric = py_to_c(inner_metric, c_type=Metric, type_name="AnyMetric")

    # Call library function.
    lib_function = lib.opendp_metrics__frame_distance
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_inner_metric), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'frame_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'inner_metric': inner_metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def hamming_distance(

) -> Metric:
    r"""Construct an instance of the `HammingDistance` metric.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__hamming_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'hamming_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def insert_delete_distance(

) -> Metric:
    r"""Construct an instance of the `InsertDeleteDistance` metric.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__insert_delete_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'insert_delete_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def l01inf_distance(
    metric: Metric
) -> Metric:
    r"""Construct an instance of the `L01InfDistance` metric.

    [l01inf_distance in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/metrics/type.L01InfDistance.html)

    .. end-markdown

    :param metric: The metric used to compute distance between partitions.
    :type metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_metric = py_to_c(metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_metrics__l01inf_distance
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_metric), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'l01inf_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'metric': metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def l02inf_distance(
    metric: Metric
) -> Metric:
    r"""Construct an instance of the `L02InfDistance` metric.

    [l02inf_distance in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/metrics/type.L02InfDistance.html)

    .. end-markdown

    :param metric: The metric used to compute distance between partitions.
    :type metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_metric = py_to_c(metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_metrics__l02inf_distance
    lib_function.argtypes = [Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_metric), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'l02inf_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'metric': metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def l1_distance(
    T: RuntimeTypeDescriptor
) -> Metric:
    r"""Construct an instance of the `L1Distance` metric.

    [l1_distance in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/metrics/type.L1Distance.html)

    .. end-markdown

    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'l1_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def l2_distance(
    T: RuntimeTypeDescriptor
) -> Metric:
    r"""Construct an instance of the `L2Distance` metric.

    [l2_distance in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/metrics/type.L2Distance.html)

    .. end-markdown

    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'l2_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def linf_distance(
    T: RuntimeTypeDescriptor,
    monotonic: bool = False
) -> Metric:
    r"""Construct an instance of the `LInfDistance` metric.

    [linf_distance in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/metrics/struct.LInfDistance.html)

    .. end-markdown

    :param monotonic: set to true if non-monotonicity implies infinite distance
    :type monotonic: bool
    :param T: The type of the distance.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)

    # Convert arguments to c types.
    c_monotonic = py_to_c(monotonic, c_type=ctypes.c_bool, type_name="bool")
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_metrics__linf_distance
    lib_function.argtypes = [ctypes.c_bool, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_monotonic, c_T), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'linf_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'monotonic': monotonic, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def metric_debug(
    this: Metric
) -> str:
    r"""Debug a `metric`.

    .. end-markdown

    :param this: The metric to debug (stringify).
    :type this: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'metric_debug',
            '__module__': 'metrics',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def metric_distance_type(
    this: Metric
) -> str:
    r"""Get the distance type of a `metric`.

    .. end-markdown

    :param this: The metric to retrieve the distance type from.
    :type this: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'metric_distance_type',
            '__module__': 'metrics',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def metric_type(
    this: Metric
) -> str:
    r"""Get the type of a `metric`.

    .. end-markdown

    :param this: The metric to retrieve the type from.
    :type this: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'metric_type',
            '__module__': 'metrics',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def symmetric_distance(

) -> Metric:
    r"""Construct an instance of the `SymmetricDistance` metric.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_metrics__symmetric_distance
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'symmetric_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def symmetric_id_distance(
    identifier
) -> SymmetricIdDistance:
    r"""Construct an instance of the `SymmetricIdDistance` metric.

    .. end-markdown

    :param identifier: 
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_identifier = py_to_c(identifier, c_type=AnyObjectPtr, type_name="Expr")

    # Call library function.
    lib_function = lib.opendp_metrics__symmetric_id_distance
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_identifier), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'symmetric_id_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'identifier': identifier
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def user_distance(
    identifier: str,
    descriptor = None
) -> Metric:
    r"""Construct a new UserDistance.
    Any two instances of an UserDistance are equal if their string descriptors are equal.


    Required features: `honest-but-curious`

    **Why honest-but-curious?:**

    Your definition of `d` must satisfy the requirements of a pseudo-metric:

    1. for any $x$, $d(x, x) = 0$
    2. for any $x, y$, $d(x, y) \ge 0$ (non-negativity)
    3. for any $x, y$, $d(x, y) = d(y, x)$ (symmetry)
    4. for any $x, y, z$, $d(x, z) \le d(x, y) + d(y, z)$ (triangle inequality)

    .. end-markdown

    :param identifier: A string description of the metric.
    :type identifier: str
    :param descriptor: Additional constraints on the domain.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("honest-but-curious")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_identifier = py_to_c(identifier, c_type=ctypes.c_char_p, type_name=None)
    c_descriptor = py_to_c(descriptor, c_type=ExtrinsicObjectPtr, type_name="ExtrinsicObject")

    # Call library function.
    lib_function = lib.opendp_metrics__user_distance
    lib_function.argtypes = [ctypes.c_char_p, ExtrinsicObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_identifier, c_descriptor), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': 'user_distance',
            '__module__': 'metrics',
            '__kwargs__': {
                'identifier': identifier, 'descriptor': descriptor
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output
