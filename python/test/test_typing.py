import pytest
from typing import List, Tuple, Any

from opendp.mod import *
from opendp.typing import *


def test_numpy_function():
    np = pytest.importorskip('numpy')
    print(RuntimeType.infer(np.array([1, 2, 3])))
    print(RuntimeType.infer(np.array(1)))
    print(RuntimeType.infer(np.array(1.)))
    print(RuntimeType.infer(np.array("A")))
    print(RuntimeType.infer(np.array(["A", "B"])))


def test_typing_hint():
    assert str(RuntimeType.parse(Tuple[int, float])) == "(i32, f64)" # type: ignore[arg-type]
    assert str(RuntimeType.parse(tuple[int, float])) == "(i32, f64)" # type: ignore[arg-type]
    assert str(RuntimeType.parse(Tuple[int, Tuple[str]])) == "(i32, (String))" # type: ignore[arg-type]
    assert str(RuntimeType.parse(tuple[int, tuple[str]])) == "(i32, (String))" # type: ignore[arg-type]
    assert str(RuntimeType.parse(List[int])) == "Vec<i32>"
    assert str(RuntimeType.parse(list[int])) == "Vec<i32>"
    assert str(RuntimeType.parse(List[List[str]])) == "Vec<Vec<String>>"
    assert str(RuntimeType.parse(list[list[str]])) == "Vec<Vec<String>>"
    assert str(RuntimeType.parse((List[int], (int, bool)))) == '(Vec<i32>, (i32, bool))'
    assert str(RuntimeType.parse((list[int], (int, bool)))) == '(Vec<i32>, (i32, bool))'
    assert isinstance(RuntimeType.parse('L1Distance<f64>'), SensitivityMetric)

    try:
        RuntimeType.parse(list[Any])
        raise Exception("typing.Any should fail to parse")
    except UnknownTypeException:
        pass


def test_sensitivity():
    assert isinstance(L1Distance[int], SensitivityMetric)
    assert not isinstance(RuntimeType.parse('(f32)'), SensitivityMetric)
    assert str(L1Distance[int]) == "L1Distance<i32>"
    assert L1Distance[int] != {}


def test_tuples():
    assert str(RuntimeType.parse((float, int))) == "(f64, i32)"
    assert str(RuntimeType.parse(('f32', (int, 'L1Distance<i32>')))) == '(f32, (i32, L1Distance<i32>))'
    assert str(RuntimeType.parse(('f32', (int, L2Distance[float])))) == '(f32, (i32, L2Distance<f64>))'


def test_c():
    """test that c_type strings are not mangled"""
    rt_type = RuntimeType.parse('FfiResult<void *>')
    assert isinstance(rt_type, RuntimeType)
    assert str(rt_type) == 'FfiResult<void *>'
    assert rt_type.args[0] == 'void *'


def test_feature_fails():
    with pytest.raises(AssertionError):
        assert_features("A")


def test_set_feature():
    enable_features("A")
    assert_features("A")

    disable_features("A")
    assert "A" not in GLOBAL_FEATURES


disallowed_int_default_types = set([i128, u128, isize])

@pytest.mark.parametrize('integer_type', set(INTEGER_TYPES) - disallowed_int_default_types)
def test_default_int_type(integer_type):
    assert RuntimeType.parse(int) == i32

    set_default_int_type(integer_type)
    assert RuntimeType.parse(int) == integer_type

    set_default_int_type(i32)


@pytest.mark.parametrize('integer_type', disallowed_int_default_types)
def test_disallowed_default_int_type(integer_type):
    assert RuntimeType.parse(int) == i32

    with pytest.raises(AssertionError):
        set_default_int_type(integer_type)

    set_default_int_type(i32)

# for a more thorough manual test of the set_default_int_type and set_default_float_type functions:
# 1. recompile with --release mode
# 2. set OPENDP_TEST_RELEASE
# 3. uncomment these commands
#    set_default_int_type(i64)
#    set_default_float_type(f32)
# 4. run pytest, and sanity check the failures
