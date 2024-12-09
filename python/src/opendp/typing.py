'''
The ``typing`` module provides utilities that bridge between Python and Rust types.
OpenDP relies on precise descriptions of data types to make its security guarantees:
These are more natural in Rust with its fine-grained type system,
but they may feel out of place in Python. These utilities try to fill that gap.

For more context, see :ref:`typing in the User Guide <typing-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp
'''
from __future__ import annotations
import typing
from collections.abc import Hashable
from typing import Optional, Union, Any, Type, _GenericAlias # type: ignore[attr-defined]
from types import GenericAlias
import re

from opendp.mod import Function, UnknownTypeException, Measurement, Transformation, Domain, Metric, Measure
from opendp._lib import ATOM_EQUIVALENCE_CLASSES, import_optional_dependency


ELEMENTARY_TYPES: dict[Any, str] = {
    int: 'i32',
    float: 'f64',
    str: 'String',
    bool: 'bool',
    Measurement: 'AnyMeasurementPtr',
    Transformation: 'AnyTransformationPtr'
}
try:
    np = import_optional_dependency('numpy')
    # https://numpy.org/doc/stable/reference/arrays.scalars.html#sized-aliases
    ELEMENTARY_TYPES.update({
        # np.bytes_: '&[u8]',  # np.string_ # not used in OpenDP
        np.str_: 'String',  # np.unicode_
        np.bool_: 'bool',  # np.bool_
        np.int8: 'i8',  # np.byte
        np.int16: 'i16',  # np.short
        np.int32: 'i32',  # np.intc
        np.int64: 'i64',  # np.int_
        np.longlong: 'i128',
        np.uint8: 'u8',  # np.ubyte
        np.uint16: 'u16',  # np.ushort
        np.uint32: 'u32',  # np.uintc
        np.uint64: 'u64',
        np.ulonglong: 'u128',
        # np.intp: 'isize',  # not used in OpenDP
        # np.uintp: 'usize', # an alias for one of np.uint* that would overwrite the respective key
        # np.float16: 'f16',  # not used in OpenDP
        np.float32: 'f32',
        np.float64: 'f64',  # np.double, np.float_
    })
except ImportError: # pragma: no cover
    np = None # type: ignore[assignment]

INTEGER_TYPES = {"i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "usize"}
NUMERIC_TYPES = INTEGER_TYPES | {"f32", "f64"}
HASHABLE_TYPES = INTEGER_TYPES | {"bool", "String"}
PRIMITIVE_TYPES = NUMERIC_TYPES | {"bool", "String"}


# all ways of providing type information
RuntimeTypeDescriptor = Union[
    "RuntimeType",  # as the normalized type -- ChangeOneDistance; RuntimeType.parse("i32")
    str,  # plaintext string in terms of Rust types -- "Vec<i32>"
    Type[Union[list[Any], tuple[Any, Any], float, str, bool]],  # using the Python type class itself -- int, float
    tuple["RuntimeTypeDescriptor", ...],  # shorthand for tuples -- (float, "f64"); (ChangeOneDistance, list[int])
    _GenericAlias, # a Python type hint from the std typing module -- List[int]
    GenericAlias, # a Python type hint from the std types module -- list[int]
]


def set_default_int_type(T: RuntimeTypeDescriptor) -> None:
    """Set the default integer type throughout the library.
    This function is particularly useful when building computation chains with constructors.
    When you build a computation chain, any unspecified integer types default to this int type.

    The default int type is i32.
    
    :params T: must be one of [u8, u16, u32, u64, usize, i8, i16, i32, i64]
    :type T: :ref:`RuntimeTypeDescriptor`
    """
    equivalence_class = ATOM_EQUIVALENCE_CLASSES[ELEMENTARY_TYPES[int]]
    T = RuntimeType.parse(T)
    assert T in equivalence_class, f"T must be one of {equivalence_class}"

    ATOM_EQUIVALENCE_CLASSES[T] = ATOM_EQUIVALENCE_CLASSES.pop(ELEMENTARY_TYPES[int]) # type: ignore[index]
    ELEMENTARY_TYPES[int] = T # type: ignore[assignment]


def set_default_float_type(T: RuntimeTypeDescriptor) -> None:
    """Set the default float type throughout the library.
    This function is particularly useful when building computation chains with constructors.
    When you build a computation chain, any unspecified float types default to this float type.

    The default float type is f64.

    :params T: must be one of [f32, f64]
    :type T: :ref:`RuntimeTypeDescriptor`
    """

    equivalence_class = ATOM_EQUIVALENCE_CLASSES[ELEMENTARY_TYPES[float]]
    T = RuntimeType.parse(T)
    assert T in equivalence_class, f"T must be a float type in {equivalence_class}"

    ATOM_EQUIVALENCE_CLASSES[T] = ATOM_EQUIVALENCE_CLASSES.pop(ELEMENTARY_TYPES[float]) # type: ignore[index]
    ELEMENTARY_TYPES[float] = T # type: ignore[assignment]


class RuntimeType(object):
    """Utility for validating, manipulating, inferring and parsing/normalizing type information.
    """
    origin: str
    args: list[Union["RuntimeType", str]]

    def __init__(self, origin, args=None):
        if not isinstance(origin, str):
            raise ValueError("origin must be a string", origin)  # pragma: no cover
        self.origin = origin
        self.args = args or []

    def __eq__(self, other):
        if isinstance(other, str):
            other = RuntimeType.parse(other)
        if not isinstance(other, RuntimeType):
            return False
        return self.origin == other.origin and self.args == other.args

    def __repr__(self):
        result = self.origin or ''
        if result == 'Tuple':
            return f'({", ".join(map(str, self.args))})'
        if self.args:
            result += f'<{", ".join(map(str, self.args))}>'
        return result
    
    def __hash__(self) -> int:
        return hash(str(self))

    @classmethod
    def parse(cls, type_name: RuntimeTypeDescriptor, generics: Optional[list[str]] = None) -> Union["RuntimeType", str]:
        """Parse type descriptor into a normalized Rust type.

        Type descriptor may be expressed as:

        - Python type hints from std typing module
        - plaintext Rust type strings for setting specific bit depth
        - Python type class - one of {int, str, float, bool}
        - tuple of type information - for example: (float, float)

        :param type_name: type specifier
        :param generics: For internal use. List of type names to consider generic when parsing.
        :type: list[str]
        :return: Normalized type. If the type has subtypes, returns a RuntimeType, else a str.
        :rtype: Union["RuntimeType", str]
        :raises UnknownTypeException: if `type_name` fails to parse

        :examples:

        >>> dp.RuntimeType.parse(int)
        'i32'
        >>> dp.RuntimeType.parse("i32")
        'i32'
        >>> dp.RuntimeType.parse(L1Distance[int])
        L1Distance<i32>
        >>> dp.RuntimeType.parse(L1Distance["f32"])
        L1Distance<f32>
        """
        generics = generics or []
        if isinstance(type_name, RuntimeType):
            return type_name

        # parse type hints from the typing module
        hinted_type = None
        if isinstance(type_name, _GenericAlias):
            hinted_type = typing.get_origin(type_name), typing.get_args(type_name)
        if isinstance(type_name, GenericAlias): # type: ignore[attr-defined]
            hinted_type = type_name.__origin__, type_name.__args__ # type: ignore[attr-defined]
    
        if hinted_type:
            origin, args = hinted_type
            args = [RuntimeType.parse(v, generics=generics) for v in args] or None # type: ignore[assignment]
            if origin == tuple:
                origin = 'Tuple'
            elif origin == list:
                origin = 'Vec'
            elif origin == dict:
                origin = 'HashMap'
            
            return RuntimeType(RuntimeType.parse(origin, generics=generics), args)

        # parse a tuple of types-- (int, "f64"); (list[int], (int, bool))
        if isinstance(type_name, tuple):
            return RuntimeType('Tuple', list(cls.parse(v, generics=generics) for v in type_name))

        # parse a string-- "Vec<f32>",
        if isinstance(type_name, str):
            type_name = type_name.strip()
            if type_name in generics:
                return GenericType(type_name)
            if type_name.startswith('(') and type_name.endswith(')'):
                return RuntimeType('Tuple', cls._parse_args(type_name[1:-1], generics=generics))
            start, end = type_name.find('<'), type_name.rfind('>')

            # attempt to upgrade strings to the metric instance
            origin = type_name[:start] if 0 < start else type_name
            closeness: RuntimeType = { # type: ignore[assignment]
                'ChangeOneDistance': ChangeOneDistance,
                'SymmetricDistance': SymmetricDistance,
                'AbsoluteDistance': AbsoluteDistance,
                'L1Distance': L1Distance,
                'L2Distance': L2Distance,
            }.get(origin)
            if closeness is not None:
                if isinstance(closeness, SensitivityMetric):
                    return closeness[cls._parse_args(type_name[start + 1: end], generics=generics)[0]]
                return closeness

            domain = {
                'AtomDomain': AtomDomain,
                'VectorDomain': VectorDomain,
                'MapDomain': MapDomain,
                'OptionDomain': OptionDomain,
            }.get(origin)
            if domain is not None:
                return domain[cls._parse_args(type_name[start + 1: end], generics=generics)[0]]

            if 0 < start < end < len(type_name):
                return RuntimeType(origin, args=cls._parse_args(type_name[start + 1: end], generics=generics))
            if start == end < 0:
                if type_name == "int":
                    return ELEMENTARY_TYPES[int]
                if type_name == "float":
                    return ELEMENTARY_TYPES[float]
                return type_name

        if isinstance(type_name, Hashable) and type_name in ELEMENTARY_TYPES:
            return ELEMENTARY_TYPES[type_name]

        if type_name == tuple:
            raise UnknownTypeException("non-parameterized argument")  # pragma: no cover

        raise UnknownTypeException(f"unable to parse type: {type_name}")

    @classmethod
    def _parse_args(cls, args, generics: Optional[list[str]] = None):
        import re
        return [cls.parse(v, generics=generics) for v in re.split(r",\s*(?![^()<>]*\))", args)]

    @classmethod
    def infer(cls, public_example: Any, py_object=False) -> Union["RuntimeType", str]:
        """Infer the normalized type from a public example.

        :param public_example: data used to infer the type
        :param py_object: return "ExtrinsicObject" when type not recognized, instead of error
        :return: Normalized type. If the type has subtypes, returns a RuntimeType, else a str.
        :rtype: Union["RuntimeType", str]
        :raises UnknownTypeException: if inference fails on `public_example`

        :examples:

        >>> dp.RuntimeType.infer(23)
        'i32'
        >>> dp.RuntimeType.infer(12.)
        'f64'
        >>> dp.RuntimeType.infer(["A", "B"])
        Vec<String>
        >>> dp.RuntimeType.infer((12., True, "A"))
        (f64, bool, String)
        
        >>> dp.RuntimeType.infer([])
        Traceback (most recent call last):
        ...
        opendp.mod.UnknownTypeException: Cannot infer atomic type when empty
        """
        if type(public_example) in ELEMENTARY_TYPES:
            return ELEMENTARY_TYPES[type(public_example)]
        
        if isinstance(public_example, (Domain, Metric, Measure)):
            return RuntimeType.parse(public_example.type)
        
        pl = import_optional_dependency("polars", raise_error=False)
        if pl is not None:
            if isinstance(public_example, pl.LazyFrame):
                return LazyFrame
            
            if isinstance(public_example, pl.DataFrame):
                return DataFrame
            
            if isinstance(public_example, pl.Series):
                return Series
            
            if isinstance(public_example, pl.Expr):
                return Expr

        if isinstance(public_example, tuple):
            return RuntimeType('Tuple', [cls.infer(e, py_object) for e in public_example])

        def infer_homogeneous(value):
            types = {cls.infer(v, py_object=py_object) for v in value}

            if len(types) == 0:
                raise UnknownTypeException("Cannot infer atomic type when empty")
            if len(types) == 1:
                return next(iter(types))
            if py_object:
                return "ExtrinsicObject"
            raise TypeError(f"elements must be homogeneously typed. Found {types}")
        
        if isinstance(public_example, list):
            return RuntimeType('Vec', [infer_homogeneous(public_example)])

        if np is not None and isinstance(public_example, np.ndarray):
            if public_example.ndim == 0:
                return cls.infer(public_example.item(), py_object)

            if public_example.ndim == 1:
                inner_type = ELEMENTARY_TYPES.get(public_example.dtype.type)
                if inner_type is None:
                    raise UnknownTypeException(f"Unknown numpy array dtype: {public_example.dtype.type}")  # pragma: no cover
                return RuntimeType('Vec', [inner_type])

            raise UnknownTypeException("arrays with greater than one axis are not yet supported")  # pragma: no cover

        if isinstance(public_example, dict):
            return RuntimeType('HashMap', [
                infer_homogeneous(public_example.keys()),
                infer_homogeneous(public_example.values())
            ])

        if public_example is None:
            raise UnknownTypeException("Type of Option cannot be inferred from None")
        
        if callable(public_example):
            return "CallbackFn"

        if py_object:
            return "ExtrinsicObject"
        raise UnknownTypeException(type(public_example))

    @classmethod
    def parse_or_infer(
            cls,
            type_name: RuntimeTypeDescriptor | None = None,
            public_example: Any = None,
            generics: Optional[list[str]] = None
    ) -> Union["RuntimeType", str]:
        """If type_name is supplied, normalize it. Otherwise, infer the normalized type from a public example.

        :param type_name: type specifier. See RuntimeType.parse for documentation on valid inputs
        :param public_example: data used to infer the type
        :return: Normalized type. If the type has subtypes, returns a RuntimeType, else a str.
        :rtype: Union["RuntimeType", str]
        :param generics: For internal use. List of type names to consider generic when parsing.
        :type: list[str]
        :raises ValueError: if `type_name` fails to parse
        :raises UnknownTypeException: if inference fails on `public_example` or no args are supplied
        """
        if type_name is not None:
            return cls.parse(type_name, generics)
        if public_example is not None:
            return cls.infer(public_example)
        raise UnknownTypeException("either type_name or public_example must be passed")  # pragma: no cover

    def substitute(self: Union["RuntimeType", str], **kwargs):
        if isinstance(self, GenericType):
            return kwargs.get(self.origin, self)
        if isinstance(self, RuntimeType):
            return RuntimeType(self.origin, self.args and [RuntimeType.substitute(arg, **kwargs) for arg in self.args])
        return self
    

class GenericType(RuntimeType):
    def __repr__(self):
        raise UnknownTypeException(f"attempted to create a type_name with an unknown generic: {self.origin}")  # pragma: no cover


SymmetricDistance = 'SymmetricDistance'
InsertDeleteDistance = 'InsertDeleteDistance'
ChangeOneDistance = 'ChangeOneDistance'
HammingDistance = 'HammingDistance'

DiscreteDistance = 'DiscreteDistance'


class SensitivityMetric(RuntimeType):
    """All sensitivity RuntimeTypes inherit from SensitivityMetric.
    Provides static type checking in user-code for sensitivity metrics and a getitem interface like stdlib typing.
    """
    def __getitem__(self, associated_type):
        return SensitivityMetric(self.origin, [self.parse(type_name=associated_type)])


AbsoluteDistance: SensitivityMetric = SensitivityMetric('AbsoluteDistance')
L1Distance: SensitivityMetric = SensitivityMetric('L1Distance')
L2Distance: SensitivityMetric = SensitivityMetric('L2Distance')


MaxDivergence = 'MaxDivergence'
SmoothedMaxDivergence = 'SmoothedMaxDivergence'
FixedSmoothedMaxDivergence = 'FixedSmoothedMaxDivergence'
ZeroConcentratedDivergence = 'ZeroConcentratedDivergence'

class Carrier(RuntimeType):
    def __getitem__(self, subdomains):
        if not isinstance(subdomains, tuple):
            subdomains = (subdomains,)
        return Carrier(self.origin, [self.parse(type_name=subdomain) for subdomain in subdomains])


Vec: Carrier = Carrier('Vec')
HashMap: Carrier = Carrier('HashMap')
i8: str = 'i8'
i16: str = 'i16'
i32: str = 'i32'
i64: str = 'i64'
i128: str = 'i128'
isize: str = 'isize'
u8: str = 'u8'
u16: str = 'u16'
u32: str = 'u32'
u64: str = 'u64'
u128: str = 'u128'
usize: str = 'usize'
f32: str = 'f32'
f64: str = 'f64'
String: str = 'String'
BitVector: str = 'BitVector'
LazyFrame: str = 'LazyFrame'
DataFrame: str = 'DataFrame'
Series: str = 'Series'
Expr: str = 'Expr'
AnyMeasurementPtr: str = 'AnyMeasurementPtr'
AnyTransformationPtr: str = 'AnyTransformationPtr'
LazyFrameDomain: str = 'LazyFrame'
SeriesDomain: str = 'SeriesDomain'

class DomainDescriptor(RuntimeType):
    def __getitem__(self, subdomain):
        if not isinstance(subdomain, tuple):
            subdomain = (subdomain,)
        return DomainDescriptor(self.origin, [self.parse(type_name=sub_i) for sub_i in subdomain])

    def __call__(self, *args, **kwargs):
        '''
        >>> FakeDomain = DomainDescriptor('FakeDomain')
        >>> FakeDomain(int)
        Traceback (most recent call last):
        ...
        Exception: Use dp.fake_domain to construst a new FakeDomain
        '''
        # https://stackoverflow.com/a/12867228/10727889
        lc_name = re.sub('(?!^)([A-Z])', r'_\1', self.origin).lower()
        raise Exception(f'Use dp.{lc_name} to construst a new {self.origin}')


AtomDomain: DomainDescriptor = DomainDescriptor('AtomDomain')
VectorDomain: DomainDescriptor = DomainDescriptor('VectorDomain')
OptionDomain: DomainDescriptor = DomainDescriptor('OptionDomain')
SizedDomain: DomainDescriptor = DomainDescriptor('SizedDomain')
MapDomain: DomainDescriptor = DomainDescriptor('MapDomain')


def get_atom(type_name):
    type_name = RuntimeType.parse(type_name)
    while isinstance(type_name, RuntimeType):
        if isinstance(type_name, GenericType):
            return
        type_name = type_name.args[0]
    return type_name


def get_atom_or_infer(type_name: Union[RuntimeType, str], example):
    return get_atom(type_name) or RuntimeType.infer(example)


def get_first(value):
    if value is None or not len(value):
        return None
    return next(iter(value))

def parse_or_infer(type_name: RuntimeTypeDescriptor | None, example) -> Union[RuntimeType, str]:
    return RuntimeType.parse_or_infer(type_name, example)

def pass_through(value: Any) -> Any:
    return value

def get_dependencies(value: Union[Measurement, Transformation, Function]) -> Any:
    return getattr(value, "_dependencies", None)

def get_dependencies_iterable(value: list[Union[Measurement, Transformation, Function]]) -> list[Any]:
    return list(map(get_dependencies, value))

def get_carrier_type(value: Domain) -> Union[RuntimeType, str]:
    return value.carrier_type


def get_type(value):
    return value.type

def get_value_type(type_name):
    return RuntimeType.parse(type_name).args[1] # type: ignore[union-attr]

def get_distance_type(value: Union[Metric, Measure]) -> Union[RuntimeType, str]:
    return value.distance_type
