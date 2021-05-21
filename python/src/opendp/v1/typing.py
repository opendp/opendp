from typing import Union, Any, _GenericAlias
import sys
from collections import Hashable

import typing
from opendp.v1.mod import UnknownTypeException

ELEMENTARY_TYPES = {int: 'i32', float: 'f64', str: 'String', bool: 'bool'}
RuntimeTypeDescriptor = Union["RuntimeType", "typing._GenericAlias", tuple, list, int, float, str, bool]


class RuntimeType(object):
    def __init__(self, origin, args=None):
        if not isinstance(origin, str):
            raise ValueError("origin must be a string", origin)
        self.origin = origin
        self.args = args

    def __eq__(self, other):
        return self.origin == other.origin and self.args == other.args

    def __str__(self):
        result = self.origin or ''
        if self.args:
            result += f'<{",".join(map(str, self.args))}>'
        return result

    @classmethod
    def parse(cls, type_name: RuntimeTypeDescriptor) -> Union["RuntimeType", str]:
        if isinstance(type_name, RuntimeType):
            return type_name

        # parse type hints from the typing module
        if isinstance(type_name, _GenericAlias):
            if sys.version_info < (3, 8):
                raise NotImplementedError("typing hints are only supported in python 3.8 and above")

            origin = typing.get_origin(type_name)
            args = list(map(RuntimeType.parse, typing.get_args(type_name))) or None
            if origin == tuple:
                return Tuple(*args)
            if origin == list:
                origin = 'Vec'
            return RuntimeType(RuntimeType.parse(origin), args)

        # parse a tuple of types-- (int, "f64"); (List[int], (int, bool))
        if isinstance(type_name, tuple):
            return Tuple(*(cls.parse(v) for v in type_name))

        # parse a string-- "Vec<f32>",
        if isinstance(type_name, str):
            type_name = type_name.strip()
            if type_name.startswith('(') and type_name.endswith(')'):
                return Tuple(cls._parse_args(type_name[1:-1]))
            start, end = type_name.find('<'), type_name.rfind('>')
            if 0 < start < end < len(type_name):
                return RuntimeType(type_name[:start], cls._parse_args(type_name[start + 1: end]))
            if start == end < 0:
                return type_name

        if isinstance(type_name, Hashable) and type_name in ELEMENTARY_TYPES:
            return ELEMENTARY_TYPES[type_name]

        if type_name == tuple:
            raise ValueError(f"non-parameterized argument")

        raise ValueError(f"unable to parse type: {type_name}")

    @classmethod
    def _parse_args(cls, args):
        import re
        return [cls.parse(v) for v in re.split(",\\s*(?![^()<>]*\\))", args)]

    @classmethod
    def infer(cls, public_example: Any) -> Union["RuntimeType", str]:
        if type(public_example) in ELEMENTARY_TYPES:
            return ELEMENTARY_TYPES[type(public_example)]

        elif isinstance(public_example, tuple):
            return Tuple(map(cls.infer, public_example))

        elif isinstance(public_example, list):
            if not public_example:
                raise UnknownTypeException("attempted to infer inner type of empty list. Please fill the type argument.")
            return RuntimeType('Vec', [cls.infer(public_example[0])])

        raise UnknownTypeException(public_example)

    @classmethod
    def parse_or_infer(
            cls,
            type_name: RuntimeTypeDescriptor = None,
            public_example: Any = None
    ) -> Union["RuntimeType", str]:
        if type_name is not None:
            return cls.parse(type_name)
        if public_example is not None:
            return cls.infer(public_example)
        raise UnknownTypeException("either type_name or public_example must be passed")


class Tuple(RuntimeType):
    def __init__(self, *args):
        super().__init__('Tuple', list(args))

    def __str__(self):
        return f'({",".join(map(str, self.args))})'


class SensitivityMetric(RuntimeType):
    def __getitem__(self, associated_type):
        return SensitivityMetric(self.origin, [self.parse(type_name=associated_type)])


class DatasetMetric(RuntimeType):
    pass


class PrivacyMeasure(RuntimeType):
    def __getitem__(self, associated_type):
        return PrivacyMeasure(self.origin, [self.parse(type_name=associated_type)])


L1Sensitivity = SensitivityMetric('L1Sensitivity')
L2Sensitivity = SensitivityMetric('L2Sensitivity')

HammingDistance = DatasetMetric('HammingDistance')
SymmetricDistance = DatasetMetric('SymmetricDistance')

MaxDivergence = PrivacyMeasure('MaxDivergence')
SmoothedMaxDivergence = PrivacyMeasure('SmoothedMaxDivergence')
