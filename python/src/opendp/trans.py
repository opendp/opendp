# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *


def make_cast(
    TI: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TI` to type `TO`. 
    Failure to parse results in None, else Some<TO>.
    
    :param TI: input data type to cast from
    :type TI: RuntimeTypeDescriptor
    :param TO: data type to cast into
    :type TO: RuntimeTypeDescriptor
    :return: A cast step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    TI = RuntimeType.parse(type_name=TI)
    TO = RuntimeType.parse(type_name=TO)
    
    # Convert arguments to c types.
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_cast
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TI, TO), Transformation))


def make_cast_default(
    TI: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TI` to type `TO`. If cast fails, fill with default.
    
    :param TI: input data type to cast from
    :type TI: RuntimeTypeDescriptor
    :param TO: data type to cast into
    :type TO: RuntimeTypeDescriptor
    :return: A cast_default step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    TI = RuntimeType.parse(type_name=TI)
    TO = RuntimeType.parse(type_name=TO)
    
    # Convert arguments to c types.
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_cast_default
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TI, TO), Transformation))


def make_is_equal(
    value: Any,
    TI: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that checks if each element is equal to `value`.
    
    :param value: value to check against
    :type value: Any
    :param TI: input data type
    :type TI: RuntimeTypeDescriptor
    :return: A is_equal step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    TI = RuntimeType.parse_or_infer(type_name=TI, public_example=value)
    
    # Convert arguments to c types.
    value = py_to_c(value, c_type=AnyObjectPtr, type_name=TI)
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_is_equal
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(value, TI), Transformation))


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
    TI: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TI` to a type that can represent nullity `TO`. 
    If cast fails, fill with `TO`'s null value.
    
    :param TI: input data type to cast from
    :type TI: RuntimeTypeDescriptor
    :param TO: data type to cast into
    :type TO: RuntimeTypeDescriptor
    :return: A cast_inherent step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    TI = RuntimeType.parse(type_name=TI)
    TO = RuntimeType.parse(type_name=TO)
    
    # Convert arguments to c types.
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_cast_inherent
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(TI, TO), Transformation))


def make_cast_metric(
    MI: DatasetMetric,
    MO: DatasetMetric,
    T: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that converts the dataset metric from type `MI` to type `MO`.
    
    :param MI: input dataset metric
    :type MI: DatasetMetric
    :param MO: output dataset metric
    :type MO: DatasetMetric
    :param T: atomic type of data
    :type T: RuntimeTypeDescriptor
    :return: A cast_metric step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    MI = py_to_c(MI, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_cast_metric
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(MI, MO, T), Transformation))


def make_clamp(
    lower,
    upper,
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that clamps numeric data in Vec<`T`> between `lower` and `upper`.
    
    :param lower: If datum is less than lower, let datum be lower.
    :param upper: If datum is greater than upper, let datum be upper.
    :param T: atomic data type
    :type T: RuntimeTypeDescriptor
    :return: A clamp step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    
    # Convert arguments to c types.
    lower = py_to_c(lower, c_type=ctypes.c_void_p, type_name=T)
    upper = py_to_c(upper, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_clamp
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(lower, upper, T), Transformation))


def make_unclamp(
    lower,
    upper,
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that unclamps a VectorDomain<IntervalDomain<T>> to a VectorDomain<AllDomain<T>>.
    
    :param lower: Lower bound of the input data.
    :param upper: Upper bound of the input data.
    :param T: atomic data type
    :type T: RuntimeTypeDescriptor
    :return: A unclamp step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    
    # Convert arguments to c types.
    lower = py_to_c(lower, c_type=ctypes.c_void_p, type_name=T)
    upper = py_to_c(upper, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_unclamp
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(lower, upper, T), Transformation))


def make_count(
    TIA: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor = "i32"
) -> Transformation:
    """Make a Transformation that computes a count of the number of records in data.
    
    :param TIA: Atomic Input Type. Input data is expected to be of the form Vec<TIA>.
    :type TIA: RuntimeTypeDescriptor
    :param TO: type of output integer
    :type TO: RuntimeTypeDescriptor
    :return: A count step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
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
    :param TO: Output Type. integer
    :type TO: RuntimeTypeDescriptor
    :return: A count_distinct step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
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
    n: int,
    MO: SensitivityMetric,
    TI: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor = "i32"
) -> Transformation:
    """Make a Transformation that computes the count of each unique value in data. 
    This assumes that the category set is unknown. 
    Use make_base_stability to release this query.
    
    :param n: Number of records in input data.
    :type n: int
    :param MO: Output Metric.
    :type MO: SensitivityMetric
    :param TI: Input Type. Categorical/hashable input data type. Input data must be Vec<TI>.
    :type TI: RuntimeTypeDescriptor
    :param TO: Output Type. express counts in terms of this integral type
    :type TO: RuntimeTypeDescriptor
    :return: The carrier type is HashMap<TI, TO>- the counts for each unique data input.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)
    TI = RuntimeType.parse(type_name=TI)
    TO = RuntimeType.parse(type_name=TO)
    
    # Convert arguments to c types.
    n = py_to_c(n, c_type=ctypes.c_uint)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_count_by
    function.argtypes = [ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(n, MO, TI, TO), Transformation))


def make_count_by_categories(
    categories: Any,
    MO: SensitivityMetric,
    TI: RuntimeTypeDescriptor = None,
    TO: RuntimeTypeDescriptor = "i32"
) -> Transformation:
    """Make a Transformation that computes the number of times each category appears in the data. 
    This assumes that the category set is known.
    
    :param categories: The set of categories to compute counts for.
    :type categories: Any
    :param MO: output sensitivity metric
    :type MO: SensitivityMetric
    :param TI: categorical/hashable input data type. Input data must be Vec<TI>.
    :type TI: RuntimeTypeDescriptor
    :param TO: express counts in terms of this integral type
    :type TO: RuntimeTypeDescriptor
    :return: A count_by_categories step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)
    TI = RuntimeType.parse_or_infer(type_name=TI, public_example=next(iter(categories), None))
    TO = RuntimeType.parse(type_name=TO)
    
    # Convert arguments to c types.
    categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TI]))
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    TI = py_to_c(TI, c_type=ctypes.c_char_p)
    TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_count_by_categories
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(categories, MO, TI, TO), Transformation))


def make_split_lines(
    
) -> Transformation:
    """Make a Transformation that takes a string and splits it into a Vec<String> of its lines.
    
    
    :return: A split_lines step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
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
    """Make a Transformation that splits each record in a Vec<String> into a Vec<Vec<String>>,
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


def make_parse_column(
    key: Any,
    T: RuntimeTypeDescriptor,
    impute: bool = True,
    K: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that parses the `key` column of a dataframe as `T`.
    
    :param key: name of column to select from dataframe and parse
    :type key: Any
    :param impute: Enable to impute values that fail to parse. If false, raise an error if parsing fails.
    :type impute: bool
    :param K: categorical/hashable data type of the key/column name
    :type K: RuntimeTypeDescriptor
    :param T: data type to parse into
    :type T: RuntimeTypeDescriptor
    :return: A parse_column step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    K = RuntimeType.parse_or_infer(type_name=K, public_example=key)
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    key = py_to_c(key, c_type=AnyObjectPtr, type_name=K)
    impute = py_to_c(impute, c_type=ctypes.c_bool)
    K = py_to_c(K, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_parse_column
    function.argtypes = [AnyObjectPtr, ctypes.c_bool, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(key, impute, K, T), Transformation))


def make_select_column(
    key: Any,
    T: RuntimeTypeDescriptor,
    K: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that retrieves the column `key` from a dataframe as Vec<`T`>.
    
    :param key: categorical/hashable data type of the key/column name
    :type key: Any
    :param K: data type of the key
    :type K: RuntimeTypeDescriptor
    :param T: data type to downcast to
    :type T: RuntimeTypeDescriptor
    :return: A select_column step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    K = RuntimeType.parse_or_infer(type_name=K, public_example=key)
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    key = py_to_c(key, c_type=AnyObjectPtr, type_name=K)
    K = py_to_c(K, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_select_column
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(key, K, T), Transformation))


def make_identity(
    M: DatasetMetric,
    T: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that simply passes the data through.
    
    :param M: dataset metric
    :type M: DatasetMetric
    :param T: Type of data passed to the identity function.
    :type T: RuntimeTypeDescriptor
    :return: A identity step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    M = RuntimeType.parse(type_name=M)
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    M = py_to_c(M, c_type=ctypes.c_char_p)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_identity
    function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(M, T), Transformation))


def make_impute_constant(
    constant: Any,
    DA: RuntimeTypeDescriptor = "OptionNullDomain<AllDomain<T>>"
) -> Transformation:
    """Make a Transformation that replaces null/None data with `constant`.
    By default, the input type is Vec<Option<`T`>>, as emitted by make_cast. 
    Set DA to InherentNullDomain<AllDomain<T>> for imputing on types that have an inherent representation of nullity, like floats.
    
    :param constant: Value to replace nulls with.
    :type constant: Any
    :param DA: domain of data being imputed. This is OptionNullDomain<AllDomain<T>> or InherentNullDomain<AllDomain<T>>
    :type DA: RuntimeTypeDescriptor
    :return: A impute_constant step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    DA = RuntimeType.parse(type_name=DA, generics=["T"])
    T = get_domain_atom_or_infer(DA, constant)
    DA = DA.substitute(T=T)
    
    # Convert arguments to c types.
    constant = py_to_c(constant, c_type=AnyObjectPtr, type_name=T)
    DA = py_to_c(DA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_impute_constant
    function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(constant, DA), Transformation))


def make_impute_uniform_float(
    lower,
    upper,
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that replaces null/None data in Vec<`T`> with `constant`
    
    :param lower: Lower bound of uniform distribution to sample from.
    :param upper: Upper bound of uniform distribution to sample from.
    :param T: type of data being imputed
    :type T: RuntimeTypeDescriptor
    :return: A impute_uniform_float step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    
    # Convert arguments to c types.
    lower = py_to_c(lower, c_type=ctypes.c_void_p, type_name=T)
    upper = py_to_c(upper, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_impute_uniform_float
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(lower, upper, T), Transformation))


def make_bounded_mean(
    lower,
    upper,
    n: int,
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the mean of bounded data. 
    Use make_clamp to bound data.
    
    :param lower: Lower bound of input data.
    :param upper: Upper bound of input data.
    :param n: Number of records in input data.
    :type n: int
    :param T: atomic data type
    :type T: RuntimeTypeDescriptor
    :return: A bounded_mean step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    
    # Convert arguments to c types.
    lower = py_to_c(lower, c_type=ctypes.c_void_p, type_name=T)
    upper = py_to_c(upper, c_type=ctypes.c_void_p, type_name=T)
    n = py_to_c(n, c_type=ctypes.c_uint)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_mean
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(lower, upper, n, T), Transformation))


def make_resize(
    constant: Any,
    length: int,
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that either truncates or imputes records with `constant` in a Vec<`T`> to match a provided `length`.\nWARNING: This function is temporary. It will be replaced by a more general make_resize that accepts domains
    
    :param constant: Value to impute with.
    :type constant: Any
    :param length: Number of records in output data.
    :type length: int
    :param TA: Atomic type.
    :type TA: RuntimeTypeDescriptor
    :return: A vector of the same type `TA`, but with the provided `length`.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=constant)
    
    # Convert arguments to c types.
    constant = py_to_c(constant, c_type=AnyObjectPtr, type_name=TA)
    length = py_to_c(length, c_type=ctypes.c_uint)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_resize
    function.argtypes = [AnyObjectPtr, ctypes.c_uint, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(constant, length, TA), Transformation))


def make_resize_bounded(
    constant,
    length: int,
    lower,
    upper,
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that either truncates or imputes records with `constant` in a Vec<`T`> to match a provided `length`.\nWARNING: This function is temporary. It will be replaced by a more general make_resize_constant that accepts domains
    
    :param constant: Value to impute with.
    :param length: Number of records in output data.
    :type length: int
    :param lower: Lower bound of data in input domain
    :param upper: Upper bound of data in input domain
    :param TA: Atomic type.
    :type TA: RuntimeTypeDescriptor
    :return: A vector of the same type `TA`, but with the provided `length`.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=constant)
    
    # Convert arguments to c types.
    constant = py_to_c(constant, c_type=ctypes.c_void_p, type_name=TA)
    length = py_to_c(length, c_type=ctypes.c_uint)
    lower = py_to_c(lower, c_type=ctypes.c_void_p, type_name=TA)
    upper = py_to_c(upper, c_type=ctypes.c_void_p, type_name=TA)
    TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_resize_bounded
    function.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(constant, length, lower, upper, TA), Transformation))


def make_bounded_sum(
    lower,
    upper,
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded data. 
    Use make_clamp to bound data.
    
    :param lower: Lower bound of input data.
    :param upper: Upper bound of input data.
    :param T: atomic type of data
    :type T: RuntimeTypeDescriptor
    :return: A bounded_sum step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    
    # Convert arguments to c types.
    lower = py_to_c(lower, c_type=ctypes.c_void_p, type_name=T)
    upper = py_to_c(upper, c_type=ctypes.c_void_p, type_name=T)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_sum
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(lower, upper, T), Transformation))


def make_bounded_sum_n(
    lower,
    upper,
    n: int,
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded data with known length. 
    This uses a restricted-sensitivity proof that takes advantage of known N for better utility. 
    Use make_clamp to bound data.
    
    :param lower: Lower bound of input data.
    :param upper: Upper bound of input data.
    :param n: Number of records in input data.
    :type n: int
    :param T: atomic type of data
    :type T: RuntimeTypeDescriptor
    :return: A bounded_sum_n step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    
    # Convert arguments to c types.
    lower = py_to_c(lower, c_type=ctypes.c_void_p, type_name=T)
    upper = py_to_c(upper, c_type=ctypes.c_void_p, type_name=T)
    n = py_to_c(n, c_type=ctypes.c_uint)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_sum_n
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(lower, upper, n, T), Transformation))


def make_bounded_variance(
    lower,
    upper,
    n: int,
    ddof: int = 1,
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the variance of bounded data. 
    Use make_clamp to bound data.
    
    :param lower: Lower bound of input data.
    :param upper: Upper bound of input data.
    :param n: Number of records in input data.
    :type n: int
    :param ddof: Delta degrees of freedom. Set to 0 if not a sample, 1 for sample estimate.
    :type ddof: int
    :param T: atomic data type
    :type T: RuntimeTypeDescriptor
    :return: A bounded_variance step.
    :rtype: Transformation
    :raises AssertionError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type-argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=lower)
    
    # Convert arguments to c types.
    lower = py_to_c(lower, c_type=ctypes.c_void_p, type_name=T)
    upper = py_to_c(upper, c_type=ctypes.c_void_p, type_name=T)
    n = py_to_c(n, c_type=ctypes.c_uint)
    ddof = py_to_c(ddof, c_type=ctypes.c_uint)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_trans__make_bounded_variance
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint, ctypes.c_uint, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(lower, upper, n, ddof, T), Transformation))
