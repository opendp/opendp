# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "_error_free",
    "_function_free",
    "_measurement_free",
    "_odometer_free",
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
    "odometer_function",
    "odometer_input_domain",
    "odometer_input_metric",
    "odometer_invoke",
    "odometer_output_measure",
    "odometer_queryable_invoke",
    "odometer_queryable_map",
    "queryable_eval",
    "queryable_query_odometer_invoke_type",
    "queryable_query_odometer_map_type",
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


@versioned
def _error_free(
    this: FfiError
) -> bool:
    """Internal function. Free the memory associated with `error`.
    
    [_error_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn._error_free.html)
    
    :param this: 
    :type this: FfiError
    :return: A boolean, where true indicates successful free
    :rtype: bool
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def _function_free(
    this: Function
):
    """Internal function. Free the memory associated with `this`.
    
    [_function_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn._function_free.html)
    
    :param this: 
    :type this: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def _measurement_free(
    this: Measurement
):
    """Internal function. Free the memory associated with `this`.
    
    [_measurement_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn._measurement_free.html)
    
    :param this: 
    :type this: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def _odometer_free(
    this: Odometer
):
    """Internal function. Free the memory associated with `this`.
    
    [_odometer_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn._odometer_free.html)
    
    :param this: 
    :type this: Odometer
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this
    
    # Call library function.
    lib_function = lib.opendp_core___odometer_free
    lib_function.argtypes = [Odometer]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))
    
    return output


@versioned
def _transformation_free(
    this: Transformation
):
    """Internal function. Free the memory associated with `this`.
    
    [_transformation_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn._transformation_free.html)
    
    :param this: 
    :type this: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def function_eval(
    this: Function,
    arg: Any,
    TI: str = None
) -> Any:
    """Eval the `function` with `arg`.
    
    [function_eval in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.function_eval.html)
    
    :param this: Function to invoke.
    :type this: Function
    :param arg: Input data to supply to the measurement. A member of the measurement's input domain.
    :type arg: Any
    :param TI: Input Type.
    :type TI: str
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_check(
    measurement: Measurement,
    distance_in: Any,
    distance_out: Any
):
    """Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
    
    [measurement_check in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_check.html)
    
    :param measurement: Measurement to check the privacy relation of.
    :type measurement: Measurement
    :param distance_in: 
    :type distance_in: Any
    :param distance_out: 
    :type distance_out: Any
    :return: True indicates that the relation passed at the given distance.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_function(
    this: Measurement
) -> Function:
    """Get the function from a measurement.
    
    [measurement_function in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_function.html)
    
    :param this: The measurement to retrieve the value from.
    :type this: Measurement
    :rtype: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_input_carrier_type(
    this: Measurement
) -> str:
    """Get the input (carrier) data type of `this`.
    
    [measurement_input_carrier_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_input_carrier_type.html)
    
    :param this: The measurement to retrieve the type from.
    :type this: Measurement
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_input_distance_type(
    this: Measurement
) -> str:
    """Get the input distance type of `measurement`.
    
    [measurement_input_distance_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_input_distance_type.html)
    
    :param this: The measurement to retrieve the type from.
    :type this: Measurement
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_input_domain(
    this: Measurement
):
    """Get the input domain from a `measurement`.
    
    [measurement_input_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_input_domain.html)
    
    :param this: The measurement to retrieve the value from.
    :type this: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_input_metric(
    this: Measurement
):
    """Get the input domain from a `measurement`.
    
    [measurement_input_metric in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_input_metric.html)
    
    :param this: The measurement to retrieve the value from.
    :type this: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_invoke(
    this: Measurement,
    arg: Any
) -> Any:
    """Invoke the `measurement` with `arg`. Returns a differentially private release.
    
    [measurement_invoke in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_invoke.html)
    
    :param this: Measurement to invoke.
    :type this: Measurement
    :param arg: Input data to supply to the measurement. A member of the measurement's input domain.
    :type arg: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_map(
    measurement: Measurement,
    distance_in: Any
) -> Any:
    """Use the `measurement` to map a given `d_in` to `d_out`.
    
    [measurement_map in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_map.html)
    
    :param measurement: Measurement to check the map distances with.
    :type measurement: Measurement
    :param distance_in: Distance in terms of the input metric.
    :type distance_in: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_output_distance_type(
    this: Measurement
) -> str:
    """Get the output distance type of `measurement`.
    
    [measurement_output_distance_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_output_distance_type.html)
    
    :param this: The measurement to retrieve the type from.
    :type this: Measurement
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def measurement_output_measure(
    this: Measurement
):
    """Get the output domain from a `measurement`.
    
    [measurement_output_measure in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.measurement_output_measure.html)
    
    :param this: The measurement to retrieve the value from.
    :type this: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def odometer_function(
    this: Odometer
) -> Function:
    """Get the function from a odometer.
    
    [odometer_function in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.odometer_function.html)
    
    :param this: The odometer to retrieve the value from.
    :type this: Odometer
    :rtype: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Odometer, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_core__odometer_function
    lib_function.argtypes = [Odometer]
    lib_function.restype = FfiResult
    
    output = unwrap(lib_function(c_this), Function)
    
    return output


@versioned
def odometer_input_domain(
    this: Odometer
):
    """Get the input domain from a `odometer`.
    
    [odometer_input_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.odometer_input_domain.html)
    
    :param this: The odometer to retrieve the value from.
    :type this: Odometer
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Odometer, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_core__odometer_input_domain
    lib_function.argtypes = [Odometer]
    lib_function.restype = FfiResult
    
    output = unwrap(lib_function(c_this), Domain)
    
    return output


@versioned
def odometer_input_metric(
    this: Odometer
):
    """Get the input domain from a `odometer`.
    
    [odometer_input_metric in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.odometer_input_metric.html)
    
    :param this: The odometer to retrieve the value from.
    :type this: Odometer
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Odometer, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_core__odometer_input_metric
    lib_function.argtypes = [Odometer]
    lib_function.restype = FfiResult
    
    output = unwrap(lib_function(c_this), Metric)
    
    return output


@versioned
def odometer_invoke(
    this: Odometer,
    arg: Any
) -> Any:
    """Invoke the `odometer` with `arg`. Returns a differentially private release.
    
    [odometer_invoke in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.odometer_invoke.html)
    
    :param this: Odometer to invoke.
    :type this: Odometer
    :param arg: Input data to supply to the odometer. A member of the odometer's input domain.
    :type arg: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Odometer, type_name=None)
    c_arg = py_to_c(arg, c_type=AnyObjectPtr, type_name=get_carrier_type(get_input_domain(this)))
    
    # Call library function.
    lib_function = lib.opendp_core__odometer_invoke
    lib_function.argtypes = [Odometer, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this, c_arg), AnyObjectPtr))
    
    return output


@versioned
def odometer_output_measure(
    this: Odometer
):
    """Get the output domain from a `odometer`.
    
    [odometer_output_measure in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.odometer_output_measure.html)
    
    :param this: The odometer to retrieve the value from.
    :type this: Odometer
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Odometer, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_core__odometer_output_measure
    lib_function.argtypes = [Odometer]
    lib_function.restype = FfiResult
    
    output = unwrap(lib_function(c_this), Measure)
    
    return output


@versioned
def odometer_queryable_invoke(
    queryable: Any,
    query: Any
) -> Any:
    """Eval the odometer `queryable` with an invoke `query`. Returns a differentially private release.
    
    [odometer_queryable_invoke in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.odometer_queryable_invoke.html)
    
    :param queryable: Queryable to eval.
    :type queryable: Any
    :param query: Invoke query to supply to the queryable.
    :type query: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_queryable = py_to_c(queryable, c_type=AnyObjectPtr, type_name=None)
    c_query = py_to_c(query, c_type=AnyObjectPtr, type_name=queryable_query_odometer_invoke_type(queryable))
    
    # Call library function.
    lib_function = lib.opendp_core__odometer_queryable_invoke
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_queryable, c_query), AnyObjectPtr))
    
    return output


@versioned
def odometer_queryable_map(
    queryable: Any,
    query: Any
) -> Any:
    """Eval the odometer `queryable` with a map `query`. Returns the current d_out.
    
    [odometer_queryable_map in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.odometer_queryable_map.html)
    
    :param queryable: Queryable to eval.
    :type queryable: Any
    :param query: Map query to supply to the queryable.
    :type query: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_queryable = py_to_c(queryable, c_type=AnyObjectPtr, type_name=None)
    c_query = py_to_c(query, c_type=AnyObjectPtr, type_name=queryable_query_odometer_map_type(queryable))
    
    # Call library function.
    lib_function = lib.opendp_core__odometer_queryable_map
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_queryable, c_query), AnyObjectPtr))
    
    return output


@versioned
def queryable_eval(
    queryable: Any,
    query: Any
) -> Any:
    """Eval the `queryable` with `query`. Returns a differentially private release.
    
    [queryable_eval in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.queryable_eval.html)
    
    :param queryable: Queryable to eval.
    :type queryable: Any
    :param query: The input to the queryable.
    :type query: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def queryable_query_odometer_invoke_type(
    this: Any
) -> str:
    """Get the query odometer invoke type of `queryable`.
    
    [queryable_query_odometer_invoke_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.queryable_query_odometer_invoke_type.html)
    
    :param this: The queryable to retrieve the type from.
    :type this: Any
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=AnyObjectPtr, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_core__queryable_query_odometer_invoke_type
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def queryable_query_odometer_map_type(
    this: Any
) -> str:
    """Get the query odometer map type of `queryable`.
    
    [queryable_query_odometer_map_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.queryable_query_odometer_map_type.html)
    
    :param this: The queryable to retrieve the type from.
    :type this: Any
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=AnyObjectPtr, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_core__queryable_query_odometer_map_type
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def queryable_query_type(
    this: Any
) -> str:
    """Get the query type of `queryable`.
    
    [queryable_query_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.queryable_query_type.html)
    
    :param this: The queryable to retrieve the query type from.
    :type this: Any
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_check(
    transformation: Transformation,
    distance_in: Any,
    distance_out: Any
):
    """Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
    
    [transformation_check in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_check.html)
    
    :param transformation: 
    :type transformation: Transformation
    :param distance_in: 
    :type distance_in: Any
    :param distance_out: 
    :type distance_out: Any
    :return: True indicates that the relation passed at the given distance.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_function(
    this: Transformation
) -> Function:
    """Get the function from a transformation.
    
    [transformation_function in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_function.html)
    
    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :rtype: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_input_carrier_type(
    this: Transformation
) -> str:
    """Get the input (carrier) data type of `this`.
    
    [transformation_input_carrier_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_input_carrier_type.html)
    
    :param this: The transformation to retrieve the type from.
    :type this: Transformation
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_input_distance_type(
    this: Transformation
) -> str:
    """Get the input distance type of `transformation`.
    
    [transformation_input_distance_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_input_distance_type.html)
    
    :param this: The transformation to retrieve the type from.
    :type this: Transformation
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_input_domain(
    this: Transformation
):
    """Get the input domain from a `transformation`.
    
    [transformation_input_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_input_domain.html)
    
    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_input_metric(
    this: Transformation
):
    """Get the input domain from a `transformation`.
    
    [transformation_input_metric in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_input_metric.html)
    
    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_invoke(
    this: Transformation,
    arg: Any
) -> Any:
    """Invoke the `transformation` with `arg`. Returns a differentially private release.
    
    [transformation_invoke in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_invoke.html)
    
    :param this: Transformation to invoke.
    :type this: Transformation
    :param arg: Input data to supply to the transformation. A member of the transformation's input domain.
    :type arg: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_map(
    transformation: Transformation,
    distance_in: Any
) -> Any:
    """Use the `transformation` to map a given `d_in` to `d_out`.
    
    [transformation_map in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_map.html)
    
    :param transformation: Transformation to check the map distances with.
    :type transformation: Transformation
    :param distance_in: Distance in terms of the input metric.
    :type distance_in: Any
    :rtype: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_output_distance_type(
    this: Transformation
) -> str:
    """Get the output distance type of `transformation`.
    
    [transformation_output_distance_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_output_distance_type.html)
    
    :param this: The transformation to retrieve the type from.
    :type this: Transformation
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_output_domain(
    this: Transformation
):
    """Get the output domain from a `transformation`.
    
    [transformation_output_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_output_domain.html)
    
    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def transformation_output_metric(
    this: Transformation
):
    """Get the output domain from a `transformation`.
    
    [transformation_output_metric in Rust documentation.](https://docs.rs/opendp/latest/opendp/core/fn.transformation_output_metric.html)
    
    :param this: The transformation to retrieve the value from.
    :type this: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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
