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
    "make_cast_metric",
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
    "make_sized_bounded_mean",
    "make_resize",
    "make_bounded_resize",
    "make_bounded_sum",
    "make_sized_bounded_sum",
    "make_sized_bounded_variance"
]


def make_cast(
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TIA` to type `TOA`. 
    Failure to parse results in None, else Some<TOA>.
    
    :param TIA: atomic input data type to cast from
    :type TIA: RuntimeTypeDescriptor
    :param TOA: atomic data type to cast into
    :type TOA: RuntimeTypeDescriptor
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
    :type TIA: RuntimeTypeDescriptor
    :param TOA: atomic data type to cast into
    :type TOA: RuntimeTypeDescriptor
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
    :type TIA: RuntimeTypeDescriptor
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
    :type DIA: RuntimeTypeDescriptor
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
    :type TIA: RuntimeTypeDescriptor
    :param TOA: data type to cast into
    :type TOA: RuntimeTypeDescriptor
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


def make_cast_metric(
    MI: DatasetMetric,
    MO: DatasetMetric,
    TA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that converts the dataset metric from type `MI` to type `MO`.
    
    :param MI: input dataset metric
    :type MI: DatasetMetric
    :param MO: output dataset metric
    :type MO: DatasetMetric
    :param TA: atomic type of data
    :type TA: RuntimeTypeDescriptor
    :return: A cast_metric step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_cast_metric
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(MI, MO, TA), Transformation))


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
    :type TA: RuntimeTypeDescriptor
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
    :type TA: RuntimeTypeDescriptor
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
    TO: RuntimeTypeDescriptor = "i32"
) -> Transformation:
    """Make a Transformation that computes a count of the number of records in data.
    
    :param TIA: Atomic Input Type. Input data is expected to be of the form Vec<TIA>.
    :type TIA: RuntimeTypeDescriptor
    :param TO: Output Type. Must be numeric.
    :type TO: RuntimeTypeDescriptor
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
    TO: RuntimeTypeDescriptor = "i32"
) -> Transformation:
    """Make a Transformation that computes a count of the number of unique, distinct records in data.
    
    :param TIA: Atomic Input Type. Input data is expected to be of the form Vec<TIA>.
    :type TIA: RuntimeTypeDescriptor
    :param TO: Output Type. Must be numeric.
    :type TO: RuntimeTypeDescriptor
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
    size: int,
    MO: SensitivityMetric,
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor = "i32"
) -> Transformation:
    """Make a Transformation that computes the count of each unique value in data. 
    This assumes that the category set is unknown. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size. 
    Use `make_resize` to establish dataset size. 
    This transformation depends on a measurement that has not yet been implemented.
    
    :param size: Number of records in input data.
    :type size: int
    :param MO: Output Metric.
    :type MO: SensitivityMetric
    :param TIA: Atomic Input Type. Categorical/hashable input data type. Input data must be Vec<TI>.
    :type TIA: RuntimeTypeDescriptor
    :param TOA: Atomic Output Type. Express counts in terms of this integral type.
    :type TOA: RuntimeTypeDescriptor
    :return: The carrier type is HashMap<TI, TO>- the counts for each unique data input.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)
    TIA = RuntimeType.parse(type_name=TIA)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_count_by
    function.argtypes = [ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, MO, TIA, TOA), Transformation))


def make_count_by_categories(
    categories: Any,
    MO: SensitivityMetric = "L1Distance<i32>",
    TIA: RuntimeTypeDescriptor = None,
    TOA: RuntimeTypeDescriptor = "i32"
) -> Transformation:
    """Make a Transformation that computes the number of times each category appears in the data. 
    This assumes that the category set is known.
    
    :param categories: The set of categories to compute counts for.
    :type categories: Any
    :param MO: output sensitivity metric
    :type MO: SensitivityMetric
    :param TIA: categorical/hashable input type. Input data must be Vec<TIA>.
    :type TIA: RuntimeTypeDescriptor
    :param TOA: express counts in terms of this numeric type
    :type TOA: RuntimeTypeDescriptor
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
    :type K: RuntimeTypeDescriptor
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
    :type K: RuntimeTypeDescriptor
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
    :type K: RuntimeTypeDescriptor
    :param TOA: atomic data type to downcast to
    :type TOA: RuntimeTypeDescriptor
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
    :type D: RuntimeTypeDescriptor
    :param M: metric. Must be a dataset metric if D is a VectorDomain or a sensitivity metric if D is an AllDomain
    :type M: RuntimeTypeDescriptor
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
    :type DA: RuntimeTypeDescriptor
    :return: A impute_constant step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    DA = RuntimeType.parse(type_name=DA, generics=["TA"])
    TA = get_domain_atom_or_infer(DA, constant)
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
    :type DA: RuntimeTypeDescriptor
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
    :type TA: RuntimeTypeDescriptor
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
    :type TIA: RuntimeTypeDescriptor
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
    :type TIA: RuntimeTypeDescriptor
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
    :type TOA: RuntimeTypeDescriptor
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


def make_sized_bounded_mean(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the mean of bounded data. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size. 
    Use `make_clamp` to bound data and `make_bounded_resize` to establish dataset size.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of inclusive lower and upper bounds of the input data.
    :type bounds: Tuple[Any, Any]
    :param T: atomic data type
    :type T: RuntimeTypeDescriptor
    :return: A sized_bounded_mean step.
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
    function = lib.opendp_trans__make_sized_bounded_mean
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, T), Transformation))


def make_resize(
    size: int,
    constant: Any,
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that either truncates or imputes records with `constant` in a Vec<`TA`> to match a provided `size`.
    
    :param size: Number of records in output data.
    :type size: int
    :param constant: Value to impute with.
    :type constant: Any
    :param TA: Atomic type.
    :type TA: RuntimeTypeDescriptor
    :return: A vector of the same type `TA`, but with the provided `size`.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=constant)
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    constant = py_to_c(constant, c_type=AnyObjectPtr, type_name=TA)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_resize
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, constant, TA), Transformation))


def make_bounded_resize(
    size: int,
    bounds: Tuple[Any, Any],
    constant,
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that either truncates or imputes records with `constant` in a Vec<`TA`> to match a provided `size`.
    
    :param size: Number of records in output data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain
    :type bounds: Tuple[Any, Any]
    :param constant: Value to impute with.
    :param TA: Atomic type. If not passed, TA is inferred from the lower bound.
    :type TA: RuntimeTypeDescriptor
    :return: A vector of the same type `TA`, but with the provided `size`.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    size = py_to_c(size, c_type=ctypes.c_uint)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))
    constant = py_to_c(constant, c_type=ctypes.c_void_p, type_name=TA)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_resize
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, constant, TA), Transformation))


def make_bounded_sum(
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded data. 
    Use `make_clamp` to bound data.
    
    :param bounds: Tuple of lower and upper bounds for data in the input domain
    :type bounds: Tuple[Any, Any]
    :param T: atomic type of data
    :type T: RuntimeTypeDescriptor
    :return: A bounded_sum step.
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
    function = lib.opendp_trans__make_bounded_sum
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(bounds, T), Transformation))


def make_sized_bounded_sum(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded data with known dataset size. 
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility. 
    Use `make_clamp` to bound data and `make_bounded_resize` to establish dataset size.
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for input data
    :type bounds: Tuple[Any, Any]
    :param T: atomic type of data
    :type T: RuntimeTypeDescriptor
    :return: A sized_bounded_sum step.
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
    function = lib.opendp_trans__make_sized_bounded_sum
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, T), Transformation))


def make_sized_bounded_variance(
    size: int,
    bounds: Tuple[Any, Any],
    ddof: int = 1,
    T: RuntimeTypeDescriptor = None
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
    :param T: atomic data type
    :type T: RuntimeTypeDescriptor
    :return: A sized_bounded_variance step.
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
    ddof = py_to_c(ddof, c_type=ctypes.c_uint)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_sized_bounded_variance
    function.argtypes = [ctypes.c_uint, AnyObjectPtr, ctypes.c_uint, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(size, bounds, ddof, T), Transformation))
