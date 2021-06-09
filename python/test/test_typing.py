import sys
from typing import List, Tuple, Any

from opendp.v1.mod import UnknownTypeException
from opendp.v1.typing import RuntimeType, L1Sensitivity, SensitivityMetric, L2Sensitivity, DatasetMetric


def test_typing_hint():
    # python < 3.8 should raise a NotImplementedError
    if sys.version_info < (3, 8):
        try:
            assert str(RuntimeType.parse(Tuple[int, float])) == "(i32, f64)"
            raise Exception("typing hints should fail with error below python 3.8")
        except NotImplementedError:
            # on python < 3.8 the remaining tests do not apply
            return

    assert str(RuntimeType.parse(Tuple[int, float])) == "(i32, f64)"
    assert str(RuntimeType.parse(Tuple[int, Tuple[str]])) == "(i32, (String))"
    assert str(RuntimeType.parse(List[int])) == "Vec<i32>"
    assert str(RuntimeType.parse(List[List[str]])) == "Vec<Vec<String>>"
    assert str(RuntimeType.parse((List[int], (int, bool)))) == '(Vec<i32>, (i32, bool))'
    assert isinstance(RuntimeType.parse('HammingDistance'), DatasetMetric)
    assert isinstance(RuntimeType.parse('L1Sensitivity<f64>'), SensitivityMetric)

    try:
        RuntimeType.parse(List[Any])
        raise Exception("typing.Any should fail to parse")
    except UnknownTypeException:
        pass


def test_sensitivity():
    assert isinstance(L1Sensitivity[int], SensitivityMetric)
    assert not isinstance(RuntimeType.parse('(f32)'), SensitivityMetric)
    assert str(L1Sensitivity[int]) == "L1Sensitivity<i32>"


def test_tuples():
    assert str(RuntimeType.parse((float, int))) == "(f64, i32)"
    assert str(RuntimeType.parse(('f32', (int, 'L1Sensitivity<i32>')))) == '(f32, (i32, L1Sensitivity<i32>))'
    assert str(RuntimeType.parse(('f32', (int, L2Sensitivity[float])))) == '(f32, (i32, L2Sensitivity<f64>))'


def test_c():
    """test that c_type strings are not mangled"""
    rt_type = RuntimeType.parse('FfiResult<void *>')
    assert str(rt_type) == 'FfiResult<void *>'
    assert rt_type.args[0] == 'void *'
