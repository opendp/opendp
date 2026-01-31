# Auto-generated. Do not edit!
'''
The ``domains`` module provides functions for creating and using domains.
For more context, see :ref:`domains in the User Guide <domains-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp
'''
from deprecated.sphinx import deprecated # noqa: F401 (Not every file actually has deprecated functions.)

from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.typing import _substitute # noqa: F401

__all__ = [
    "_atom_domain_get_bounds_closed",
    "_atom_domain_nan",
    "_domain_equal",
    "_domain_free",
    "_extrinsic_domain_descriptor",
    "_lazyframe_domain_get_columns",
    "_lazyframe_domain_get_margin",
    "_lazyframe_domain_get_series_domain",
    "_lazyframe_from_domain",
    "_member",
    "_option_domain_get_element_domain",
    "_series_domain_get_element_domain",
    "_series_domain_get_name",
    "_series_domain_get_nullable",
    "_vector_domain_get_element_domain",
    "_vector_domain_get_size",
    "array_domain",
    "atom_domain",
    "bitvector_domain",
    "categorical_domain",
    "datetime_domain",
    "domain_carrier_type",
    "domain_debug",
    "domain_type",
    "enum_domain",
    "lazyframe_domain",
    "map_domain",
    "option_domain",
    "series_domain",
    "user_domain",
    "vector_domain",
    "wild_expr_domain",
    "with_margin"
]


def _atom_domain_get_bounds_closed(
    domain: Domain
):
    r"""Retrieve bounds from an AtomDomain<T>

    [_atom_domain_get_bounds_closed in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/domains/fn._atom_domain_get_bounds_closed.html)

    .. end-markdown

    :param domain: 
    :type domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=None)

    # Call library function.
    lib_function = lib.opendp_domains___atom_domain_get_bounds_closed
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_domain), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_atom_domain_get_bounds_closed',
            '__module__': 'domains',
            '__kwargs__': {
                'domain': domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _atom_domain_nan(
    domain: Domain
):
    r"""Retrieve whether members of AtomDomain<T> may be NaN.

    [_atom_domain_nan in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/domains/fn._atom_domain_nan.html)

    .. end-markdown

    :param domain: 
    :type domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=None)

    # Call library function.
    lib_function = lib.opendp_domains___atom_domain_nan
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_domain), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_atom_domain_nan',
            '__module__': 'domains',
            '__kwargs__': {
                'domain': domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _domain_equal(
    left: Domain,
    right: Domain
) -> bool:
    r"""Check whether two domains are equal.

    .. end-markdown

    :param left: Domain to compare.
    :type left: Domain
    :param right: Domain to compare.
    :type right: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_left = py_to_c(left, c_type=Domain, type_name="AnyDomain")
    c_right = py_to_c(right, c_type=Domain, type_name="AnyDomain")

    # Call library function.
    lib_function = lib.opendp_domains___domain_equal
    lib_function.argtypes = [Domain, Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_left, c_right), BoolPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_domain_equal',
            '__module__': 'domains',
            '__kwargs__': {
                'left': left, 'right': right
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _domain_free(
    this
):
    r"""Internal function. Free the memory associated with `this`.

    .. end-markdown

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
    try:
        output.__opendp_dict__ = {
            '__function__': '_domain_free',
            '__module__': 'domains',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _extrinsic_domain_descriptor(
    domain: Domain
):
    r"""Retrieve the descriptor value stored in an extrinsic domain.

    .. end-markdown

    :param domain: The ExtrinsicDomain to extract the descriptor from
    :type domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name="AnyDomain")

    # Call library function.
    lib_function = lib.opendp_domains___extrinsic_domain_descriptor
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_domain), ExtrinsicObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_extrinsic_domain_descriptor',
            '__module__': 'domains',
            '__kwargs__': {
                'domain': domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _lazyframe_domain_get_columns(
    lazyframe_domain: Domain
):
    r"""Retrieve the column names of the LazyFrameDomain.

    .. end-markdown

    :param lazyframe_domain: Domain to retrieve the column names from
    :type lazyframe_domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_lazyframe_domain = py_to_c(lazyframe_domain, c_type=Domain, type_name="AnyDomain")

    # Call library function.
    lib_function = lib.opendp_domains___lazyframe_domain_get_columns
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_lazyframe_domain), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_lazyframe_domain_get_columns',
            '__module__': 'domains',
            '__kwargs__': {
                'lazyframe_domain': lazyframe_domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _lazyframe_domain_get_margin(
    lazyframe_domain: Domain,
    by
):
    r"""Retrieve the series domain at index 'column`.

    .. end-markdown

    :param lazyframe_domain: Domain to retrieve the SeriesDomain from
    :type lazyframe_domain: Domain
    :param by: grouping columns
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_lazyframe_domain = py_to_c(lazyframe_domain, c_type=Domain, type_name="AnyDomain")
    c_by = py_to_c(by, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=["Expr"]))

    # Call library function.
    lib_function = lib.opendp_domains___lazyframe_domain_get_margin
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_lazyframe_domain, c_by), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_lazyframe_domain_get_margin',
            '__module__': 'domains',
            '__kwargs__': {
                'lazyframe_domain': lazyframe_domain, 'by': by
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _lazyframe_domain_get_series_domain(
    lazyframe_domain: Domain,
    name: str
) -> Domain:
    r"""Retrieve the series domain at index `column`.

    .. end-markdown

    :param lazyframe_domain: Domain to retrieve the SeriesDomain from
    :type lazyframe_domain: Domain
    :param name: Name of the SeriesDomain to retrieve
    :type name: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_lazyframe_domain = py_to_c(lazyframe_domain, c_type=Domain, type_name="AnyDomain")
    c_name = py_to_c(name, c_type=ctypes.c_char_p, type_name="c_char")

    # Call library function.
    lib_function = lib.opendp_domains___lazyframe_domain_get_series_domain
    lib_function.argtypes = [Domain, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_lazyframe_domain, c_name), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': '_lazyframe_domain_get_series_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'lazyframe_domain': lazyframe_domain, 'name': name
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _lazyframe_from_domain(
    domain: Domain
):
    r"""Construct an empty LazyFrame with the same schema as in the LazyFrameDomain.

    This is useful for creating a dummy lazyframe used to write a query plan.

    [_lazyframe_from_domain in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/domains/fn._lazyframe_from_domain.html)

    .. end-markdown

    :param domain: A LazyFrameDomain.
    :type domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name="LazyFrameDomain")

    # Call library function.
    lib_function = lib.opendp_domains___lazyframe_from_domain
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_domain), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_lazyframe_from_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'domain': domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _member(
    this: Domain,
    val
) -> bool:
    r"""Check membership in a `domain`.

    .. end-markdown

    :param this: The domain to check membership in.
    :type this: Domain
    :param val: A potential element of the domain.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Domain, type_name="AnyDomain")
    c_val = py_to_c(val, c_type=AnyObjectPtr, type_name=domain_carrier_type(this))

    # Call library function.
    lib_function = lib.opendp_domains___member
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this, c_val), BoolPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_member',
            '__module__': 'domains',
            '__kwargs__': {
                'this': this, 'val': val
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _option_domain_get_element_domain(
    option_domain: Domain
) -> Domain:
    r"""Retrieve the element domain of the option domain.

    .. end-markdown

    :param option_domain: The option domain from which to retrieve the element domain
    :type option_domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_option_domain = py_to_c(option_domain, c_type=Domain, type_name="AnyDomain")

    # Call library function.
    lib_function = lib.opendp_domains___option_domain_get_element_domain
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_option_domain), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': '_option_domain_get_element_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'option_domain': option_domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _series_domain_get_element_domain(
    series_domain: Domain
) -> Domain:
    r"""Retrieve the element domain of the series domain.

    [_series_domain_get_element_domain in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/domains/fn._series_domain_get_element_domain.html)

    .. end-markdown

    :param series_domain: The series domain from which to retrieve the element domain
    :type series_domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_series_domain = py_to_c(series_domain, c_type=Domain, type_name=None)

    # Call library function.
    lib_function = lib.opendp_domains___series_domain_get_element_domain
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_series_domain), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': '_series_domain_get_element_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'series_domain': series_domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _series_domain_get_name(
    series_domain: Domain
):
    r"""

    .. end-markdown

    :param series_domain: The series domain from which to retrieve the name of elements
    :type series_domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_series_domain = py_to_c(series_domain, c_type=Domain, type_name=None)

    # Call library function.
    lib_function = lib.opendp_domains___series_domain_get_name
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_series_domain), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_series_domain_get_name',
            '__module__': 'domains',
            '__kwargs__': {
                'series_domain': series_domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _series_domain_get_nullable(
    series_domain: Domain
):
    r"""Retrieve whether elements in members of the domain may be null.

    .. end-markdown

    :param series_domain: The series domain from which to check nullability.
    :type series_domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_series_domain = py_to_c(series_domain, c_type=Domain, type_name=None)

    # Call library function.
    lib_function = lib.opendp_domains___series_domain_get_nullable
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_series_domain), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_series_domain_get_nullable',
            '__module__': 'domains',
            '__kwargs__': {
                'series_domain': series_domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _vector_domain_get_element_domain(
    vector_domain: Domain
) -> Domain:
    r"""Retrieve the element domain of the vector domain.

    .. end-markdown

    :param vector_domain: The vector domain from which to retrieve the element domain
    :type vector_domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_vector_domain = py_to_c(vector_domain, c_type=Domain, type_name="AnyDomain")

    # Call library function.
    lib_function = lib.opendp_domains___vector_domain_get_element_domain
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_vector_domain), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': '_vector_domain_get_element_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'vector_domain': vector_domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _vector_domain_get_size(
    vector_domain: Domain
):
    r"""Retrieve the size of vectors in the vector domain.

    .. end-markdown

    :param vector_domain: The vector domain from which to retrieve the size
    :type vector_domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_vector_domain = py_to_c(vector_domain, c_type=Domain, type_name="AnyDomain")

    # Call library function.
    lib_function = lib.opendp_domains___vector_domain_get_size
    lib_function.argtypes = [Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_vector_domain), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_vector_domain_get_size',
            '__module__': 'domains',
            '__kwargs__': {
                'vector_domain': vector_domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def array_domain(
    element_domain: Domain,
    width: int
) -> Domain:
    r"""Construct an instance of `ArrayDomain`.
    Can be used as an argument to a Polars series domain.

    .. end-markdown

    :param element_domain: The domain of each element in the array.
    :type element_domain: Domain
    :param width: The width of the array.
    :type width: int
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_element_domain = py_to_c(element_domain, c_type=Domain, type_name="AnyDomain")
    c_width = py_to_c(width, c_type=ctypes.c_uint32, type_name="u32")

    # Call library function.
    lib_function = lib.opendp_domains__array_domain
    lib_function.argtypes = [Domain, ctypes.c_uint32]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_element_domain, c_width), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'array_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'element_domain': element_domain, 'width': width
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def atom_domain(
    bounds = None,
    nan = None,
    T: Optional[RuntimeTypeDescriptor] = None
) -> AtomDomain:
    r"""Construct an instance of `AtomDomain`.

    The domain defaults to unbounded if `bounds` is `None`,
    If `T` is float, `nan` defaults to `true`.

    [atom_domain in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/domains/struct.AtomDomain.html)

    .. end-markdown

    :param bounds: Optional bounds of elements in the domain, if the data type is numeric.
    :param nan: Whether the domain may contain NaN, if the data type is float.
    :param T: The type of the atom.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

        >>> dp.atom_domain(T=float, nan=False)
        AtomDomain(T=f64)

    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))

    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[RuntimeType(origin='Tuple', args=[T, T])]))
    c_nan = py_to_c(nan, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=["bool"]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_domains__atom_domain
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_bounds, c_nan, c_T), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'atom_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'bounds': bounds, 'nan': nan, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def bitvector_domain(
    max_weight = None
) -> Domain:
    r"""Construct an instance of `BitVectorDomain`.

    .. end-markdown

    :param max_weight: The maximum number of positive bits.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_max_weight = py_to_c(max_weight, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=["u32"]))

    # Call library function.
    lib_function = lib.opendp_domains__bitvector_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_max_weight), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'bitvector_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'max_weight': max_weight
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def categorical_domain(
    categories = None
) -> Domain:
    r"""Construct an instance of `CategoricalDomain`.
    Can be used as an argument to a Polars series domain.

    .. end-markdown

    :param categories: Optional ordered set of valid string categories
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[RuntimeType(origin='Vec', args=["String"])]))

    # Call library function.
    lib_function = lib.opendp_domains__categorical_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_categories), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'categorical_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'categories': categories
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def datetime_domain(
    time_unit: str = "us",
    time_zone: Optional[str] = None
) -> Domain:
    r"""Construct an instance of `DatetimeDomain`.

    Documentation on valid time zones can be found [in the Polars documentation](https://docs.pola.rs/user-guide/transformations/time-series/timezones/).

    [datetime_domain in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/domains/struct.DatetimeDomain.html)

    .. end-markdown

    :param time_unit: One of ``ns``, ``us`` or ``ms``, corresponding to nano-, micro-, and milliseconds
    :type time_unit: str
    :param time_zone: Optional time zone.
    :type time_zone: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_time_unit = py_to_c(time_unit, c_type=ctypes.c_char_p, type_name="str")
    c_time_zone = py_to_c(time_zone, c_type=ctypes.c_char_p, type_name=RuntimeType(origin='Option', args=["str"]))

    # Call library function.
    lib_function = lib.opendp_domains__datetime_domain
    lib_function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_time_unit, c_time_zone), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'datetime_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'time_unit': time_unit, 'time_zone': time_zone
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def domain_carrier_type(
    this: Domain
) -> str:
    r"""Get the carrier type of a `domain`.

    .. end-markdown

    :param this: The domain to retrieve the carrier type from.
    :type this: Domain
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'domain_carrier_type',
            '__module__': 'domains',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def domain_debug(
    this: Domain
) -> str:
    r"""Debug a `domain`.

    .. end-markdown

    :param this: The domain to debug (stringify).
    :type this: Domain
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'domain_debug',
            '__module__': 'domains',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def domain_type(
    this: Domain
) -> str:
    r"""Get the type of a `domain`.

    .. end-markdown

    :param this: The domain to retrieve the type from.
    :type this: Domain
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'domain_type',
            '__module__': 'domains',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def enum_domain(
    categories
) -> Domain:
    r"""Construct an instance of `EnumDomain`.
    Can be used as an argument to a Polars series domain.

    .. end-markdown

    :param categories: Optional ordered set of string categories
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name="Series")

    # Call library function.
    lib_function = lib.opendp_domains__enum_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_categories), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'enum_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'categories': categories
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def lazyframe_domain(
    series_domains
) -> LazyFrameDomain:
    r"""Construct an instance of `LazyFrameDomain`.

    .. end-markdown

    :param series_domains: Domain of each series in the lazyframe.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_series_domains = py_to_c(series_domains, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=["SeriesDomain"]))

    # Call library function.
    lib_function = lib.opendp_domains__lazyframe_domain
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_series_domains), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'lazyframe_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'series_domains': series_domains
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def map_domain(
    key_domain: Domain,
    value_domain: Domain
) -> Domain:
    r"""Construct an instance of `MapDomain`.

    .. end-markdown

    :param key_domain: domain of keys in the hashmap
    :type key_domain: Domain
    :param value_domain: domain of values in the hashmap
    :type value_domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_key_domain = py_to_c(key_domain, c_type=Domain, type_name="AnyDomain")
    c_value_domain = py_to_c(value_domain, c_type=Domain, type_name="AnyDomain")

    # Call library function.
    lib_function = lib.opendp_domains__map_domain
    lib_function.argtypes = [Domain, Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_key_domain, c_value_domain), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'map_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'key_domain': key_domain, 'value_domain': value_domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def option_domain(
    element_domain: Domain,
    D: Optional[RuntimeTypeDescriptor] = None
) -> OptionDomain:
    r"""Construct an instance of `OptionDomain`.

    [option_domain in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/domains/struct.OptionDomain.html)

    .. end-markdown

    :param element_domain: 
    :type element_domain: Domain
    :param D: The type of the inner domain.
    :type D: :py:ref:`RuntimeTypeDescriptor`
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'option_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'element_domain': element_domain, 'D': D
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def series_domain(
    name: str,
    element_domain: Domain
) -> SeriesDomain:
    r"""Construct an instance of `SeriesDomain`.

    [series_domain in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/domains/struct.SeriesDomain.html)

    .. end-markdown

    :param name: The name of the series.
    :type name: str
    :param element_domain: The domain of elements in the series.
    :type element_domain: Domain
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_name = py_to_c(name, c_type=ctypes.c_char_p, type_name="str")
    c_element_domain = py_to_c(element_domain, c_type=Domain, type_name=None)

    # Call library function.
    lib_function = lib.opendp_domains__series_domain
    lib_function.argtypes = [ctypes.c_char_p, Domain]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_name, c_element_domain), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'series_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'name': name, 'element_domain': element_domain
            },
        }
    except AttributeError:  # pragma: no cover
        pass
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

    .. end-markdown

    :param identifier: A string description of the data domain.
    :type identifier: str
    :param member: A function used to test if a value is a member of the data domain.
    :param descriptor: Additional constraints on the domain.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("honest-but-curious")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_identifier = py_to_c(identifier, c_type=ctypes.c_char_p, type_name=None)
    c_member = py_to_c(member, c_type=CallbackFnPtr, type_name="bool")
    c_descriptor = py_to_c(descriptor, c_type=ExtrinsicObjectPtr, type_name="ExtrinsicObject")

    # Call library function.
    lib_function = lib.opendp_domains__user_domain
    lib_function.argtypes = [ctypes.c_char_p, CallbackFnPtr, ExtrinsicObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_identifier, c_member, c_descriptor), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'user_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'identifier': identifier, 'member': member, 'descriptor': descriptor
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def vector_domain(
    atom_domain: Domain,
    size = None
) -> VectorDomain:
    r"""Construct an instance of `VectorDomain`.

    .. end-markdown

    :param atom_domain: The inner domain.
    :type atom_domain: Domain
    :param size: 
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_atom_domain = py_to_c(atom_domain, c_type=Domain, type_name=None)
    c_size = py_to_c(size, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=["i32"]))

    # Call library function.
    lib_function = lib.opendp_domains__vector_domain
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_atom_domain, c_size), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'vector_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'atom_domain': atom_domain, 'size': size
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def wild_expr_domain(
    columns,
    margin = None
) -> Domain:
    r"""Construct a WildExprDomain.


    Required features: `contrib`

    .. end-markdown

    :param columns: descriptors for each column in the data
    :param margin: descriptors for grouped data
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_columns = py_to_c(columns, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=["SeriesDomain"]))
    c_margin = py_to_c(margin, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=["Margin"]))

    # Call library function.
    lib_function = lib.opendp_domains__wild_expr_domain
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_columns, c_margin), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'wild_expr_domain',
            '__module__': 'domains',
            '__kwargs__': {
                'columns': columns, 'margin': margin
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def with_margin(
    frame_domain: Domain,
    margin
) -> LazyFrameDomain:
    r"""

    .. end-markdown

    :param frame_domain: 
    :type frame_domain: Domain
    :param margin: 
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_frame_domain = py_to_c(frame_domain, c_type=Domain, type_name=None)
    c_margin = py_to_c(margin, c_type=AnyObjectPtr, type_name="Margin")

    # Call library function.
    lib_function = lib.opendp_domains__with_margin
    lib_function.argtypes = [Domain, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_frame_domain, c_margin), Domain))
    try:
        output.__opendp_dict__ = {
            '__function__': 'with_margin',
            '__module__': 'domains',
            '__kwargs__': {
                'frame_domain': frame_domain, 'margin': margin
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output
