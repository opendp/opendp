# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "atom_domain",
    "bounded_domain",
    "domain_carrier_type",
    "domain_debug",
    "domain_type",
    "member",
    "sized_domain",
    "vector_domain"
]


def atom_domain(
    T: RuntimeTypeDescriptor
):
    """Construct an instance of `AtomDomain`.
    
    [atom_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.atom_domain.html)
    
    :param T: The type of the atom.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_domains__atom_domain
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Domain))
    
    return output


def bounded_domain(
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
):
    """Construct an instance of `BoundedDomain`.
    
    [bounded_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.bounded_domain.html)
    
    :param bounds: A tuple of upper/lower bounds.
    :type bounds: Tuple[Any, Any]
    :param T: The type of the atom.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_domains__bounded_domain
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_bounds, c_T), Domain))
    
    return output


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


def sized_domain(
    inner_domain,
    size: int
):
    """Construct an instance of `VectorDomain`.
    
    [sized_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.sized_domain.html)
    
    :param inner_domain: The inner domain.
    :param size: Number of elements in inner domain.
    :type size: int
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_inner_domain = py_to_c(inner_domain, c_type=Domain, type_name=AnyDomain)
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    
    # Call library function.
    lib_function = lib.opendp_domains__sized_domain
    lib_function.argtypes = [Domain, ctypes.c_size_t]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_inner_domain, c_size), Domain))
    
    return output


def vector_domain(
    atom_domain
):
    """Construct an instance of `VectorDomain`.
    
    [vector_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.vector_domain.html)
    
    :param atom_domain: The inner domain.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_atom_domain = py_to_c(atom_domain, c_type=Domain, type_name=AnyDomain)
    
    # Call library function.
    lib_function = lib.opendp_domains__vector_domain
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_atom_domain), Domain))
    
    return output
