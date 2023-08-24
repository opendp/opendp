# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.core import *
from opendp.domains import *
from opendp.metrics import *
from opendp.measures import *
__all__ = [
    "make_basic_composition",
    "make_chain_mt",
    "make_chain_pm",
    "make_chain_tt",
    "make_fix_delta",
    "make_population_amplification",
    "make_pureDP_to_fixed_approxDP",
    "make_pureDP_to_zCDP",
    "make_sequential_composition",
    "make_zCDP_to_approxDP"
]


@versioned
def make_basic_composition(
    measurements: Any
) -> Measurement:
    """Construct the DP composition [`measurement0`, `measurement1`, ...].
    Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
    
    All metrics and domains must be equivalent, except for the output domain.
    
    [make_basic_composition in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_basic_composition.html)
    
    :param measurements: A vector of Measurements to compose.
    :type measurements: Any
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurements = py_to_c(measurements, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[AnyMeasurementPtr]))
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_basic_composition
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_measurements), Measurement))
    output._depends_on(get_dependencies_iterable(measurements))
    return output


@versioned
def make_chain_mt(
    measurement1: Measurement,
    transformation0: Transformation
) -> Measurement:
    """Construct the functional composition (`measurement1` ○ `transformation0`).
    Returns a Measurement that when invoked, computes `measurement1(transformation0(x))`.
    
    [make_chain_mt in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_chain_mt.html)
    
    :param measurement1: outer mechanism
    :type measurement1: Measurement
    :param transformation0: inner transformation
    :type transformation0: Transformation
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement1 = py_to_c(measurement1, c_type=Measurement, type_name=None)
    c_transformation0 = py_to_c(transformation0, c_type=Transformation, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_chain_mt
    lib_function.argtypes = [Measurement, Transformation]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_measurement1, c_transformation0), Measurement))
    output._depends_on(get_dependencies(measurement1), get_dependencies(transformation0))
    return output


@versioned
def make_chain_pm(
    postprocess1: Function,
    measurement0: Measurement
) -> Measurement:
    """Construct the functional composition (`postprocess1` ○ `measurement0`).
    Returns a Measurement that when invoked, computes `postprocess1(measurement0(x))`.
    Used to represent non-interactive postprocessing.
    
    [make_chain_pm in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_chain_pm.html)
    
    :param postprocess1: outer postprocessor
    :type postprocess1: Function
    :param measurement0: inner measurement/mechanism
    :type measurement0: Measurement
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_postprocess1 = py_to_c(postprocess1, c_type=Function, type_name=None)
    c_measurement0 = py_to_c(measurement0, c_type=Measurement, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_chain_pm
    lib_function.argtypes = [Function, Measurement]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_postprocess1, c_measurement0), Measurement))
    output._depends_on(get_dependencies(postprocess1), get_dependencies(measurement0))
    return output


@versioned
def make_chain_tt(
    transformation1: Transformation,
    transformation0: Transformation
) -> Transformation:
    """Construct the functional composition (`transformation1` ○ `transformation0`).
    Returns a Transformation that when invoked, computes `transformation1(transformation0(x))`.
    
    [make_chain_tt in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_chain_tt.html)
    
    :param transformation1: outer transformation
    :type transformation1: Transformation
    :param transformation0: inner transformation
    :type transformation0: Transformation
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_transformation1 = py_to_c(transformation1, c_type=Transformation, type_name=None)
    c_transformation0 = py_to_c(transformation0, c_type=Transformation, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_chain_tt
    lib_function.argtypes = [Transformation, Transformation]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_transformation1, c_transformation0), Transformation))
    output._depends_on(get_dependencies(transformation1), get_dependencies(transformation0))
    return output


@versioned
def make_fix_delta(
    measurement: Measurement,
    delta: Any
) -> Measurement:
    """Fix the delta parameter in the privacy map of a `measurement` with a SmoothedMaxDivergence output measure.
    
    [make_fix_delta in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_fix_delta.html)
    
    :param measurement: a measurement with a privacy curve to be fixed
    :type measurement: Measurement
    :param delta: parameter to fix the privacy curve with
    :type delta: Any
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=None)
    c_delta = py_to_c(delta, c_type=AnyObjectPtr, type_name=get_atom(measurement_output_distance_type(measurement)))
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_fix_delta
    lib_function.argtypes = [Measurement, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_measurement, c_delta), Measurement))
    output._depends_on(get_dependencies(measurement))
    return output


@versioned
def make_population_amplification(
    measurement: Measurement,
    population_size: int
) -> Measurement:
    """Construct an amplified measurement from a `measurement` with privacy amplification by subsampling.
    This measurement does not perform any sampling.
    It is useful when you have a dataset on-hand that is a simple random sample from a larger population.
    
    The DIA, DO, MI and MO between the input measurement and amplified output measurement all match.
    
    Protected by the "honest-but-curious" feature flag
    because a dishonest adversary could set the population size to be arbitrarily large.
    
    [make_population_amplification in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_population_amplification.html)
    
    :param measurement: the computation to amplify
    :type measurement: Measurement
    :param population_size: the size of the population from which the input dataset is a simple sample
    :type population_size: int
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib", "honest-but-curious")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=AnyMeasurement)
    c_population_size = py_to_c(population_size, c_type=ctypes.c_size_t, type_name=usize)
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_population_amplification
    lib_function.argtypes = [Measurement, ctypes.c_size_t]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_measurement, c_population_size), Measurement))
    output._depends_on(get_dependencies(measurement))
    return output


@versioned
def make_pureDP_to_fixed_approxDP(
    measurement: Measurement
) -> Measurement:
    """Constructs a new output measurement where the output measure
    is casted from `MaxDivergence<QO>` to `FixedSmoothedMaxDivergence<QO>`.
    
    [make_pureDP_to_fixed_approxDP in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_pureDP_to_fixed_approxDP.html)
    
    :param measurement: a measurement with a privacy measure to be casted
    :type measurement: Measurement
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=AnyMeasurement)
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_pureDP_to_fixed_approxDP
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_measurement), Measurement))
    
    return output


@versioned
def make_pureDP_to_zCDP(
    measurement: Measurement
) -> Measurement:
    """Constructs a new output measurement where the output measure
    is casted from `MaxDivergence<QO>` to `ZeroConcentratedDivergence<QO>`.
    
    [make_pureDP_to_zCDP in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_pureDP_to_zCDP.html)
    
    **Citations:**
    
    - [BS16 Concentrated Differential Privacy: Simplifications, Extensions, and Lower Bounds](https://arxiv.org/pdf/1605.02065.pdf#subsection.3.1)
    
    :param measurement: a measurement with a privacy measure to be casted
    :type measurement: Measurement
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=AnyMeasurement)
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_pureDP_to_zCDP
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_measurement), Measurement))
    
    return output


@versioned
def make_sequential_composition(
    input_domain,
    input_metric,
    output_measure,
    d_in: Any,
    d_mids: Any
) -> Measurement:
    """Construct a queryable that interactively composes interactive measurements.
    
    [make_sequential_composition in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_sequential_composition.html)
    
    :param input_domain: indicates the space of valid input datasets
    :param input_metric: how distances are measured between members of the input domain
    :param output_measure: how privacy is measured
    :param d_in: maximum distance between adjacent input datasets
    :type d_in: Any
    :param d_mids: maximum privacy expenditure of each query
    :type d_mids: Any
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    QO = get_distance_type(output_measure)
    
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=AnyDomain)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=AnyMetric)
    c_output_measure = py_to_c(output_measure, c_type=Measure, type_name=AnyMeasure)
    c_d_in = py_to_c(d_in, c_type=AnyObjectPtr, type_name=get_distance_type(input_metric))
    c_d_mids = py_to_c(d_mids, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[QO]))
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_sequential_composition
    lib_function.argtypes = [Domain, Metric, Measure, AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_output_measure, c_d_in, c_d_mids), Measurement))
    
    return output


@versioned
def make_zCDP_to_approxDP(
    measurement: Measurement
) -> Measurement:
    """Constructs a new output measurement where the output measure
    is casted from `ZeroConcentratedDivergence<QO>` to `SmoothedMaxDivergence<QO>`.
    
    [make_zCDP_to_approxDP in Rust documentation.](https://docs.rs/opendp/latest/opendp/combinators/fn.make_zCDP_to_approxDP.html)
    
    :param measurement: a measurement with a privacy measure to be casted
    :type measurement: Measurement
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=AnyMeasurement)
    
    # Call library function.
    lib_function = lib.opendp_combinators__make_zCDP_to_approxDP
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_measurement), Measurement))
    output._depends_on(get_dependencies(measurement))
    return output
