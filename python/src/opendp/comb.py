# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "make_chain_mt",
    "make_chain_tt",
    "make_sequential_composition_static_distances",
    "make_population_amplification"
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


def make_sequential_composition_static_distances(
    measurement_pairs: Any,
    QO: RuntimeTypeDescriptor = "f64"
) -> Measurement:
    """Construct the DP composition [`measurement0`, `measurement1`, ...]. Returns a Measurement.
    
    :param measurement_pairs: A list of measurements to compose.
    :type measurement_pairs: Any
    :param QO: Type of output distance.
    :type QO: RuntimeTypeDescriptor
    :return: Measurement representing the composed transformations.
    :rtype: Measurement
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    QO = RuntimeType.parse(type_name=QO)
    
    # Convert arguments to c types.
    measurement_pairs = py_to_c(measurement_pairs, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[RuntimeType(origin='Tuple', args=["AnyMeasurementPtr", QO])]))
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_comb__make_sequential_composition_static_distances
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(measurement_pairs, QO), Measurement))


def make_population_amplification(
    measurement: Measurement,
    population_size: int
) -> Measurement:
    """Construct an amplified measurement from a `measurement` with privacy amplification by subsampling.
    
    :param measurement: The measurement to amplify.
    :type measurement: Measurement
    :param population_size: Number of records in population.
    :type population_size: int
    :return: New measurement with the same function, but an adjusted privacy relation.
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
