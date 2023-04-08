import sys
import typing
from collections.abc import Hashable
from typing import Union, Any, Type, List

from opendp.mod import UnknownTypeException, Measurement, Transformation, Domain, Metric, Measure
from opendp._lib import ATOM_EQUIVALENCE_CLASSES

ELEMENTARY_TYPES = {
    int: 'i32',
    float: 'f64',
    str: 'String',
    bool: 'bool',
    Measurement: 'AnyMeasurementPtr',
    Transformation: 'AnyTransformationPtr'
}
try:
    import numpy as np
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
except ImportError:
    np = None

INTEGER_TYPES = {"i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "usize"}
NUMERIC_TYPES = INTEGER_TYPES | {"f32", "f64"}
HASHABLE_TYPES = INTEGER_TYPES | {"bool", "String"}
PRIMITIVE_TYPES = NUMERIC_TYPES | {"bool", "String"}


# all ways of providing type information
RuntimeTypeDescriptor = Union[
    "RuntimeType",  # as the normalized type -- ChangeOneDistance; RuntimeType.parse("i32")
    str,  # plaintext string in terms of Rust types -- "Vec<i32>"
    Type[Union[typing.List, typing.Tuple, int, float, str, bool]],  # using the Python type class itself -- int, float
    tuple,  # shorthand for tuples -- (float, "f64"); (ChangeOneDistance, List[int])
]

if sys.version_info >= (3, 8):
    from typing import _GenericAlias
    # a Python type hint from the std typing module -- List[int]
    RuntimeTypeDescriptor.__args__ = RuntimeTypeDescriptor.__args__ + (_GenericAlias,)

if sys.version_info >= (3, 9):
    from types import GenericAlias
    # a Python type hint from the std types module -- list[int]
    RuntimeTypeDescriptor.__args__ = RuntimeTypeDescriptor.__args__ + (GenericAlias,)


def set_default_int_type(T: RuntimeTypeDescriptor):
    """Set the default integer type throughout the library.
    This function is particularly useful when building computation chains with constructors.
    When you build a computation chain, any unspecified integer types default to this int type.

    The default int type is i32.
    
    :params T: must be one of [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    :type T: :ref:`RuntimeTypeDescriptor`
    """
    equivalence_class = ATOM_EQUIVALENCE_CLASSES[ELEMENTARY_TYPES[int]]
    assert T in equivalence_class, f"T must be one of {equivalence_class}"

    ATOM_EQUIVALENCE_CLASSES[T] = ATOM_EQUIVALENCE_CLASSES.pop(ELEMENTARY_TYPES[int])
    ELEMENTARY_TYPES[int] = T


def set_default_float_type(T: RuntimeTypeDescriptor):
    """Set the default float type throughout the library.
    This function is particularly useful when building computation chains with constructors.
    When you build a computation chain, any unspecified float types default to this float type.

    The default float type is f64.

    :params T: must be one of [f32, f64]
    :type T: :ref:`RuntimeTypeDescriptor`
    """

    equivalence_class = ATOM_EQUIVALENCE_CLASSES[ELEMENTARY_TYPES[float]]
    assert T in equivalence_class, f"T must be a float type in {equivalence_class}"

    ATOM_EQUIVALENCE_CLASSES[T] = ATOM_EQUIVALENCE_CLASSES.pop(ELEMENTARY_TYPES[float])
    ELEMENTARY_TYPES[float] = T


class RuntimeType(object):
    """Utility for validating, manipulating, inferring and parsing/normalizing type information.
    """

    def __init__(self, origin, args=None):
        if not isinstance(origin, str):
            raise ValueError("origin must be a string", origin)
        self.origin = origin
        self.args = args

    def __eq__(self, other):
        if isinstance(other, str):
            other = RuntimeType.parse(other)
        if isinstance(other, str):
            return False
        return self.origin == other.origin and self.args == other.args

    def __str__(self):
        result = self.origin or ''
        if result == 'Tuple':
            return f'({", ".join(map(str, self.args))})'
        if self.args:
            result += f'<{", ".join(map(str, self.args))}>'
        return result

    @classmethod
    def parse(cls, type_name: RuntimeTypeDescriptor, generics: List[str] = None) -> Union["RuntimeType", str]:
        """Parse type descriptor into a normalized Rust type.

        Type descriptor may be expressed as:

        - Python type hints from std typing module
        - plaintext Rust type strings for setting specific bit depth
        - Python type class - one of {int, str, float, bool}
        - tuple of type information - for example: (float, float)

        :param type_name: type specifier
        :param generics: For internal use. List of type names to consider generic when parsing.
        :type: List[str]
        :return: Normalized type. If the type has subtypes, returns a RuntimeType, else a str.
        :rtype: Union["RuntimeType", str]
        :raises UnknownTypeError: if `type_name` fails to parse

        :examples:

        >>> from opendp.typing import RuntimeType, L1Distance
        >>> assert RuntimeType.parse(int) == "i32"
        >>> assert RuntimeType.parse("i32") == "i32"
        >>> assert RuntimeType.parse(L1Distance[int]) == "L1Distance<i32>"
        >>> assert RuntimeType.parse(L1Distance["f32"]) == "L1Distance<f32>"
        """
        generics = generics or []
        if isinstance(type_name, RuntimeType):
            return type_name

        # parse type hints from the typing module
        hinted_type = None
        if sys.version_info >= (3, 8):
            from typing import _GenericAlias
            if isinstance(type_name, _GenericAlias):
                hinted_type = typing.get_origin(type_name), typing.get_args(type_name)
        if sys.version_info >= (3, 9):
            from types import GenericAlias
            if isinstance(type_name, GenericAlias):
                hinted_type = type_name.__origin__, type_name.__args__
    
        if hinted_type:
            origin, args = hinted_type
            args = [RuntimeType.parse(v, generics=generics) for v in args] or None
            if origin == tuple:
                origin = 'Tuple'
            elif origin == list:
                origin = 'Vec'
            elif origin == dict:
                origin = 'HashMap'
            
            return RuntimeType(RuntimeType.parse(origin, generics=generics), args)

        # parse a tuple of types-- (int, "f64"); (List[int], (int, bool))
        if isinstance(type_name, tuple):
            return RuntimeType('Tuple', list(cls.parse(v, generics=generics) for v in type_name))

        # parse a string-- "Vec<f32>",
        if isinstance(type_name, str):

            if "AllDomain" in type_name:
                import warnings
                warnings.warn("AllDomain is deprecated. Use AtomDomain instead.", DeprecationWarning)
                type_name = type_name.replace("AllDomain", "AtomDomain")

            type_name = type_name.strip()
            if type_name in generics:
                return GenericType(type_name)
            if type_name.startswith('(') and type_name.endswith(')'):
                return RuntimeType('Tuple', cls._parse_args(type_name[1:-1], generics=generics))
            start, end = type_name.find('<'), type_name.rfind('>')

            # attempt to upgrade strings to the metric/measure instance
            origin = type_name[:start] if 0 < start else type_name
            closeness = {
                'ChangeOneDistance': ChangeOneDistance,
                'SymmetricDistance': SymmetricDistance,
                'AbsoluteDistance': AbsoluteDistance,
                'L1Distance': L1Distance,
                'L2Distance': L2Distance,
                'MaxDivergence': MaxDivergence,
                'SmoothedMaxDivergence': SmoothedMaxDivergence
            }.get(origin)
            if closeness is not None:
                if isinstance(closeness, (SensitivityMetric, PrivacyMeasure)):
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
            raise UnknownTypeException(f"non-parameterized argument")

        raise UnknownTypeException(f"unable to parse type: {type_name}")

    @classmethod
    def _parse_args(cls, args, generics=None):
        import re
        return [cls.parse(v, generics=generics) for v in re.split(r",\s*(?![^()<>]*\))", args)]

    @classmethod
    def infer(cls, public_example: Any) -> Union["RuntimeType", str]:
        """Infer the normalized type from a public example.

        :param public_example: data used to infer the type
        :return: Normalized type. If the type has subtypes, returns a RuntimeType, else a str.
        :rtype: Union["RuntimeType", str]
        :raises UnknownTypeException: if inference fails on `public_example`

        :examples:

        >>> from opendp.typing import RuntimeType, L1Distance
        >>> assert RuntimeType.infer(23) == "i32"
        >>> assert RuntimeType.infer(12.) == "f64"
        >>> assert RuntimeType.infer(["A", "B"]) == "Vec<String>"
        >>> assert RuntimeType.infer((12., True, "A")) == "(f64,  bool,String)" # eq doesn't care about whitespace
        """
        if type(public_example) in ELEMENTARY_TYPES:
            return ELEMENTARY_TYPES[type(public_example)]
        
        if isinstance(public_example, (Domain, Metric, Measure)):
            return RuntimeType.parse(public_example.type)

        if isinstance(public_example, tuple):
            return RuntimeType('Tuple', list(map(cls.infer, public_example)))

        if isinstance(public_example, list):
            if public_example:
                inner_type = cls.infer(public_example[0])
                for inner in public_example[1:]:
                    other_type = cls.infer(inner)
                    if other_type != inner_type:
                        raise TypeError(f"vectors must be homogeneously typed, found {inner_type} and {other_type}")
            else:
                inner_type = UnknownType("cannot infer atomic type of empty list")

            return RuntimeType('Vec', [inner_type])

        if np is not None and isinstance(public_example, np.ndarray):
            if public_example.ndim == 0:
                return cls.infer(public_example.item())

            if public_example.ndim == 1:
                inner_type = ELEMENTARY_TYPES.get(public_example.dtype.type)
                if inner_type is None:
                    raise UnknownTypeException(f"Unknown numpy array dtype: {public_example.dtype.type}")
                return RuntimeType('Vec', [inner_type])

            raise UnknownTypeException("arrays with greater than one axis are not yet supported")

        if isinstance(public_example, dict):
            return RuntimeType('HashMap', [
                cls.infer(next(iter(public_example.keys()))),
                cls.infer(next(iter(public_example.values())))
            ])

        if isinstance(public_example, Measurement):
            return "AnyMeasurementPtr"

        if isinstance(public_example, Transformation):
            return "AnyTransformationPtr"

        if public_example is None:
            return RuntimeType('Option', [UnknownType("Constructed Option from a None variant")])
        
        if callable(public_example):
            return "CallbackFn"

        raise UnknownTypeException(type(public_example))

    @classmethod
    def parse_or_infer(
            cls,
            type_name: RuntimeTypeDescriptor = None,
            public_example: Any = None,
            generics: List[str] = None
    ) -> Union["RuntimeType", str]:
        """If type_name is supplied, normalize it. Otherwise, infer the normalized type from a public example.

        :param type_name: type specifier. See RuntimeType.parse for documentation on valid inputs
        :param public_example: data used to infer the type
        :return: Normalized type. If the type has subtypes, returns a RuntimeType, else a str.
        :rtype: Union["RuntimeType", str]
        :param generics: For internal use. List of type names to consider generic when parsing.
        :type: List[str]
        :raises ValueError: if `type_name` fails to parse
        :raises UnknownTypeException: if inference fails on `public_example` or no args are supplied
        """
        if type_name is not None:
            return cls.parse(type_name, generics)
        if public_example is not None:
            return cls.infer(public_example)
        raise UnknownTypeException("either type_name or public_example must be passed")

    @classmethod
    def assert_is_similar(cls, expected, inferred):
        """Assert that `inferred` is a member of the same equivalence class as `parsed`.

        :param expected: the type that the data will be converted to
        :param inferred: the type inferred from data
        :raises TypeError: if `expected` type differs significantly from `inferred` type
        """

        ERROR_URL_298 = "https://github.com/opendp/opendp/discussions/298"
        if isinstance(inferred, UnknownType):
            return
        
        # allow extra flexibility around options, as the inferred type of an Option::<T>::Some will just be T
        def is_option(type_name):
            return isinstance(type_name, RuntimeType) and type_name.origin == "Option"
        if is_option(expected):
            expected = expected.args[0]
            if is_option(inferred):
                if isinstance(inferred.args[0], UnknownType):
                    return
                else:
                    inferred = inferred.args[0]

        if isinstance(expected, str) and isinstance(inferred, str):
            if inferred in ATOM_EQUIVALENCE_CLASSES:
                if expected not in ATOM_EQUIVALENCE_CLASSES[inferred]:
                    raise TypeError(f"inferred type is {inferred}, expected {expected}. See {ERROR_URL_298}")
            else:
                if expected != inferred:
                    raise TypeError(f"inferred type is {inferred}, expected {expected}. See {ERROR_URL_298}")

        elif isinstance(expected, RuntimeType) and isinstance(inferred, RuntimeType):
            if expected.origin != inferred.origin:
                raise TypeError(f"inferred type is {inferred.origin}, expected {expected.origin}. See {ERROR_URL_298}")

            if len(expected.args) != len(inferred.args):
                raise TypeError(f"inferred type has {len(inferred.args)} arg(s), expected {len(expected.args)} arg(s). See {ERROR_URL_298}")

            for (arg_par, arg_inf) in zip(expected.args, inferred.args):
                RuntimeType.assert_is_similar(arg_par, arg_inf)
        else:
            # inferred type differs in structure
            raise TypeError(f"inferred type is {inferred}, expected {expected}. See {ERROR_URL_298}")

    def substitute(self, **kwargs):
        if isinstance(self, GenericType):
            return kwargs.get(self.origin, self)
        if isinstance(self, RuntimeType):
            return RuntimeType(self.origin, self.args and [RuntimeType.substitute(arg, **kwargs) for arg in self.args])
        return self
    

class GenericType(RuntimeType):
    def __str__(self):
        raise UnknownTypeException(f"attempted to create a type_name with an unknown generic: {self.origin}")


class UnknownType(RuntimeType):
    """Indicator for a type that cannot be inferred. Typically the atomic type of an empty list.
    RuntimeTypes containing UnknownType cannot be used in FFI, but still pass RuntimeType.assert_is_similar
    """
    def __init__(self, reason):
        self.origin = None
        self.args = []
        self.reason = reason

    def __str__(self):
        raise UnknownTypeException(f"attempted to create a type_name with an unknown type: {self.reason}")


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


AbsoluteDistance = SensitivityMetric('AbsoluteDistance')
L1Distance = SensitivityMetric('L1Distance')
L2Distance = SensitivityMetric('L2Distance')


class PrivacyMeasure(RuntimeType):
    """All measure RuntimeTypes inherit from PrivacyMeasure.
    Provides static type checking in user-code for privacy measures and a getitem interface like stdlib typing.
    """
    def __getitem__(self, associated_type):
        return PrivacyMeasure(self.origin, [self.parse(type_name=associated_type)])


MaxDivergence = PrivacyMeasure('MaxDivergence')
SmoothedMaxDivergence = PrivacyMeasure('SmoothedMaxDivergence')
FixedSmoothedMaxDivergence = PrivacyMeasure('FixedSmoothedMaxDivergence')
ZeroConcentratedDivergence = PrivacyMeasure('ZeroConcentratedDivergence')

class Carrier(RuntimeType):
    def __getitem__(self, subdomains):
        if not isinstance(subdomains, tuple):
            subdomains = (subdomains,)
        return Carrier(self.origin, [self.parse(type_name=subdomain) for subdomain in subdomains])


Vec = Carrier('Vec')
HashMap = Carrier('HashMap')
i8 = 'i8'
i16 = 'i16'
i32 = 'i32'
i64 = 'i64'
i128 = 'i128'
isize = 'isize'
u8 = 'u8'
u16 = 'u16'
u32 = 'u32'
u64 = 'u64'
u128 = 'u128'
usize = 'usize'
f32 = 'f32'
f64 = 'f64'
String = 'String'
AnyMeasurementPtr = "AnyMeasurementPtr"
AnyTransformationPtr = "AnyTransformationPtr"


class DomainDescriptor(RuntimeType):
    def __getitem__(self, subdomain):
        if not isinstance(subdomain, tuple):
            subdomain = (subdomain,)
        return DomainDescriptor(self.origin, [self.parse(type_name=sub_i) for sub_i in subdomain])    


AtomDomain = DomainDescriptor('AtomDomain')
VectorDomain = DomainDescriptor('VectorDomain')
MapDomain = DomainDescriptor('MapDomain')
OptionDomain = DomainDescriptor('OptionDomain')


def get_atom(type_name):
    type_name = RuntimeType.parse(type_name)
    while isinstance(type_name, RuntimeType):
        if isinstance(type_name, (UnknownType, GenericType)):
            return
        type_name = type_name.args[0]
    return type_name


def get_atom_or_infer(type_name: RuntimeType, example):
    return get_atom(type_name) or RuntimeType.infer(example)


def get_first(value):
    if value is None or not len(value):
        return None
    return next(iter(value))

def parse_or_infer(type_name: RuntimeType, example):
    return RuntimeType.parse_or_infer(type_name, example)

def pass_through(value):
    return value

def get_dependencies(value):
    return getattr(value, "_dependencies", None)

def get_dependencies_iterable(value):
    return list(map(get_dependencies, value))

def get_carrier_type(value):
    return value.carrier_type

def get_distance_type(value):
    return value.distance_type

def get_type(value):
    return value.type