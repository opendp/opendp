# Auto-generated. Do not edit!
'''
The ``domains`` module provides functions for creating and using domains.
For more context, see :ref:`domains in the User Guide <domains-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp
'''
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "_domain_free",
    "_user_domain_descriptor",
    "atom_domain",
    "domain_carrier_type",
    "domain_debug",
    "domain_type",
    "map_domain",
    "member",
    "option_domain",
    "user_domain",
    "vector_domain"
]


def _domain_free(
    this
):
    r"""Internal function. Free the memory associated with `this`.

    :param this: 
    :type this: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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


def _user_domain_descriptor(
    domain: Domain
):
    r"""Retrieve the descriptor value stored in a user domain.

    :param domain: The UserDomain to extract the descriptor from
    :type domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=AnyDomain)

    # Call library function.
    lib_function = lib.opendp_domains___user_domain_descriptor
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_domain), ExtrinsicObjectPtr))

    return output


def atom_domain(
    bounds: Optional[Any] = None,
    nullable: bool = False,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Domain:
    r"""Construct an instance of `AtomDomain`.

    [atom_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.atom_domain.html)

    :param bounds: 
    :type bounds: Any
    :param nullable: 
    :type nullable: bool
    :param T: The type of the atom.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> import opendp.prelude as dp
    >>> dp.atom_domain(T=float)
    AtomDomain(T=f64)

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


def domain_carrier_type(
    this: Domain
) -> str:
    r"""Get the carrier type of a `domain`.

    :param this: The domain to retrieve the carrier type from.
    :type this: Domain
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    this: Domain
) -> str:
    r"""Debug a `domain`.

    :param this: The domain to debug (stringify).
    :type this: Domain
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    this: Domain
) -> str:
    r"""Get the type of a `domain`.

    :param this: The domain to retrieve the type from.
    :type this: Domain
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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


def map_domain(
    key_domain: Domain,
    value_domain: Domain
) -> Domain:
    r"""Construct an instance of `MapDomain`.

    :param key_domain: domain of keys in the hashmap
    :type key_domain: Domain
    :param value_domain: domain of values in the hashmap
    :type value_domain: Domain
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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


def member(
    this: Domain,
    val: Any
):
    r"""Check membership in a `domain`.

    :param this: The domain to check membership in.
    :type this: Domain
    :param val: A potential element of the domain.
    :type val: Any
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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


def option_domain(
    element_domain: Domain,
    D: Optional[RuntimeTypeDescriptor] = None
) -> Domain:
    r"""Construct an instance of `OptionDomain`.

    [option_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.option_domain.html)

    :param element_domain: 
    :type element_domain: Domain
    :param D: The type of the inner domain.
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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


def user_domain(
    identifier: str,
    member,
    descriptor = None
) -> Domain:
    r"""Construct a new UserDomain.
    Any two instances of an UserDomain are equal if their string descriptors are equal.
    Contains a function used to check if any value is a member of the domain.

    :param identifier: A string description of the data domain.
    :type identifier: str
    :param member: A function used to test if a value is a member of the data domain.
    :param descriptor: Additional constraints on the domain.
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("honest-but-curious")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_identifier = py_to_c(identifier, c_type=ctypes.c_char_p, type_name=None)
    c_member = py_to_c(member, c_type=CallbackFn, type_name=bool)
    c_descriptor = py_to_c(descriptor, c_type=ExtrinsicObjectPtr, type_name=ExtrinsicObject)

    # Call library function.
    lib_function = lib.opendp_domains__user_domain
    lib_function.argtypes = [ctypes.c_char_p, CallbackFn, ExtrinsicObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_identifier, c_member, c_descriptor), Domain))
    output._depends_on(c_member)
    return output


def vector_domain(
    atom_domain: Domain,
    size: Optional[Any] = None
) -> Domain:
    r"""Construct an instance of `VectorDomain`.

    :param atom_domain: The inner domain.
    :type atom_domain: Domain
    :param size: 
    :type size: Any
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_atom_domain = py_to_c(atom_domain, c_type=Domain, type_name=None)
    c_size = py_to_c(size, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[i32]))

    # Call library function.
    lib_function = lib.opendp_domains__vector_domain
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_atom_domain, c_size), Domain))

    return output
