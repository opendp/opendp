# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "_domain_free",
    "atom_domain",
    "csv_domain",
    "dataframe_domain",
    "dataframe_domain_with_counts",
    "domain_carrier_type",
    "domain_debug",
    "domain_type",
    "expr_domain",
    "lazyframe_domain",
    "lazyframe_domain_with_counts",
    "map_domain",
    "member",
    "option_domain",
    "series_domain",
    "vector_domain"
]


@versioned
def _domain_free(
    this
):
    """Internal function. Free the memory associated with `this`.
    
    [_domain_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn._domain_free.html)
    
    :param this: 
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this
    
    # Call library function.
    lib_function = lib.opendp_domains___domain_free
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))
    
    return output


@versioned
def atom_domain(
    bounds: Any = None,
    nullable: bool = False,
    T: RuntimeTypeDescriptor = None
):
    """Construct an instance of `AtomDomain`.
    
    [atom_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.atom_domain.html)
    
    :param bounds: 
    :type bounds: Any
    :param nullable: 
    :type nullable: bool
    :param T: The type of the atom.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[RuntimeType(origin='Tuple', args=[T, T])]))
    c_nullable = py_to_c(nullable, c_type=ctypes.c_bool, type_name=bool)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_domains__atom_domain
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_bool, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_bounds, c_nullable, c_T), Domain))
    
    return output


@versioned
def csv_domain(
    lazyframe_domain,
    delimiter = ",",
    has_header: bool = True,
    skip_rows: int = 0,
    comment_char: str = None,
    quote_char: str = "\"",
    eol_char = "\n"
):
    """Parse a path to a CSV into a LazyFrame.
    
    [csv_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.csv_domain.html)
    
    :param lazyframe_domain: The domain of the LazyFrame to be constructed
    :param delimiter: Set the CSV file's column delimiter as a byte character
    :param has_header: Set whether the CSV file has headers
    :type has_header: bool
    :param skip_rows: Skip the first `n` rows during parsing. The header will be parsed at row `n`.
    :type skip_rows: int
    :param comment_char: Set the comment character. Lines starting with this character will be ignored.
    :type comment_char: str
    :param quote_char: Set the `char` used as quote char. The default is `"`. If set to `[None]` quoting is disabled.
    :type quote_char: str
    :param eol_char: Set the `char` used as end of line. The default is `\\n`.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_lazyframe_domain = py_to_c(lazyframe_domain, c_type=Domain, type_name=None)
    c_delimiter = py_to_c(delimiter, c_type=ctypes.c_char, type_name=char)
    c_has_header = py_to_c(has_header, c_type=ctypes.c_bool, type_name=bool)
    c_skip_rows = py_to_c(skip_rows, c_type=ctypes.c_uint, type_name=usize)
    c_comment_char = py_to_c(comment_char, c_type=ctypes.c_char_p, type_name=RuntimeType(origin='Option', args=[char]))
    c_quote_char = py_to_c(quote_char, c_type=ctypes.c_char_p, type_name=RuntimeType(origin='Option', args=[char]))
    c_eol_char = py_to_c(eol_char, c_type=ctypes.c_char, type_name=char)
    
    # Call library function.
    lib_function = lib.opendp_domains__csv_domain
    lib_function.argtypes = [Domain, ctypes.c_char, ctypes.c_bool, ctypes.c_uint, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_lazyframe_domain, c_delimiter, c_has_header, c_skip_rows, c_comment_char, c_quote_char, c_eol_char), Domain))
    
    return output


@versioned
def dataframe_domain(
    series_domains: Any
):
    """Construct an instance of `DataFrameDomain`.
    
    [dataframe_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.dataframe_domain.html)
    
    :param series_domains: Domain of each series in the dataframe.
    :type series_domains: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_series_domains = py_to_c(series_domains, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[SeriesDomain]))
    
    # Call library function.
    lib_function = lib.opendp_domains__dataframe_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_series_domains), Domain))
    
    return output


@versioned
def dataframe_domain_with_counts(
    dataframe_domain,
    counts: Any
):
    """[dataframe_domain_with_counts in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.dataframe_domain_with_counts.html)
    
    :param dataframe_domain: 
    :param counts: 
    :type counts: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_dataframe_domain = py_to_c(dataframe_domain, c_type=Domain, type_name=None)
    c_counts = py_to_c(counts, c_type=AnyObjectPtr, type_name=DataFrame)
    
    # Call library function.
    lib_function = lib.opendp_domains__dataframe_domain_with_counts
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_dataframe_domain, c_counts), Domain))
    
    return output


@versioned
def domain_carrier_type(
    this
) -> str:
    """Get the carrier type of a `domain`.
    
    [domain_carrier_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.domain_carrier_type.html)
    
    :param this: The domain to retrieve the carrier type from.
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Domain, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_domains__domain_carrier_type
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def domain_debug(
    this
) -> str:
    """Debug a `domain`.
    
    [domain_debug in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.domain_debug.html)
    
    :param this: The domain to debug (stringify).
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Domain, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_domains__domain_debug
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def domain_type(
    this
) -> str:
    """Get the type of a `domain`.
    
    [domain_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.domain_type.html)
    
    :param this: The domain to retrieve the type from.
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Domain, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_domains__domain_type
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def expr_domain(
    lazyframe_domain,
    active_column: str,
    context: str = None,
    grouping_columns: Any = None
):
    """Construct an ExprDomain from a LazyFrameDomain.
    
    Must pass either `context` or `grouping_columns`.
    
    [expr_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.expr_domain.html)
    
    :param lazyframe_domain: the domain of the LazyFrame to be constructed
    :param context: used when the constructor is called inside a lazyframe context constructor
    :type context: str
    :param grouping_columns: used when the constructor is called inside a groupby context constructor
    :type grouping_columns: Any
    :param active_column: which column to apply expressions to
    :type active_column: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_lazyframe_domain = py_to_c(lazyframe_domain, c_type=Domain, type_name=None)
    c_context = py_to_c(context, c_type=ctypes.c_char_p, type_name=None)
    c_grouping_columns = py_to_c(grouping_columns, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[RuntimeType(origin='Vec', args=[String])]))
    c_active_column = py_to_c(active_column, c_type=ctypes.c_char_p, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_domains__expr_domain
    lib_function.argtypes = [Domain, ctypes.c_char_p, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_lazyframe_domain, c_context, c_grouping_columns, c_active_column), Domain))
    
    return output


@versioned
def lazyframe_domain(
    series_domains: Any
):
    """Construct an instance of `LazyFrameDomain`.
    
    [lazyframe_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.lazyframe_domain.html)
    
    :param series_domains: Domain of each series in the lazyframe.
    :type series_domains: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_series_domains = py_to_c(series_domains, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[SeriesDomain]))
    
    # Call library function.
    lib_function = lib.opendp_domains__lazyframe_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_series_domains), Domain))
    
    return output


@versioned
def lazyframe_domain_with_counts(
    lazyframe_domain,
    counts: Any
):
    """[lazyframe_domain_with_counts in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.lazyframe_domain_with_counts.html)
    
    :param lazyframe_domain: 
    :param counts: 
    :type counts: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_lazyframe_domain = py_to_c(lazyframe_domain, c_type=Domain, type_name=None)
    c_counts = py_to_c(counts, c_type=AnyObjectPtr, type_name=LazyFrame)
    
    # Call library function.
    lib_function = lib.opendp_domains__lazyframe_domain_with_counts
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_lazyframe_domain, c_counts), Domain))
    
    return output


@versioned
def map_domain(
    key_domain,
    value_domain
):
    """Construct an instance of `MapDomain`.
    
    [map_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.map_domain.html)
    
    :param key_domain: domain of keys in the hashmap
    :param value_domain: domain of values in the hashmap
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_key_domain = py_to_c(key_domain, c_type=Domain, type_name=AnyDomain)
    c_value_domain = py_to_c(value_domain, c_type=Domain, type_name=AnyDomain)
    
    # Call library function.
    lib_function = lib.opendp_domains__map_domain
    lib_function.argtypes = [Domain, Domain]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_key_domain, c_value_domain), Domain))
    
    return output


@versioned
def member(
    this: Domain,
    val: Any
):
    """Check membership in a `domain`.
    
    [member in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.member.html)
    
    :param this: The domain to check membership in.
    :type this: Domain
    :param val: A potential element of the domain.
    :type val: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Domain, type_name=AnyDomain)
    c_val = py_to_c(val, c_type=AnyObjectPtr, type_name=domain_carrier_type(this))
    
    # Call library function.
    lib_function = lib.opendp_domains__member
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this, c_val), BoolPtr))
    
    return output


@versioned
def option_domain(
    element_domain,
    D: RuntimeTypeDescriptor = None
):
    """Construct an instance of `OptionDomain`.
    
    [option_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.option_domain.html)
    
    :param element_domain: 
    :param D: The type of the inner domain.
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    D = RuntimeType.parse_or_infer(type_name=D, public_example=element_domain)
    
    # Convert arguments to c types.
    c_element_domain = py_to_c(element_domain, c_type=Domain, type_name=D)
    c_D = py_to_c(D, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_domains__option_domain
    lib_function.argtypes = [Domain, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_element_domain, c_D), Domain))
    
    return output


@versioned
def series_domain(
    name: str,
    element_domain
):
    """Construct an instance of `SeriesDomain`.
    
    [series_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.series_domain.html)
    
    :param name: The name of the series.
    :type name: str
    :param element_domain: The domain of elements in the series.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_name = py_to_c(name, c_type=ctypes.c_char_p, type_name=str)
    c_element_domain = py_to_c(element_domain, c_type=Domain, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_domains__series_domain
    lib_function.argtypes = [ctypes.c_char_p, Domain]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_name, c_element_domain), Domain))
    
    return output


@versioned
def vector_domain(
    atom_domain,
    size: Any = None
):
    """Construct an instance of `VectorDomain`.
    
    [vector_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.vector_domain.html)
    
    :param atom_domain: The inner domain.
    :param size: 
    :type size: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_atom_domain = py_to_c(atom_domain, c_type=Domain, type_name=AnyDomain)
    c_size = py_to_c(size, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[i32]))
    
    # Call library function.
    lib_function = lib.opendp_domains__vector_domain
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_atom_domain, c_size), Domain))
    
    return output
