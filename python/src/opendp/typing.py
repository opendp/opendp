import sys
import typing
from collections.abc import Hashable
from typing import Union, Any, Type, List

from opendp.mod import UnknownTypeException
from opendp._lib import ATOM_EQUIVALENCE_CLASSES

if sys.version_info >= (3, 7):
    from typing import _GenericAlias
else:
    from typing import GenericMeta as _GenericAlias

ELEMENTARY_TYPES = {int: 'i32', float: 'f64', str: 'String', bool: 'bool'}

# all ways of providing type information
RuntimeTypeDescriptor = Union[
    "RuntimeType",  # as the normalized type -- SubstituteDistance; RuntimeType.parse("i32")
    _GenericAlias,  # a python type hint from the std typing module -- List[int]
    str,  # plaintext string in terms of rust types -- "Vec<i32>"
    Type[Union[typing.List, typing.Tuple, int, float, str, bool]],  # using the python type class itself -- int, float
    tuple,  # shorthand for tuples -- (float, "f64"); (SubstituteDistance, List[int])
]


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
        """Parse type descriptor into a normalized rust type.

        Type descriptor may be expressed as:

        - python type hints from std typing module
        - plaintext rust type strings for setting specific bit depth
        - python type class - one of {int, str, float, bool}
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
        if isinstance(type_name, _GenericAlias):
            if sys.version_info < (3, 8):
                raise NotImplementedError("parsing type hint annotations are only supported in python 3.8 and above")

            origin = typing.get_origin(type_name)
            args = [RuntimeType.parse(v, generics=generics) for v in typing.get_args(type_name)] or None
            if origin == tuple:
                origin = 'Tuple'
            if origin == list:
                origin = 'Vec'
            return RuntimeType(RuntimeType.parse(origin, generics=generics), args)

        # parse a tuple of types-- (int, "f64"); (List[int], (int, bool))
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

            # attempt to upgrade strings to the metric/measure instance
            origin = type_name[:start] if 0 < start else type_name
            closeness = {
                'SubstituteDistance': SubstituteDistance,
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
                'AllDomain': AllDomain,
                'BoundedDomain': BoundedDomain,
                'VectorDomain': VectorDomain,
                'OptionNullDomain': OptionNullDomain,
                'InherentNullDomain': InherentNullDomain,
                'SizedDomain': SizedDomain
            }.get(origin)
            if domain is not None:
                return domain[cls._parse_args(type_name[start + 1: end], generics=generics)[0]]

            if 0 < start < end < len(type_name):
                return RuntimeType(origin, args=cls._parse_args(type_name[start + 1: end], generics=generics))
            if start == end < 0:
                return type_name

        if isinstance(type_name, Hashable) and type_name in ELEMENTARY_TYPES:
            return ELEMENTARY_TYPES[type_name]

        if type_name == tuple:
            raise UnknownTypeException(f"non-parameterized argument")

        raise UnknownTypeException(f"unable to parse type: {type_name}")

    @classmethod
    def _parse_args(cls, args, generics=None):
        import re
        return [cls.parse(v, generics=generics) for v in re.split(",\\s*(?![^()<>]*\\))", args)]

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

        if isinstance(public_example, tuple):
            return RuntimeType('Tuple', list(map(cls.infer, public_example)))

        if isinstance(public_example, list):
            return RuntimeType('Vec', [
                cls.infer(public_example[0]) if public_example else UnknownType(
                    "cannot infer atomic type of empty list")
            ])

        if isinstance(public_example, dict):
            return RuntimeType('HashMap', [
                cls.infer(next(iter(public_example.keys()))),
                cls.infer(next(iter(public_example.values())))
            ])

        if public_example is None:
            return RuntimeType('Option', [UnknownType("Constructed Option from a None variant")])

        raise UnknownTypeException(public_example)

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
        :raises AssertionError: if `expected` type differs significantly from `inferred` type
        """

        ERROR_URL_298 = "https://github.com/opendp/opendp/discussions/298"
        if isinstance(inferred, UnknownType):
            return
        if isinstance(expected, str) and isinstance(inferred, str):
            if inferred in ATOM_EQUIVALENCE_CLASSES:
                assert expected in ATOM_EQUIVALENCE_CLASSES[inferred], \
                    f"inferred type is {inferred}, expected {expected}. See {ERROR_URL_298}"
            else:
                assert expected == inferred, \
                    f"inferred type is {inferred}, expected {expected}. See {ERROR_URL_298}"

        elif isinstance(expected, RuntimeType) and isinstance(inferred, RuntimeType):
            # allow extra flexibility around options, as the inferred type of an Option::<T>::Some will just be T
            if expected.origin == "Option" and inferred.origin != "Option":
                expected = expected.args[0]

            assert expected.origin == inferred.origin, \
                f"inferred type is {inferred.origin}, expected {expected.origin}. See {ERROR_URL_298}"

            assert len(expected.args) == len(inferred.args), \
                f"inferred type has {len(inferred.args)} arg(s), expected {len(expected.args)} arg(s). See {ERROR_URL_298}"

            for (arg_par, arg_inf) in zip(expected.args, inferred.args):
                RuntimeType.assert_is_similar(arg_par, arg_inf)
        else:
            # inferred type differs in structure
            raise AssertionError(f"inferred type is {inferred}, expected {expected}. See {ERROR_URL_298}")

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


class DatasetMetric(RuntimeType):
    """All dataset metric RuntimeTypes inherit from DatasetMetric.
    Provides static type checking in user-code for dataset metrics.
    """
    pass


SubstituteDistance = DatasetMetric('SubstituteDistance')
SymmetricDistance = DatasetMetric('SymmetricDistance')


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


class Domain(RuntimeType):
    def __getitem__(self, subdomain):
        return Domain(self.origin, [self.parse(type_name=subdomain)])


AllDomain = Domain('AllDomain')
BoundedDomain = Domain('BoundedDomain')
VectorDomain = Domain('VectorDomain')
OptionNullDomain = Domain('OptionNullDomain')
InherentNullDomain = Domain('InherentNullDomain')
SizedDomain = Domain('SizedDomain')


def get_domain_atom(domain):
    while isinstance(domain, RuntimeType):
        if isinstance(domain, (UnknownType, GenericType)):
            return
        domain = domain.args[0]
    return domain


def get_domain_atom_or_infer(domain: RuntimeType, example):
    return get_domain_atom(domain) or RuntimeType.infer(example)


def get_first(value):
    return value[0]
