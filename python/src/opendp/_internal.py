# Auto-generated. Do not edit!
'''
The ``internal`` module provides functions that can be used to construct library primitives without the use of the "honest-but-curious" flag.
'''
from deprecated.sphinx import deprecated # noqa: F401 (Not every file actually has deprecated functions.)

from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.typing import _substitute # noqa: F401
from opendp.core import *
from opendp.domains import *
from opendp.metrics import *
from opendp.measures import *
__all__ = [
    "_extrinsic_distance",
    "_extrinsic_divergence",
    "_extrinsic_domain",
    "_make_measurement",
    "_make_transformation",
    "_new_pure_function"
]


def _extrinsic_distance(
    identifier: str,
    descriptor = None
) -> Metric:
    r"""Construct a new ExtrinsicDistance.
    This is meant for internal use, as it does not require "honest-but-curious",
    unlike `user_distance`.

    See `user_distance` for correct usage of this function.

    .. end-markdown

    :param identifier: A string description of the metric.
    :type identifier: str
    :param descriptor: Additional constraints on the domain.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_identifier = py_to_c(identifier, c_type=ctypes.c_char_p, type_name=None)
    c_descriptor = py_to_c(descriptor, c_type=ExtrinsicObjectPtr, type_name="ExtrinsicObject")

    # Call library function.
    lib_function = lib.opendp_internal___extrinsic_distance
    lib_function.argtypes = [ctypes.c_char_p, ExtrinsicObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_identifier, c_descriptor), Metric))
    try:
        output.__opendp_dict__ = {
            '__function__': '_extrinsic_distance',
            '__module__': 'internal',
            '__kwargs__': {
                'identifier': identifier, 'descriptor': descriptor
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _extrinsic_divergence(
    descriptor: str
) -> Measure:
    r"""Construct a new ExtrinsicDivergence, a privacy measure defined from a bindings language.
    This is meant for internal use, as it does not require "honest-but-curious",
    unlike `user_divergence`.

    See `user_divergence` for correct usage and proof definition for this function.

    .. end-markdown

    :param descriptor: A string description of the privacy measure.
    :type descriptor: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_descriptor = py_to_c(descriptor, c_type=ctypes.c_char_p, type_name="String")

    # Call library function.
    lib_function = lib.opendp_internal___extrinsic_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_descriptor), Measure))
    try:
        output.__opendp_dict__ = {
            '__function__': '_extrinsic_divergence',
            '__module__': 'internal',
            '__kwargs__': {
                'descriptor': descriptor
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _extrinsic_domain(
    identifier: str,
    member,
    descriptor = None
) -> Domain:
    r"""Construct a new ExtrinsicDomain.
    This is meant for internal use, as it does not require "honest-but-curious",
    unlike `user_domain`.

    See `user_domain` for correct usage and proof definition for this function.

    .. end-markdown

    :param identifier: A string description of the data domain.
    :type identifier: str
    :param member: A function used to test if a value is a member of the data domain.
    :param descriptor: Additional constraints on the domain.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_identifier = py_to_c(identifier, c_type=ctypes.c_char_p, type_name=None)
    c_member = py_to_c(member, c_type=CallbackFnPtr, type_name="bool")
    c_descriptor = py_to_c(descriptor, c_type=ExtrinsicObjectPtr, type_name="ExtrinsicObject")

    # Call library function.
    lib_function = lib.opendp_internal___extrinsic_domain
    lib_function.argtypes = [ctypes.c_char_p, CallbackFnPtr, ExtrinsicObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_identifier, c_member, c_descriptor), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': '_extrinsic_domain',
            '__module__': 'internal',
            '__kwargs__': {
                'identifier': identifier, 'member': member, 'descriptor': descriptor
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _make_measurement(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    function,
    privacy_map,
    TO: RuntimeTypeDescriptor = "ExtrinsicObject"
) -> Measurement:
    r"""Construct a Measurement from user-defined callbacks.
    This is meant for internal use, as it does not require "honest-but-curious",
    unlike `make_user_measurement`.

    See `make_user_measurement` for correct usage and proof definition for this function.

    **Supporting Elements:**

    * Input Domain:   `AnyDomain`
    * Output Type:    `AnyMetric`
    * Input Metric:   `AnyMeasure`
    * Output Measure: `AnyObject`

    .. end-markdown

    :param input_domain: A domain describing the set of valid inputs for the function.
    :type input_domain: Domain
    :param input_metric: The metric from which distances between adjacent inputs are measured.
    :type input_metric: Metric
    :param output_measure: The measure from which distances between adjacent output distributions are measured.
    :type output_measure: Measure
    :param function: A function mapping data from ``input_domain`` to a release of type ``TO``.
    :param privacy_map: A function mapping distances from ``input_metric`` to ``output_measure``.
    :param TO: The data type of outputs from the function.
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    TO = RuntimeType.parse(type_name=TO)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name="AnyDomain")
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name="AnyMetric")
    c_output_measure = py_to_c(output_measure, c_type=Measure, type_name="AnyMeasure")
    c_function = py_to_c(function, c_type=CallbackFnPtr, type_name=pass_through(TO))
    c_privacy_map = py_to_c(privacy_map, c_type=CallbackFnPtr, type_name=measure_distance_type(output_measure))
    c_TO = py_to_c(TO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_internal___make_measurement
    lib_function.argtypes = [Domain, Metric, Measure, CallbackFnPtr, CallbackFnPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_output_measure, c_function, c_privacy_map, c_TO), Measurement))
    try:
        output.__opendp_dict__ = {
            '__function__': '_make_measurement',
            '__module__': 'internal',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'output_measure': output_measure, 'function': function, 'privacy_map': privacy_map, 'TO': TO
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _make_transformation(
    input_domain: Domain,
    input_metric: Metric,
    output_domain: Domain,
    output_metric: Metric,
    function,
    stability_map
) -> Transformation:
    r"""Construct a Transformation from user-defined callbacks.
    This is meant for internal use, as it does not require "honest-but-curious",
    unlike `make_user_transformation`.

    See `make_user_transformation` for correct usage and proof definition for this function.

    .. end-markdown

    :param input_domain: A domain describing the set of valid inputs for the function.
    :type input_domain: Domain
    :param input_metric: The metric from which distances between adjacent inputs are measured.
    :type input_metric: Metric
    :param output_domain: A domain describing the set of valid outputs of the function.
    :type output_domain: Domain
    :param output_metric: The metric from which distances between outputs of adjacent inputs are measured.
    :type output_metric: Metric
    :param function: A function mapping data from ``input_domain`` to ``output_domain``.
    :param stability_map: A function mapping distances from ``input_metric`` to ``output_metric``.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name="AnyDomain")
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name="AnyMetric")
    c_output_domain = py_to_c(output_domain, c_type=Domain, type_name="AnyDomain")
    c_output_metric = py_to_c(output_metric, c_type=Metric, type_name="AnyMetric")
    c_function = py_to_c(function, c_type=CallbackFnPtr, type_name=domain_carrier_type(output_domain))
    c_stability_map = py_to_c(stability_map, c_type=CallbackFnPtr, type_name=metric_distance_type(output_metric))

    # Call library function.
    lib_function = lib.opendp_internal___make_transformation
    lib_function.argtypes = [Domain, Metric, Domain, Metric, CallbackFnPtr, CallbackFnPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_output_domain, c_output_metric, c_function, c_stability_map), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': '_make_transformation',
            '__module__': 'internal',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'output_domain': output_domain, 'output_metric': output_metric, 'function': function, 'stability_map': stability_map
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _new_pure_function(
    function,
    TO: RuntimeTypeDescriptor = "ExtrinsicObject"
) -> Function:
    r"""Construct a Function from a user-defined callback.
    This is meant for internal use, as it does not require "honest-but-curious",
    unlike `new_function`.

    See `new_function` for correct usage and proof definition for this function.


    Required features: `contrib`

    [_new_pure_function in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/internal/fn._new_pure_function.html)

    .. end-markdown

    :param function: A function mapping data to a value of type ``TO``
    :param TO: Output Type
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TO = RuntimeType.parse(type_name=TO)

    # Convert arguments to c types.
    c_function = py_to_c(function, c_type=CallbackFnPtr, type_name=pass_through(TO))
    c_TO = py_to_c(TO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_internal___new_pure_function
    lib_function.argtypes = [CallbackFnPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_function, c_TO), Function))
    try:
        output.__opendp_dict__ = {
            '__function__': '_new_pure_function',
            '__module__': 'internal',
            '__kwargs__': {
                'function': function, 'TO': TO
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output
