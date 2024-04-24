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
    "expr_domain",
    "infer_lazyframe_domain",
    "lazyframe_domain",
    "map_domain",
    "member",
    "option_domain",
    "series_domain",
    "user_domain",
    "vector_domain",
    "with_margin"
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
    bounds = None,
    nullable: bool = False,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Domain:
    r"""Construct an instance of `AtomDomain`.

    [atom_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.atom_domain.html)

    :param bounds: 
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


def expr_domain(
    lazyframe_domain: Domain,
    grouping_columns: Optional[List[str]] = None
) -> Domain:
    r"""Construct an ExprDomain from a LazyFrameDomain.

    Must pass either `context` or `grouping_columns`.

    :param lazyframe_domain: the domain of the LazyFrame to be constructed
    :type lazyframe_domain: Domain
    :param grouping_columns: set when creating an expression that aggregates
    :type grouping_columns: List[str]
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_lazyframe_domain = py_to_c(lazyframe_domain, c_type=Domain, type_name=None)
    c_grouping_columns = py_to_c(grouping_columns, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[RuntimeType(origin='Vec', args=[String])]))

    # Call library function.
    lib_function = lib.opendp_domains__expr_domain
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_lazyframe_domain, c_grouping_columns), Domain))

    return output


def infer_lazyframe_domain(
    lazyframe
) -> Domain:
    r"""Infer the lazyframe domain that a dataset is a member of.

    WARNING: This function looks at the data to infer the domain,
    and should only be used if you consider the column names and column types to be public information.

    [infer_lazyframe_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.infer_lazyframe_domain.html)

    :param lazyframe: The lazyframe to infer the domain from.
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_lazyframe = py_to_c(lazyframe, c_type=AnyObjectPtr, type_name=LazyFrame)

    # Call library function.
    lib_function = lib.opendp_domains__infer_lazyframe_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_lazyframe), Domain))

    return output


def lazyframe_domain(
    series_domains
) -> Domain:
    r"""Construct an instance of `LazyFrameDomain`.

    :param series_domains: Domain of each series in the lazyframe.
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    val
):
    r"""Check membership in a `domain`.

    :param this: The domain to check membership in.
    :type this: Domain
    :param val: A potential element of the domain.
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


def series_domain(
    name: str,
    element_domain: Domain
) -> Domain:
    r"""Construct an instance of `SeriesDomain`.

    [series_domain in Rust documentation.](https://docs.rs/opendp/latest/opendp/domains/fn.series_domain.html)

    :param name: The name of the series.
    :type name: str
    :param element_domain: The domain of elements in the series.
    :type element_domain: Domain
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    size = None
) -> Domain:
    r"""Construct an instance of `VectorDomain`.

    :param atom_domain: The inner domain.
    :type atom_domain: Domain
    :param size: 
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


def with_margin(
    frame_domain: Domain,
    by,
    max_partition_length = None,
    max_num_partitions = None,
    max_partition_contributions = None,
    max_influenced_partitions = None,
    public_info: Optional[str] = None
) -> Domain:
    r"""

    :param frame_domain: 
    :type frame_domain: Domain
    :param by: 
    :param max_partition_length: 
    :param max_num_partitions: 
    :param max_partition_contributions: 
    :param max_influenced_partitions: 
    :param public_info: 
    :type public_info: str
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_frame_domain = py_to_c(frame_domain, c_type=Domain, type_name=None)
    c_by = py_to_c(by, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[String]))
    c_max_partition_length = py_to_c(max_partition_length, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[u32]))
    c_max_num_partitions = py_to_c(max_num_partitions, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[u32]))
    c_max_partition_contributions = py_to_c(max_partition_contributions, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[u32]))
    c_max_influenced_partitions = py_to_c(max_influenced_partitions, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[u32]))
    c_public_info = py_to_c(public_info, c_type=ctypes.c_char_p, type_name=RuntimeType(origin='Option', args=[String]))

    # Call library function.
    lib_function = lib.opendp_domains__with_margin
    lib_function.argtypes = [Domain, AnyObjectPtr, AnyObjectPtr, AnyObjectPtr, AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_frame_domain, c_by, c_max_partition_length, c_max_num_partitions, c_max_partition_contributions, c_max_influenced_partitions, c_public_info), Domain))

    return output
