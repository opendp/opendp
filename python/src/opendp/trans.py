# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "make_cast",
    "make_cast_default",
    "make_is_equal",
    "make_is_null",
    "make_cast_inherent",
    "make_ordered_random",
    "make_sized_ordered_random",
    "make_sized_bounded_ordered_random",
    "make_unordered",
    "make_sized_unordered",
    "make_sized_bounded_unordered",
    "make_metric_bounded",
    "make_metric_unbounded",
    "make_clamp",
    "make_unclamp",
    "make_count",
    "make_count_distinct",
    "make_count_by",
    "make_count_by_categories",
    "make_split_lines",
    "make_split_records",
    "make_create_dataframe",
    "make_split_dataframe",
    "make_select_column",
    "make_identity",
    "make_impute_constant",
    "make_drop_null",
    "make_impute_uniform_float",
    "make_find",
    "make_find_bin",
    "make_index",
    "make_lipschitz_float_mul",
    "make_sized_bounded_mean",
    "make_resize",
    "make_bounded_resize",
    "make_bounded_sum",
    "make_sized_bounded_sum",
    "make_bounded_float_checked_sum",
    "make_sized_bounded_float_checked_sum",
    "make_bounded_float_ordered_sum",
    "make_sized_bounded_float_ordered_sum",
    "make_sized_bounded_int_checked_sum",
    "make_bounded_int_monotonic_sum",
    "make_sized_bounded_int_monotonic_sum",
    "make_bounded_int_ordered_sum",
    "make_sized_bounded_int_ordered_sum",
    "make_bounded_int_split_sum",
    "make_sized_bounded_int_split_sum",
    "make_sized_bounded_sum_of_squared_deviations",
    "make_sized_bounded_variance"
]


def make_cast(
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TIA` to type `TOA`. 
    Failure to parse results in None, else Some<TOA>.
    
    :param TIA: atomic input data type to cast from
    :type TIA: :ref:`RuntimeTypeDescriptor`
    :param TOA: atomic data type to cast into
    :type TOA: :ref:`RuntimeTypeDescriptor`
    :return: A cast step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_cast
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TIA, TOA), Transformation))


def make_cast_default(
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TIA` to type `TOA`. If cast fails, fill with default.
    
    :param TIA: atomic input data type to cast from
    :type TIA: :ref:`RuntimeTypeDescriptor`
    :param TOA: atomic data type to cast into
    :type TOA: :ref:`RuntimeTypeDescriptor`
    :return: A cast_default step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_cast_default
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TIA, TOA), Transformation))


def make_is_equal(
    value: Any,
    TIA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that checks if each element is equal to `value`.
    
    :param value: value to check against
    :type value: Any
    :param TIA: atomic input data type
    :type TIA: :ref:`RuntimeTypeDescriptor`
    :return: A is_equal step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=value)
    
    # Convert arguments to c types.
    value = py_to_c(value, c_type=AnyObjectPtr, type_name=TIA)
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_is_equal
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(value, TIA), Transformation))


def make_is_null(
    DIA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that checks if each element in a vector is null.
    
    :param DIA: atomic input domain
    :type DIA: :ref:`RuntimeTypeDescriptor`
    :return: A is_null step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    DIA = RuntimeType.parse(type_name=DIA)
    
    # Convert arguments to c types.
    DIA = py_to_c(DIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_is_null
    function.argtypes = [ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(DIA), Transformation))


def make_cast_inherent(
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TI` to a type that can represent nullity `TO`. 
    If cast fails, fill with `TO`'s null value.
    
    :param TIA: input data type to cast from
    :type TIA: :ref:`RuntimeTypeDescriptor`
    :param TOA: data type to cast into
    :type TOA: :ref:`RuntimeTypeDescriptor`
    :return: A cast_inherent step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_cast_inherent
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TIA, TOA), Transformation))


def make_ordered_random(
    TA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that converts the unordered dataset metric `SymmetricDistance` to the respective ordered dataset metric InsertDeleteDistance by assigning a random permutatation. 
    Operates exclusively on VectorDomain<AllDomain<`TA`>>.
    
    :param TA: atomic type of data
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A ordered_random step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_ordered_random
    function.argtypes = [ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TA), Transformation))


def make_sized_ordered_random(
    size: int,
    TA: RuntimeTypeDescriptor,
    MI: DatasetMetric = "SymmetricDistance"
) -> Transformation:
    """Make a Transformation that converts the unordered dataset metric `MI` to the respective ordered dataset metric by assigning a random permutatation. 
    Operates exclusively on SizedDomain<VectorDomain<AllDomain<`TA`>>>. 
    If `MI` is "SymmetricDistance", then output metric is "InsertDeleteDistance", and respectively "ChangeOneDistance" maps to "HammingDistance".
    
    :param size: Number of records in input data.
    :type size: int
    :param MI: input dataset metric
    :type MI: DatasetMetric
    :param TA: atomic type of data
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A sized_ordered_random step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_ordered_random
    function.argtypes = [ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, MI, TA), Transformation))


def make_sized_bounded_ordered_random(
    size: int,
    bounds: Tuple[Any, Any],
    TA: RuntimeTypeDescriptor,
    MI: DatasetMetric = "SymmetricDistance"
) -> Transformation:
    """Make a Transformation that converts the unordered dataset metric `MI` to the respective ordered dataset metric by assigning a random permutatation. 
    Operates exclusively on SizedDomain<VectorDomain<BoundedDomain<`TA`>>>. 
    If `MI` is "SymmetricDistance", then output metric is "InsertDeleteDistance", and respectively "ChangeOneDistance" maps to "HammingDistance".
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: Tuple[Any, Any]
    :param MI: input dataset metric
    :type MI: DatasetMetric
    :param TA: atomic type of data
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_ordered_random step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_ordered_random
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, MI, TA), Transformation))


def make_unordered(
    TA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that converts the ordered dataset metric `InsertDeleteDistance` to the respective unordered dataset metric SymmetricDistance with a no-op. 
    Operates exclusively on VectorDomain<AllDomain<`TA`>>. 
    If `MI` is "InsertDeleteDistance", then output metric is "SymmetricDistance", and respectively "HammingDistance" maps to "ChangeOneDistance".
    
    :param TA: atomic type of data
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A unordered step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_unordered
    function.argtypes = [ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TA), Transformation))


def make_sized_unordered(
    size: int,
    TA: RuntimeTypeDescriptor,
    MI: DatasetMetric = "InsertDeleteDistance"
) -> Transformation:
    """Make a Transformation that converts the ordered dataset metric `MI` to the respective unordered dataset metric with a no-op. 
    Operates exclusively on SizedDomain<VectorDomain<AllDomain<`TA`>>>. 
    If `MI` is "InsertDeleteDistance", then output metric is "SymmetricDistance", and respectively "HammingDistance" maps to "ChangeOneDistance".
    
    :param size: Number of records in input data.
    :type size: int
    :param MI: input dataset metric.
    :type MI: DatasetMetric
    :param TA: atomic type of data
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A sized_unordered step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_unordered
    function.argtypes = [ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, MI, TA), Transformation))


def make_sized_bounded_unordered(
    size: int,
    bounds: Tuple[Any, Any],
    TA: RuntimeTypeDescriptor,
    MI: DatasetMetric = "InsertDeleteDistance"
) -> Transformation:
    """Make a Transformation that converts the ordered dataset metric `MI` to the respective unordered dataset metric with a no-op. 
    Operates exclusively on SizedDomain<VectorDomain<BoundedDomain<`TA`>>>. 
    If `MI` is "InsertDeleteDistance", then output metric is "SymmetricDistance", and respectively "HammingDistance" maps to "ChangeOneDistance".
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: Tuple[Any, Any]
    :param MI: input dataset metric
    :type MI: DatasetMetric
    :param TA: atomic type of data
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_unordered step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_unordered
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, MI, TA), Transformation))


def make_metric_bounded(
    size: int,
    TA: RuntimeTypeDescriptor,
    MI: DatasetMetric = "SymmetricDistance"
) -> Transformation:
    """Make a Transformation that converts the unbounded dataset metric `MI` to the respective bounded dataset metric with a no-op. 
    If "SymmetricDistance", then output metric is "ChangeOneDistance", and respectively "InsertDeleteDistance" maps to "HammingDistance".
    
    :param size: Number of records in input data.
    :type size: int
    :param MI: input dataset metric.
    :type MI: DatasetMetric
    :param TA: atomic type of data
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A metric_bounded step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_metric_bounded
    function.argtypes = [ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, MI, TA), Transformation))


def make_metric_unbounded(
    size: int,
    TA: RuntimeTypeDescriptor,
    MI: DatasetMetric = "ChangeOneDistance"
) -> Transformation:
    """Make a Transformation that converts the bounded dataset metric `MI` to the respective unbounded dataset metric with a no-op. 
    If "ChangeOneDistance", then output metric is "SymmetricDistance", and respectively "HammingDistance" maps to "InsertDeleteDistance".
    
    :param size: Number of records in input data.
    :type size: int
    :param MI: input dataset metric.
    :type MI: DatasetMetric
    :param TA: atomic type of data
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A metric_unbounded step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_metric_unbounded
    function.argtypes = [ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, MI, TA), Transformation))


def make_clamp(
    bounds: Tuple[Any, Any],
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that clamps numeric data in Vec<`T`> to `bounds`. 
    If datum is less than lower, let datum be lower. 
    If datum is greater than upper, let datum be upper.
    
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: Tuple[Any, Any]
    :param TA: atomic data type
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A clamp step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_clamp
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(bounds, TA), Transformation))


def make_unclamp(
    bounds: Tuple[Any, Any],
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that unclamps a VectorDomain<BoundedDomain<T>> to a VectorDomain<AllDomain<T>>.
    
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: Tuple[Any, Any]
    :param TA: atomic data type
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A unclamp step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_unclamp
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(bounds, TA), Transformation))


def make_count(
    TIA: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor = "int"
) -> Transformation:
    """Make a Transformation that computes a count of the number of records in data.
    
    :param TIA: Atomic Input Type. Input data is expected to be of the form Vec<TIA>.
    :type TIA: :ref:`RuntimeTypeDescriptor`
    :param TO: Output Type. Must be numeric.
    :type TO: :ref:`RuntimeTypeDescriptor`
    :return: A count step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TO = RuntimeType.parse(type_name=TO)
    
    # Convert arguments to c types.
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_count
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TIA, TO), Transformation))


def make_count_distinct(
    TIA: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor = "int"
) -> Transformation:
    """Make a Transformation that computes a count of the number of unique, distinct records in data.
    
    :param TIA: Atomic Input Type. Input data is expected to be of the form Vec<TIA>.
    :type TIA: :ref:`RuntimeTypeDescriptor`
    :param TO: Output Type. Must be numeric.
    :type TO: :ref:`RuntimeTypeDescriptor`
    :return: A count_distinct step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TO = RuntimeType.parse(type_name=TO)
    
    # Convert arguments to c types.
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_count_distinct
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TIA, TO), Transformation))


def make_count_by(
    MO: SensitivityMetric,
    TK: RuntimeTypeDescriptor,
    TV: RuntimeTypeDescriptor = "int"
) -> Transformation:
    """Make a Transformation that computes the count of each unique value in data. 
    This assumes that the category set is unknown.
    
    :param MO: Output Metric.
    :type MO: SensitivityMetric
    :param TK: Type of Key. Categorical/hashable input data type. Input data must be Vec<TK>.
    :type TK: :ref:`RuntimeTypeDescriptor`
    :param TV: Type of Value. Express counts in terms of this integral type.
    :type TV: :ref:`RuntimeTypeDescriptor`
    :return: The carrier type is HashMap<TK, TV>, a hashmap of the count (TV) for each unique data input (TK).
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)
    TK = RuntimeType.parse(type_name=TK)
    TV = RuntimeType.parse(type_name=TV)
    
    # Convert arguments to c types.
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TK = py_to_c(TK, c_type=ctypes.c_char_p)
    TV = py_to_c(TV, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_count_by
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(MO, TK, TV), Transformation))


def make_count_by_categories(
    categories: Any,
    MO: SensitivityMetric = "L1Distance<int>",
    TIA: RuntimeTypeDescriptor = None,
    TOA: RuntimeTypeDescriptor = "int"
) -> Transformation:
    """Make a Transformation that computes the number of times each category appears in the data. 
    This assumes that the category set is known.
    
    :param categories: The set of categories to compute counts for.
    :type categories: Any
    :param MO: output sensitivity metric
    :type MO: SensitivityMetric
    :param TIA: categorical/hashable input type. Input data must be Vec<TIA>.
    :type TIA: :ref:`RuntimeTypeDescriptor`
    :param TOA: express counts in terms of this numeric type
    :type TOA: :ref:`RuntimeTypeDescriptor`
    :return: A count_by_categories step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=next(iter(categories), None))
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_count_by_categories
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(categories, MO, TIA, TOA), Transformation))


def make_split_lines(
    
) -> Transformation:
    """Make a Transformation that takes a string and splits it into a Vec<String> of its lines.
    
    
    :return: A split_lines step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    function = lib.opendp_trans__make_split_lines
    function.argtypes = []
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(), Transformation))


def make_split_records(
    separator: str
) -> Transformation:
    """Make a Transformation that splits each record in a Vec<String> into a Vec<Vec<String>>.
    
    :param separator: The token(s) that separate entries in each record.
    :type separator: str
    :return: A split_records step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    separator = py_to_c(separator, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_split_records
    function.argtypes = [ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(separator), Transformation))


def make_create_dataframe(
    col_names: Any,
    K: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that constructs a dataframe from a Vec<Vec<String>>.
    
    :param col_names: Column names for each record entry.
    :type col_names: Any
    :param K: categorical/hashable data type of column names
    :type K: :ref:`RuntimeTypeDescriptor`
    :return: A create_dataframe step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    K = RuntimeType.parse_or_infer(type_name=K, public_example=next(iter(col_names), None))
    
    # Convert arguments to c types.
    col_names = py_to_c(col_names, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[K]))
    K = py_to_c(K, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_create_dataframe
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(col_names, K), Transformation))


def make_split_dataframe(
    separator: str,
    col_names: Any,
    K: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that splits each record in a String into a Vec<Vec<String>>,
    and loads the resulting table into a dataframe keyed by `col_names`.
    
    :param separator: The token(s) that separate entries in each record.
    :type separator: str
    :param col_names: Column names for each record entry.
    :type col_names: Any
    :param K: categorical/hashable data type of column names
    :type K: :ref:`RuntimeTypeDescriptor`
    :return: A split_dataframe step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    K = RuntimeType.parse_or_infer(type_name=K, public_example=next(iter(col_names), None))
    
    # Convert arguments to c types.
    separator = py_to_c(separator, c_type=ctypes.c_char_p)
    col_names = py_to_c(col_names, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[K]))
    K = py_to_c(K, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_split_dataframe
    function.argtypes = [ctypes.c_char_p, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(separator, col_names, K), Transformation))


def make_select_column(
    key: Any,
    TOA: RuntimeTypeDescriptor,
    K: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that retrieves the column `key` from a dataframe as Vec<`TOA`>.
    
    :param key: categorical/hashable data type of the key/column name
    :type key: Any
    :param K: data type of the key
    :type K: :ref:`RuntimeTypeDescriptor`
    :param TOA: atomic data type to downcast to
    :type TOA: :ref:`RuntimeTypeDescriptor`
    :return: A select_column step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    K = RuntimeType.parse_or_infer(type_name=K, public_example=key)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    key = py_to_c(key, c_type=AnyObjectPtr, type_name=K)
    K = py_to_c(K, c_type=ctypes.c_char_p)
    TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_select_column
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(key, K, TOA), Transformation))


def make_identity(
    D: RuntimeTypeDescriptor,
    M: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that simply passes the data through.
    
    :param D: Domain of the identity function. Must be VectorDomain<AllDomain<_>> or AllDomain<_>
    :type D: :ref:`RuntimeTypeDescriptor`
    :param M: metric. Must be a dataset metric if D is a VectorDomain or a sensitivity metric if D is an AllDomain
    :type M: :ref:`RuntimeTypeDescriptor`
    :return: A transformation where the input and output domain are D and the input and output metric are M
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D)
    M = RuntimeType.parse(type_name=M)
    
    # Convert arguments to c types.
    D = py_to_c(D, c_type=ctypes.c_char_p)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_identity
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(D, M), Transformation))


def make_impute_constant(
    constant: Any,
    DA: RuntimeTypeDescriptor = "OptionNullDomain<AllDomain<TA>>"
) -> Transformation:
    """Make a Transformation that replaces null/None data with `constant`.
    By default, the input type is Vec<Option<TA>>, as emitted by make_cast. 
    Set `DA` to InherentNullDomain<AllDomain<TA>> for imputing on types that have an inherent representation of nullity, like floats.
    
    :param constant: Value to replace nulls with.
    :type constant: Any
    :param DA: domain of data being imputed. This is OptionNullDomain<AllDomain<TA>> or InherentNullDomain<AllDomain<TA>>
    :type DA: :ref:`RuntimeTypeDescriptor`
    :return: A impute_constant step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    DA = RuntimeType.parse(type_name=DA, generics=["TA"])
    TA = get_atom_or_infer(DA, constant)
    DA = DA.substitute(TA=TA)
    
    # Convert arguments to c types.
    constant = py_to_c(constant, c_type=AnyObjectPtr, type_name=TA)
    DA = py_to_c(DA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_impute_constant
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(constant, DA), Transformation))


def make_drop_null(
    DA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that drops null values.
    
    :param DA: atomic domain of input data that contains nulls. This is OptionNullDomain<AllDomain<TA>> or InherentNullDomain<AllDomain<TA>>
    :type DA: :ref:`RuntimeTypeDescriptor`
    :return: A drop_null step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    DA = RuntimeType.parse(type_name=DA)
    
    # Convert arguments to c types.
    DA = py_to_c(DA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_drop_null
    function.argtypes = [ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(DA), Transformation))


def make_impute_uniform_float(
    bounds: Tuple[Any, Any],
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that replaces null/None data in Vec<`TA`> with uniformly distributed floats within `bounds`.
    
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: Tuple[Any, Any]
    :param TA: type of data being imputed
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A impute_uniform_float step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_impute_uniform_float
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(bounds, TA), Transformation))


def make_find(
    categories: Any,
    TIA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Find the index of a data value in a set of categories.
    
    :param categories: The set of categories to find indexes from.
    :type categories: Any
    :param TIA: categorical/hashable input type. Input data must be Vec<TIA>.
    :type TIA: :ref:`RuntimeTypeDescriptor`
    :return: A find step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=next(iter(categories), None))
    
    # Convert arguments to c types.
    categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_find
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(categories, TIA), Transformation))


def make_find_bin(
    edges: Any,
    TIA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Find the bin index from a monotonically increasing vector of edges.
    
    :param edges: The set of edges to split bins by.
    :type edges: Any
    :param TIA: numerical input type. Input data must be Vec<TIA>.
    :type TIA: :ref:`RuntimeTypeDescriptor`
    :return: A find_bin step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=next(iter(edges), None))
    
    # Convert arguments to c types.
    edges = py_to_c(edges, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_find_bin
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(edges, TIA), Transformation))


def make_index(
    categories: Any,
    null: Any,
    TOA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Index into a vector of categories.
    
    :param categories: The set of categories to index into.
    :type categories: Any
    :param null: Category to return if the index is out-of-range of the category set.
    :type null: Any
    :param TOA: atomic output type. Output data will be Vec<TIA>.
    :type TOA: :ref:`RuntimeTypeDescriptor`
    :return: A index step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TOA = RuntimeType.parse_or_infer(type_name=TOA, public_example=next(iter(categories), None))
    
    # Convert arguments to c types.
    categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TOA]))
    null = py_to_c(null, c_type=AnyObjectPtr, type_name=TOA)
    TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_index
    function.argtypes = [AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(categories, null, TOA), Transformation))


def make_lipschitz_float_mul(
    constant,
    bounds: Tuple[Any, Any],
    D: RuntimeTypeDescriptor = "AllDomain<T>",
    M: RuntimeTypeDescriptor = "AbsoluteDistance<T>"
) -> Transformation:
    """Multiply an aggregate by a constant.
    
    :param constant: The constant to multiply aggregates by.
    :param bounds: Tuple of inclusive lower and upper bounds of the input data.
    :type bounds: Tuple[Any, Any]
    :param D: Domain of the function. Must be AllDomain<T> or VectorDomain<AllDomain<T>>
    :type D: :ref:`RuntimeTypeDescriptor`
    :param M: Metric. Must be AbsoluteDistance<T>, L1Distance<T> or L2Distance<T>
    :type M: :ref:`RuntimeTypeDescriptor`
    :return: A lipschitz_float_mul step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D, generics=["T"])
    M = RuntimeType.parse(type_name=M, generics=["T"])
    T = get_atom_or_infer(D, constant)
    D = D.substitute(T=T)
    M = M.substitute(T=T)
    
    # Convert arguments to c types.
    constant = py_to_c(constant, c_type=ctypes.c_void_p, type_name=T)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    D = py_to_c(D, c_type=ctypes.c_char_p)
    M = py_to_c(M, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_lipschitz_float_mul
    function.argtypes = [ctypes.c_void_p, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(constant, bounds, D, M), Transformation))


def make_sized_bounded_mean(
    size: int,
    bounds: Tuple[Any, Any],
    MI: RuntimeTypeDescriptor = "SymmetricDistance",
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the mean of bounded data. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size. 
    Use `make_clamp` to bound data and `make_bounded_resize` to establish dataset size.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of inclusive lower and upper bounds of the input data.
    :type bounds: Tuple[Any, Any]
    :param MI: Input Metric. One of `SymmetricDistance` or `InsertDeleteDistance`
    :type MI: :ref:`RuntimeTypeDescriptor`
    :param T: atomic data type
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_mean step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_mean
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, MI, T), Transformation))


def make_resize(
    size: int,
    constant: Any,
    MI: RuntimeTypeDescriptor = "SymmetricDistance",
    MO: RuntimeTypeDescriptor = "SymmetricDistance",
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that either truncates or imputes records with `constant` in a Vec<`TA`> to match a provided `size`.
    
    :param size: Number of records in output data.
    :type size: int
    :param constant: Value to impute with.
    :type constant: Any
    :param MI: Input Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
    :type MI: :ref:`RuntimeTypeDescriptor`
    :param MO: Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
    :type MO: :ref:`RuntimeTypeDescriptor`
    :param TA: Atomic type.
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A vector of the same type `TA`, but with the provided `size`.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=constant)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    constant = py_to_c(constant, c_type=AnyObjectPtr, type_name=TA)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_resize
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, constant, MI, MO, TA), Transformation))


def make_bounded_resize(
    size: int,
    bounds: Tuple[Any, Any],
    constant,
    MI: RuntimeTypeDescriptor = "SymmetricDistance",
    MO: RuntimeTypeDescriptor = "SymmetricDistance",
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that either truncates or imputes records with `constant` in a Vec<`TA`> to match a provided `size`.
    
    :param size: Number of records in output data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain
    :type bounds: Tuple[Any, Any]
    :param constant: Value to impute with.
    :param MI: Input Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
    :type MI: :ref:`RuntimeTypeDescriptor`
    :param MO: Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
    :type MO: :ref:`RuntimeTypeDescriptor`
    :param TA: Atomic type. If not passed, TA is inferred from the lower bound.
    :type TA: :ref:`RuntimeTypeDescriptor`
    :return: A vector of the same type `TA`, but with the provided `size`.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))
    constant = py_to_c(constant, c_type=ctypes.c_void_p, type_name=TA)
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_resize
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, constant, MI, MO, TA), Transformation))


def make_bounded_sum(
    bounds: Tuple[Any, Any],
    MI: RuntimeTypeDescriptor = "SymmetricDistance",
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded data. 
    Use `make_clamp` to bound data.
    
    :param bounds: Tuple of lower and upper bounds for data in the input domain
    :type bounds: Tuple[Any, Any]
    :param MI: Input Metric. One of `SymmetricDistance` or `InsertDeleteDistance`.
    :type MI: :ref:`RuntimeTypeDescriptor`
    :param T: atomic type of data
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A bounded_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_sum
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(bounds, MI, T), Transformation))


def make_sized_bounded_sum(
    size: int,
    bounds: Tuple[Any, Any],
    MI: RuntimeTypeDescriptor = "SymmetricDistance",
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded data with known dataset size. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility. 
    Use `make_clamp` to bound data and `make_bounded_resize` to establish dataset size.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param MI: Input Metric. One of `SymmetricDistance` or `InsertDeleteDistance`.
    :type MI: :ref:`RuntimeTypeDescriptor`
    :param T: atomic type of data
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, MI, T), Transformation))


def make_bounded_float_checked_sum(
    size_limit: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of bounded floats with unknown dataset size. 
    This computes the sum on up to `size_limit` rows randomly selected from the input.
    
    :param size_limit: Limit on number of records in input data.
    :type size_limit: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param S: summation algorithm to use on data type T. One of Sequential<T> or Pairwise<T>.
    :type S: :ref:`RuntimeTypeDescriptor`
    :return: A bounded_float_checked_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    size_limit = py_to_c(size_limit, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_float_checked_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size_limit, bounds, S), Transformation))


def make_sized_bounded_float_checked_sum(
    size: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of bounded floats with known dataset size. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param S: summation algorithm to use on data type T. One of Sequential<T> or Pairwise<T>.
    :type S: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_float_checked_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_float_checked_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, S), Transformation))


def make_bounded_float_ordered_sum(
    size_limit: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of bounded floats. 
    You may need to use `make_ordered_random` to impose an ordering on the data.
    
    :param size_limit: Upper bound on the number of records in input data. Used to bound sensitivity. Can be overestimated.
    :type size_limit: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain
    :type bounds: Tuple[Any, Any]
    :param S: summation algorithm to use on data type T. One of Sequential<T> or Pairwise<T>.
    :type S: :ref:`RuntimeTypeDescriptor`
    :return: A bounded_float_ordered_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    size_limit = py_to_c(size_limit, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_float_ordered_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size_limit, bounds, S), Transformation))


def make_sized_bounded_float_ordered_sum(
    size: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of bounded floats with known dataset size. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility. 
    You may need to use `make_ordered_random` to impose an ordering on the data.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param S: summation algorithm to use on data type T. One of Sequential<T> or Pairwise<T>.
    :type S: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_float_ordered_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_float_ordered_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, S), Transformation))


def make_sized_bounded_int_checked_sum(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints. 
    The effective range is reduced, as (bounds * size) must not overflow.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param T: atomic type of data
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_int_checked_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_int_checked_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, T), Transformation))


def make_bounded_int_monotonic_sum(
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints, where all values share the same sign.
    
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param T: atomic type of data
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A bounded_int_monotonic_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_int_monotonic_sum
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(bounds, T), Transformation))


def make_sized_bounded_int_monotonic_sum(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints with known dataset size. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility. 
    Adds the saturating sum of the positives to the saturating sum of the negatives.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param T: atomic type of data
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_int_monotonic_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_int_monotonic_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, T), Transformation))


def make_bounded_int_ordered_sum(
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints. 
    You may need to use `make_ordered_random` to impose an ordering on the data.
    
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param T: atomic type of data
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A bounded_int_ordered_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_int_ordered_sum
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(bounds, T), Transformation))


def make_sized_bounded_int_ordered_sum(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints with known dataset size. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility. 
    You may need to use `make_ordered_random` to impose an ordering on the data.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param T: atomic type of data
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_int_ordered_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_int_ordered_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, T), Transformation))


def make_bounded_int_split_sum(
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints. 
    Adds the saturating sum of the positives to the saturating sum of the negatives.
    
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param T: atomic type of data
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A bounded_int_split_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_int_split_sum
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(bounds, T), Transformation))


def make_sized_bounded_int_split_sum(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints with known dataset size. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility. 
    Adds the saturating sum of the positives to the saturating sum of the negatives.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param T: atomic type of data
    :type T: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_int_split_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_int_split_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, T), Transformation))


def make_sized_bounded_sum_of_squared_deviations(
    size: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of squared deviations of bounded data. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size. 
    Use `make_clamp` to bound data and `make_bounded_resize` to establish dataset size.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param S: summation algorithm to use on data type T. One of Sequential<T> or Pairwise<T>.
    :type S: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_sum_of_squared_deviations step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_sum_of_squared_deviations
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, S), Transformation))


def make_sized_bounded_variance(
    size: int,
    bounds: Tuple[Any, Any],
    ddof: int = 1,
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the variance of bounded data. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size. 
    Use `make_clamp` to bound data and `make_bounded_resize` to establish dataset size.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param ddof: Delta degrees of freedom. Set to 0 if not a sample, 1 for sample estimate.
    :type ddof: int
    :param S: summation algorithm to use on data type T. One of Sequential<T> or Pairwise<T>.
    :type S: :ref:`RuntimeTypeDescriptor`
    :return: A sized_bounded_variance step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    ddof = py_to_c(ddof, c_type=ctypes.c_uint)
    S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_variance
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_uint, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, ddof, S), Transformation))
