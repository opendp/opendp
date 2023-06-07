# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "_domain_free",
    "atom_domain",
    "domain_carrier_type",
    "domain_debug",
    "domain_type",
    "lazy_frame_domain",
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
def lazy_frame_domain(
    series_domains: Any
):
    """Construct an instance of `LazyFrameDomain`.
    
    [lazy_frame_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.lazy_frame_domain.html)
    
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
    lib_function = lib.opendp_domains__lazy_frame_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_series_domains), Domain))
    
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
    element_domain,
    DI: RuntimeTypeDescriptor = None
):
    """Construct an instance of `SeriesDomain`.
    
    [series_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.series_domain.html)
    
    :param name: The name of the series.
    :type name: str
    :param element_domain: The domain of elements in the series.
    :param DI: 
    :type DI: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    DI = RuntimeType.parse_or_infer(type_name=DI, public_example=element_domain)
    
    # Convert arguments to c types.
    c_name = py_to_c(name, c_type=ctypes.c_char_p, type_name=str)
    c_element_domain = py_to_c(element_domain, c_type=Domain, type_name=DI)
    c_DI = py_to_c(DI, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_domains__series_domain
    lib_function.argtypes = [ctypes.c_char_p, Domain, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_name, c_element_domain, c_DI), Domain))
    
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
