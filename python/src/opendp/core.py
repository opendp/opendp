# Auto-generated. Do not edit!
'''
The ``core`` module provides functions for accessing the fields of transformations and measurements.
For more context, see :ref:`core in the User Guide <core-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp
'''
from deprecated.sphinx import deprecated # noqa: F401 (Not every file actually has deprecated functions.)

from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "_error_free",
    "_function_free",
    "_measurement_free",
    "_transformation_free",
    "function_eval",
    "measurement_check",
    "measurement_function",
    "measurement_input_carrier_type",
    "measurement_input_distance_type",
    "measurement_input_domain",
    "measurement_input_metric",
    "measurement_invoke",
    "measurement_map",
    "measurement_output_distance_type",
    "measurement_output_measure",
    "new_function",
    "new_queryable",
    "queryable_eval",
    "queryable_query_type",
    "transformation_check",
    "transformation_function",
    "transformation_input_carrier_type",
    "transformation_input_distance_type",
    "transformation_input_domain",
    "transformation_input_metric",
    "transformation_invoke",
    "transformation_map",
    "transformation_output_distance_type",
    "transformation_output_domain",
    "transformation_output_metric"
]


def _error_free(
    this
) -> bool:
    r"""Internal function. Free the memory associated with `error`.

    :param this: 
    :type this: FfiError
    :return: A boolean, where true indicates successful free
    :rtype: bool
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this

    # Call library function.
    lib_function = lib.opendp_core___error_free
    lib_function.argtypes = [ctypes.POINTER(FfiError)]
    lib_function.restype = ctypes.c_bool

    output = c_to_py(lib_function(c_this))

    return output


def _function_free(
    this
):
    r"""Internal function. Free the memory associated with `this`.

    :param this: 
    :type this: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this

    # Call library function.
    lib_function = lib.opendp_core___function_free
    lib_function.argtypes = [Function]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))

    return output


def _measurement_free(
    this
):
    r"""Internal function. Free the memory associated with `this`.

    :param this: 
    :type this: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this

    # Call library function.
    lib_function = lib.opendp_core___measurement_free
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))

    return output


def _transformation_free(
    this
):
    r"""Internal function. Free the memory associated with `this`.

    :param this: 
    :type this: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this

    # Call library function.
    lib_function = lib.opendp_core___transformation_free
    lib_function.argtypes = [Transformation]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))

    return output


def function_eval(
    this: Function,
    arg,
    TI: Optional[str] = None
):
    r"""Eval the `function` with `arg`.

    :param this: Function to invoke.
    :type this: Function
    :param arg: Input data to supply to the measurement. A member of the measurement's input domain.
    :param TI: Input Type.
    :type TI: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Function, type_name=None)
    c_arg = py_to_c(arg, c_type=AnyObjectPtr, type_name=parse_or_infer(TI, arg))
    c_TI = py_to_c(TI, c_type=ctypes.c_char_p, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__function_eval
    lib_function.argtypes = [Function, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this, c_arg, c_TI), AnyObjectPtr))

    return output


def measurement_check(
    measurement: Measurement,
    distance_in,
    distance_out
):
    r"""Check the privacy relation of the `measurement` at the given `d_in`, `d_out`

    :param measurement: Measurement to check the privacy relation of.
    :type measurement: Measurement
    :param distance_in: 
    :param distance_out: 
    :return: True indicates that the relation passed at the given distance.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=None)
    c_distance_in = py_to_c(distance_in, c_type=AnyObjectPtr, type_name=measurement_input_distance_type(measurement))
    c_distance_out = py_to_c(distance_out, c_type=AnyObjectPtr, type_name=measurement_output_distance_type(measurement))

    # Call library function.
    lib_function = lib.opendp_core__measurement_check
    lib_function.argtypes = [Measurement, AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement, c_distance_in, c_distance_out), BoolPtr))

    return output


def measurement_function(
    this: Measurement
) -> Function:
    r"""Get the function from a measurement.

    :param this: The measurement to retrieve the value from.
    :type this: Measurement
    :rtype: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measurement, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__measurement_function
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_this), Function)

    return output


def measurement_input_carrier_type(
    this: Measurement
) -> str:
    r"""Get the input (carrier) data type of `this`.

    :param this: The measurement to retrieve the type from.
    :type this: Measurement
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measurement, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__measurement_input_carrier_type
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))

    return output


def measurement_input_distance_type(
    this: Measurement
) -> str:
    r"""Get the input distance type of `measurement`.

    :param this: The measurement to retrieve the type from.
    :type this: Measurement
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measurement, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__measurement_input_distance_type
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))

    return output


def measurement_input_domain(
    this: Measurement
) -> Domain:
    r"""Get the input domain from a `measurement`.

    :param this: The measurement to retrieve the value from.
    :type this: Measurement
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measurement, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__measurement_input_domain
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_this), Domain)

    return output


def measurement_input_metric(
    this: Measurement
) -> Metric:
    r"""Get the input domain from a `measurement`.

    :param this: The measurement to retrieve the value from.
    :type this: Measurement
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measurement, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__measurement_input_metric
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_this), Metric)

    return output


def measurement_invoke(
    this: Measurement,
    arg
):
    r"""Invoke the `measurement` with `arg`. Returns a differentially private release.

    :param this: Measurement to invoke.
    :type this: Measurement
    :param arg: Input data to supply to the measurement. A member of the measurement's input domain.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measurement, type_name=None)
    c_arg = py_to_c(arg, c_type=AnyObjectPtr, type_name=measurement_input_carrier_type(this))

    # Call library function.
    lib_function = lib.opendp_core__measurement_invoke
    lib_function.argtypes = [Measurement, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this, c_arg), AnyObjectPtr))

    return output


def measurement_map(
    measurement: Measurement,
    distance_in
):
    r"""Use the `measurement` to map a given `d_in` to `d_out`.

    :param measurement: Measurement to check the map distances with.
    :type measurement: Measurement
    :param distance_in: Distance in terms of the input metric.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=None)
    c_distance_in = py_to_c(distance_in, c_type=AnyObjectPtr, type_name=measurement_input_distance_type(measurement))

    # Call library function.
    lib_function = lib.opendp_core__measurement_map
    lib_function.argtypes = [Measurement, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement, c_distance_in), AnyObjectPtr))

    return output


def measurement_output_distance_type(
    this: Measurement
) -> str:
    r"""Get the output distance type of `measurement`.

    :param this: The measurement to retrieve the type from.
    :type this: Measurement
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measurement, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__measurement_output_distance_type
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))

    return output


def measurement_output_measure(
    this: Measurement
) -> Measure:
    r"""Get the output domain from a `measurement`.

    :param this: The measurement to retrieve the value from.
    :type this: Measurement
    :rtype: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measurement, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__measurement_output_measure
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_this), Measure)

    return output


def new_function(
    function,
    TO: RuntimeTypeDescriptor
) -> Function:
    r"""Construct a Function from a user-defined callback.
    Can be used as a post-processing step.


    Required features: `contrib`

    :param function: A function mapping data to a value of type `TO`
    :param TO: Output Type
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TO = RuntimeType.parse(type_name=TO)

    # Convert arguments to c types.
    c_function = py_to_c(function, c_type=CallbackFn, type_name=pass_through(TO))
    c_TO = py_to_c(TO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_core__new_function
    lib_function.argtypes = [CallbackFn, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_function, c_TO), Function))
    output._depends_on(c_function)
    return output


def new_queryable(
    transition,
    Q: RuntimeTypeDescriptor = "ExtrinsicObject",
    A: RuntimeTypeDescriptor = "ExtrinsicObject"
):
    r"""Construct a queryable from a user-defined transition function.


    Required features: `contrib`

    :param transition: A transition function taking a reference to self, a query, and an internal/external indicator
    :param Q: Query Type
    :type Q: :py:ref:`RuntimeTypeDescriptor`
    :param A: Output Type
    :type A: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    Q = RuntimeType.parse(type_name=Q)
    A = RuntimeType.parse(type_name=A)

    # Convert arguments to c types.
    c_transition = py_to_c(transition, c_type=TransitionFn, type_name=pass_through(A))
    c_Q = py_to_c(Q, c_type=ctypes.c_char_p)
    c_A = py_to_c(A, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_core__new_queryable
    lib_function.argtypes = [TransitionFn, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_transition, c_Q, c_A), AnyObjectPtr))
    output._depends_on(c_transition)
    return output


def queryable_eval(
    queryable,
    query
):
    r"""Invoke the `queryable` with `query`. Returns a differentially private release.

    :param queryable: Queryable to eval.
    :param query: Input data to supply to the measurement. A member of the measurement's input domain.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_queryable = py_to_c(queryable, c_type=AnyObjectPtr, type_name=None)
    c_query = py_to_c(query, c_type=AnyObjectPtr, type_name=queryable_query_type(queryable))

    # Call library function.
    lib_function = lib.opendp_core__queryable_eval
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_queryable, c_query), AnyObjectPtr))

    return output


def queryable_query_type(
    this
) -> str:
    r"""Get the query type of `queryable`.

    :param this: The queryable to retrieve the query type from.
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=AnyObjectPtr, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__queryable_query_type
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))

    return output


def transformation_check(
    transformation: Transformation,
    distance_in,
    distance_out
):
    r"""Check the privacy relation of the `measurement` at the given `d_in`, `d_out`

    :param transformation: 
    :type transformation: Transformation
    :param distance_in: 
    :param distance_out: 
    :return: True indicates that the relation passed at the given distance.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_transformation = py_to_c(transformation, c_type=Transformation, type_name=None)
    c_distance_in = py_to_c(distance_in, c_type=AnyObjectPtr, type_name=transformation_input_distance_type(transformation))
    c_distance_out = py_to_c(distance_out, c_type=AnyObjectPtr, type_name=transformation_output_distance_type(transformation))

    # Call library function.
    lib_function = lib.opendp_core__transformation_check
    lib_function.argtypes = [Transformation, AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_transformation, c_distance_in, c_distance_out), BoolPtr))

    return output


def transformation_function(
    this: Transformation
) -> Function:
    r"""Get the function from a transformation.

    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :rtype: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__transformation_function
    lib_function.argtypes = [Transformation]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_this), Function)

    return output


def transformation_input_carrier_type(
    this: Transformation
) -> str:
    r"""Get the input (carrier) data type of `this`.

    :param this: The transformation to retrieve the type from.
    :type this: Transformation
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__transformation_input_carrier_type
    lib_function.argtypes = [Transformation]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))

    return output


def transformation_input_distance_type(
    this: Transformation
) -> str:
    r"""Get the input distance type of `transformation`.

    :param this: The transformation to retrieve the type from.
    :type this: Transformation
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__transformation_input_distance_type
    lib_function.argtypes = [Transformation]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))

    return output


def transformation_input_domain(
    this: Transformation
) -> Domain:
    r"""Get the input domain from a `transformation`.

    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__transformation_input_domain
    lib_function.argtypes = [Transformation]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_this), Domain)

    return output


def transformation_input_metric(
    this: Transformation
) -> Metric:
    r"""Get the input domain from a `transformation`.

    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__transformation_input_metric
    lib_function.argtypes = [Transformation]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_this), Metric)

    return output


def transformation_invoke(
    this: Transformation,
    arg
):
    r"""Invoke the `transformation` with `arg`. Returns a differentially private release.

    :param this: Transformation to invoke.
    :type this: Transformation
    :param arg: Input data to supply to the transformation. A member of the transformation's input domain.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Transformation, type_name=None)
    c_arg = py_to_c(arg, c_type=AnyObjectPtr, type_name=transformation_input_carrier_type(this))

    # Call library function.
    lib_function = lib.opendp_core__transformation_invoke
    lib_function.argtypes = [Transformation, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this, c_arg), AnyObjectPtr))

    return output


def transformation_map(
    transformation: Transformation,
    distance_in
):
    r"""Use the `transformation` to map a given `d_in` to `d_out`.

    :param transformation: Transformation to check the map distances with.
    :type transformation: Transformation
    :param distance_in: Distance in terms of the input metric.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_transformation = py_to_c(transformation, c_type=Transformation, type_name=None)
    c_distance_in = py_to_c(distance_in, c_type=AnyObjectPtr, type_name=transformation_input_distance_type(transformation))

    # Call library function.
    lib_function = lib.opendp_core__transformation_map
    lib_function.argtypes = [Transformation, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_transformation, c_distance_in), AnyObjectPtr))

    return output


def transformation_output_distance_type(
    this: Transformation
) -> str:
    r"""Get the output distance type of `transformation`.

    :param this: The transformation to retrieve the type from.
    :type this: Transformation
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__transformation_output_distance_type
    lib_function.argtypes = [Transformation]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))

    return output


def transformation_output_domain(
    this: Transformation
) -> Domain:
    r"""Get the output domain from a `transformation`.

    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__transformation_output_domain
    lib_function.argtypes = [Transformation]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_this), Domain)

    return output


def transformation_output_metric(
    this: Transformation
) -> Metric:
    r"""Get the output domain from a `transformation`.

    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :rtype: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_core__transformation_output_metric
    lib_function.argtypes = [Transformation]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_this), Metric)

    return output
