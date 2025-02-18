import re
from typing import List, Tuple, Any

import pytest

from opendp.mod import *
from opendp.typing import *
from opendp.typing import _INTEGER_TYPES


def test_numpy_function():
    np = pytest.importorskip('numpy')
    assert str(RuntimeType.infer(np.array([1, 2, 3]))) == 'Vec<i64>'
    assert str(RuntimeType.infer(np.array(1))) == 'i32'
    assert str(RuntimeType.infer(np.array(1.))) == 'f64'
    assert str(RuntimeType.infer(np.array("A"))) == 'String'
    assert str(RuntimeType.infer(np.array(["A", "B"]))) == 'Vec<String>'


def test_typing_infer_to_string():
    # Currently these return actual strings, which we test with `is`.
    # The plan is to change the response type to be consistent:
    # https://github.com/opendp/opendp/issues/1665
    assert RuntimeType.infer(23) is 'i32' # noqa: F632
    assert RuntimeType.infer(12.) is 'f64' # noqa: F632
    assert RuntimeType.infer('hello') is 'String' # noqa: F632
    assert RuntimeType.infer(lambda: True) is 'CallbackFn' # noqa: F632    
    assert RuntimeType.infer(object(), py_object=True) is 'ExtrinsicObject' # noqa: F632
    

def test_typing_infer_to_object():
    # With py_object=True, it can fall back to a more general type:
    assert RuntimeType.infer([1, True], py_object=True) == 'Vec<ExtrinsicObject>'
    # Without py_object=True, it fails:
    with pytest.raises(TypeError, match=re.escape("elements must be homogeneously typed")):
        RuntimeType.infer([1, True])

    infer_vec = RuntimeType.infer(["A", "B"])
    assert infer_vec == 'Vec<String>'
    assert isinstance(infer_vec, RuntimeType)

    infer_tuple = RuntimeType.infer((12., True, "A"))
    assert infer_tuple == '(f64, bool, String)'
    assert isinstance(infer_tuple, RuntimeType)

    with pytest.raises(UnknownTypeException, match=re.escape("<class 'object'>")):
        RuntimeType.infer(object())
    
    with pytest.raises(UnknownTypeException, match=re.escape("Type of Option cannot be inferred from None")):
        RuntimeType.infer(None)
    with pytest.raises(UnknownTypeException, match=re.escape('Cannot infer atomic type when empty')):
        RuntimeType.infer([])

def test_typing_parse():
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
    with pytest.raises(UnknownTypeException):
        RuntimeType.parse(list[Any])


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
    with pytest.raises(OpenDPException, match=re.escape('enable_features("A", "Z")')):
        assert_features("A", "Z")


def test_set_feature():
    enable_features("A")
    assert_features("A")

    disable_features("A")
    assert "A" not in GLOBAL_FEATURES


def test_default_float_type():
    assert RuntimeType.parse(float) == f64

    set_default_float_type(f64)
    assert RuntimeType.parse(float) == f64

    # Can't set to f32 because debug binary has fewer types.

def test_runtime_type_hash():
    assert {Vec[int]} == {RuntimeType.parse("Vec<int>")}


disallowed_int_default_types = set([i128, u128, isize])

@pytest.mark.parametrize('integer_type', set(_INTEGER_TYPES) - disallowed_int_default_types)
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
