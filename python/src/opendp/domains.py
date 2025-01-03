# Auto-generated. Do not edit!
'''
The ``domains`` module provides functions for creating and using domains.
For more context, see :ref:`domains in the User Guide <domains-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp
'''
from deprecated.sphinx import deprecated # noqa: F401 (Not every file actually has deprecated functions.)

from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "_domain_free",
    "_extrinsic_domain_descriptor",
    "_lazyframe_from_domain",
    "atom_domain",
    "bitvector_domain",
    "categorical_domain",
    "datetime_domain",
    "domain_carrier_type",
    "domain_debug",
    "domain_type",
    "lazyframe_domain",
    "map_domain",
    "member",
    "option_domain",
    "series_domain",
    "user_domain",
    "vector_domain",
    "wild_expr_domain",
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


def _extrinsic_domain_descriptor(
    domain: Domain
):
    r"""Retrieve the descriptor value stored in an extrinsic domain.

    :param domain: The ExtrinsicDomain to extract the descriptor from
    :type domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=AnyDomain)

    # Call library function.
    lib_function = lib.opendp_domains___extrinsic_domain_descriptor
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_domain), ExtrinsicObjectPtr))

    return output


def _lazyframe_from_domain(
    domain: Domain
):
    r"""Construct an empty LazyFrame with the same schema as in the LazyFrameDomain.

    This is useful for creating a dummy lazyframe used to write a query plan.

    [`_lazyframe_from_domain` in Rust documentation.](https://docs.rs/opendp/0.12.0-nightly.20250102.1/opendp/domains/fn._lazyframe_from_domain.html)

    :param domain: A LazyFrameDomain.
    :type domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=LazyFrameDomain)

    # Call library function.
    lib_function = lib.opendp_domains___lazyframe_from_domain
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_domain), AnyObjectPtr))

    return output


def atom_domain(
    bounds = None,
    nullable: bool = False,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Domain:
    r"""Construct an instance of `AtomDomain`.

    [`atom_domain` in Rust documentation.](https://docs.rs/opendp/0.12.0-nightly.20250102.1/opendp/domains/fn.atom_domain.html)

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


def bitvector_domain(
    max_weight = None
) -> Domain:
    r"""Construct an instance of `BitVectorDomain`.

    :param max_weight: The maximum number of positive bits.
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_max_weight = py_to_c(max_weight, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[u32]))

    # Call library function.
    lib_function = lib.opendp_domains__bitvector_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_max_weight), Domain))

    return output


def categorical_domain(
    categories = None
) -> Domain:
    r"""Construct an instance of `CategoricalDomain`.
    Can be used as an argument to a Polars series domain.

    :param categories: Optional ordered set of valid string categories
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[RuntimeType(origin='Vec', args=[String])]))

    # Call library function.
    lib_function = lib.opendp_domains__categorical_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_categories), Domain))

    return output


def datetime_domain(
    time_unit: str = "us",
    time_zone: Optional[str] = None
) -> Domain:
    r"""Construct an instance of `DatetimeDomain`.

    Documentation on valid time zones can be found [in the Polars documentation](https://docs.pola.rs/user-guide/transformations/time-series/timezones/).

    [`datetime_domain` in Rust documentation.](https://docs.rs/opendp/0.12.0-nightly.20250102.1/opendp/domains/fn.datetime_domain.html)

    :param time_unit: One of `ns`, `us` or `ms`, corresponding to nano-, micro-, and milliseconds
    :type time_unit: str
    :param time_zone: Optional time zone.
    :type time_zone: str
    :rtype: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_time_unit = py_to_c(time_unit, c_type=ctypes.c_char_p, type_name=str)
    c_time_zone = py_to_c(time_zone, c_type=ctypes.c_char_p, type_name=RuntimeType(origin='Option', args=[str]))

    # Call library function.
    lib_function = lib.opendp_domains__datetime_domain
    lib_function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_time_unit, c_time_zone), Domain))

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

    [`option_domain` in Rust documentation.](https://docs.rs/opendp/0.12.0-nightly.20250102.1/opendp/domains/fn.option_domain.html)

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

    [`series_domain` in Rust documentation.](https://docs.rs/opendp/0.12.0-nightly.20250102.1/opendp/domains/fn.series_domain.html)

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


    Required features: `honest-but-curious`

    **Why honest-but-curious?:**

    The identifier must uniquely identify this domain.
    If the identifier is not uniquely identifying,
    then two different domains with the same identifier will chain,
    which can violate transformation stability.

    In addition, the member function must:
    1. be a pure function
    2. be sound (only return true if its input is a member of the domain).

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
    c_member = py_to_c(member, c_type=CallbackFnPtr, type_name=bool)
    c_descriptor = py_to_c(descriptor, c_type=ExtrinsicObjectPtr, type_name=ExtrinsicObject)

    # Call library function.
    lib_function = lib.opendp_domains__user_domain
    lib_function.argtypes = [ctypes.c_char_p, CallbackFnPtr, ExtrinsicObjectPtr]
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


def wild_expr_domain(
    columns,
    by: Optional[list[str]] = None,
    max_partition_length = None,
    max_num_partitions = None,
    max_partition_contributions = None,
    max_influenced_partitions = None,
    public_info: Optional[str] = None
) -> Domain:
    r"""Construct a WildExprDomain.


    Required features: `contrib`

    :param columns: descriptors for each column in the data
    :param by: optional. Set if expression is applied to grouped data
    :type by: list[str]
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
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_columns = py_to_c(columns, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[SeriesDomain]))
    c_by = py_to_c(by, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[RuntimeType(origin='Vec', args=[String])]))
    c_max_partition_length = py_to_c(max_partition_length, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_max_num_partitions = py_to_c(max_num_partitions, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_max_partition_contributions = py_to_c(max_partition_contributions, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_max_influenced_partitions = py_to_c(max_influenced_partitions, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_public_info = py_to_c(public_info, c_type=ctypes.c_char_p, type_name=RuntimeType(origin='Option', args=[String]))

    # Call library function.
    lib_function = lib.opendp_domains__wild_expr_domain
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_columns, c_by, c_max_partition_length, c_max_num_partitions, c_max_partition_contributions, c_max_influenced_partitions, c_public_info), Domain))

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
    c_max_partition_length = py_to_c(max_partition_length, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_max_num_partitions = py_to_c(max_num_partitions, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_max_partition_contributions = py_to_c(max_partition_contributions, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_max_influenced_partitions = py_to_c(max_influenced_partitions, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_public_info = py_to_c(public_info, c_type=ctypes.c_char_p, type_name=RuntimeType(origin='Option', args=[String]))

    # Call library function.
    lib_function = lib.opendp_domains__with_margin
    lib_function.argtypes = [Domain, AnyObjectPtr, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_frame_domain, c_by, c_max_partition_length, c_max_num_partitions, c_max_partition_contributions, c_max_influenced_partitions, c_public_info), Domain))

    return output
