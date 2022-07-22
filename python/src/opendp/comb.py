# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.core import *

__all__ = [
    "make_chain_mt",
    "make_chain_tt",
    "make_chain_tm",
    "make_basic_composition",
    "make_population_amplification",
    "make_fix_delta",
    "make_partition_map_trans",
    "make_partition_map_meas"
]


def make_chain_mt(
    measurement: Measurement,
    transformation: Transformation
) -> Measurement:
    """Construct the functional composition (`measurement` ○ `transformation`). Returns a Measurement.
    
    :param measurement: outer privatizer
    :type measurement: Measurement
    :param transformation: inner query
    :type transformation: Transformation
    :return: Measurement representing the chained computation.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement)
    transformation = py_to_c(transformation, c_type=Transformation)
    
    # Call library function.
    function = lib.opendp_comb__make_chain_mt
    function.argtypes = [Measurement, Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, transformation), Measurement))


def make_chain_tt(
    transformation1: Transformation,
    transformation0: Transformation
) -> Transformation:
    """Construct the functional composition (`transformation1` ○ `transformation0`). Returns a Transformation.
    
    :param transformation1: outer transformation
    :type transformation1: Transformation
    :param transformation0: inner transformation
    :type transformation0: Transformation
    :return: Transformation representing the chained computation.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation1 = py_to_c(transformation1, c_type=Transformation)
    transformation0 = py_to_c(transformation0, c_type=Transformation)
    
    # Call library function.
    function = lib.opendp_comb__make_chain_tt
    function.argtypes = [Transformation, Transformation]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation1, transformation0), Transformation))


def make_chain_tm(
    transformation: Transformation,
    measurement: Measurement
) -> Measurement:
    """Construct the functional composition (`transformation` ○ `measurement`). Returns a Measurement. Used for postprocessing.
    
    :param transformation: outer postprocessor
    :type transformation: Transformation
    :param measurement: inner privatizer
    :type measurement: Measurement
    :return: Measurement representing the chained computation.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformation = py_to_c(transformation, c_type=Transformation)
    measurement = py_to_c(measurement, c_type=Measurement)
    
    # Call library function.
    function = lib.opendp_comb__make_chain_tm
    function.argtypes = [Transformation, Measurement]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformation, measurement), Measurement))


def make_basic_composition(
    measurements: Any
) -> Measurement:
    """Construct the DP composition [`measurement0`, `measurement1`, ...]. Returns a Measurement.
    
    :param measurements: A list of measurements to compose.
    :type measurements: Any
    :return: Measurement representing the composed transformations.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurements = py_to_c(measurements, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=["AnyMeasurementPtr"]))
    
    # Call library function.
    function = lib.opendp_comb__make_basic_composition
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurements), Measurement))


def make_population_amplification(
    measurement: Measurement,
    population_size: int
) -> Measurement:
    """Construct an amplified measurement from a `measurement` with privacy amplification by subsampling.
    
    :param measurement: The measurement to amplify.
    :type measurement: Measurement
    :param population_size: Number of records in population.
    :type population_size: int
    :return: New measurement with the same function, but an adjusted privacy map.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement)
    population_size = py_to_c(population_size, c_type=ctypes.c_uint)
    
    # Call library function.
    function = lib.opendp_comb__make_population_amplification
    function.argtypes = [Measurement, ctypes.c_uint]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, population_size), Measurement))


def make_fix_delta(
    measurement: Measurement,
    delta: Any
) -> Measurement:
    """Fix the delta parameter in the privacy map of a `measurement` with a SmoothedMaxDivergence output measure.
    
    :param measurement: The measurement with a privacy curve to be fixed.
    :type measurement: Measurement
    :param delta: The parameter to fix the privacy curve with.
    :type delta: Any
    :return: New measurement with the same function, but a fixed output measure and privacy map.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurement = py_to_c(measurement, c_type=Measurement)
    delta = py_to_c(delta, c_type=AnyObjectPtr, type_name=get_atom(measurement_output_distance_type(measurement)))
    
    # Call library function.
    function = lib.opendp_comb__make_fix_delta
    function.argtypes = [Measurement, AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement, delta), Measurement))


def make_partition_map_trans(
    transformations: Any
) -> Transformation:
    """Construct the parallel composition of [`transformation0`, `transformation1`, ...]. Returns a Transformation.
    
    :param transformations: A list of transformations to compose.
    :type transformations: Any
    :return: A transformation that applies a transformation to each partition.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    transformations = py_to_c(transformations, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=["AnyTransformationPtr"]))
    
    # Call library function.
    function = lib.opendp_comb__make_partition_map_trans
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(transformations), Transformation))


def make_partition_map_meas(
    measurements: Any
) -> Measurement:
    """Construct the parallel composition of [`measurement0`, `measurement1`, ...]. Returns a Measurement.
    
    :param measurements: A list of measurements to compose.
    :type measurements: Any
    :return: A measurement that applies a measurement to each partition.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    measurements = py_to_c(measurements, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=["AnyMeasurementPtr"]))
    
    # Call library function.
    function = lib.opendp_comb__make_partition_map_meas
    function.argtypes = [AnyObjectPtr]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurements), Measurement))
