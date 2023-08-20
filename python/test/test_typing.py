import pytest
import sys
from typing import List, Tuple, Any

from opendp.mod import *
from opendp.typing import *


try:
    import numpy as np
except ImportError:
    pass

@pytest.mark.skipif('numpy' not in sys.modules,
                    reason="requires the Numpy library")
def test_numpy_function():
    print(RuntimeType.infer(np.array([1, 2, 3])))
    print(RuntimeType.infer(np.array(1)))
    print(RuntimeType.infer(np.array(1.)))
    print(RuntimeType.infer(np.array("A")))
    print(RuntimeType.infer(np.array(["A", "B"])))


def test_typing_hint():
    # Python < 3.8 should raise an exception
    if sys.version_info < (3, 8):
        try:
            assert str(RuntimeType.parse(Tuple[int, float])) == "(i32, f64)"
            raise Exception("typing hints should fail with error below Python 3.8")
        except:
            # on Python < 3.8 the remaining tests do not apply
            return

    assert str(RuntimeType.parse(Tuple[int, float])) == "(i32, f64)"
    assert str(RuntimeType.parse(Tuple[int, Tuple[str]])) == "(i32, (String))"
    assert str(RuntimeType.parse(List[int])) == "Vec<i32>"
    assert str(RuntimeType.parse(List[List[str]])) == "Vec<Vec<String>>"
    assert str(RuntimeType.parse((List[int], (int, bool)))) == '(Vec<i32>, (i32, bool))'
    assert isinstance(RuntimeType.parse('L1Distance<f64>'), SensitivityMetric)

    try:
        RuntimeType.parse(List[Any])
        raise Exception("typing.Any should fail to parse")
    except UnknownTypeException:
        pass


def test_sensitivity():
    assert isinstance(L1Distance[int], SensitivityMetric)
    assert not isinstance(RuntimeType.parse('(f32)'), SensitivityMetric)
    assert str(L1Distance[int]) == "L1Distance<i32>"


def test_tuples():
    assert str(RuntimeType.parse((float, int))) == "(f64, i32)"
    assert str(RuntimeType.parse(('f32', (int, 'L1Distance<i32>')))) == '(f32, (i32, L1Distance<i32>))'
    assert str(RuntimeType.parse(('f32', (int, L2Distance[float])))) == '(f32, (i32, L2Distance<f64>))'


def test_c():
    """test that c_type strings are not mangled"""
    rt_type = RuntimeType.parse('FfiResult<void *>')
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


def test_default_type():
    assert RuntimeType.parse(int) == i32
    set_default_int_type(u64)
    assert RuntimeType.parse(int) == u64
    set_default_int_type(i32)

# for a more thorough manual test of the set_default_int_type and set_default_float_type functions:
# 1. recompile with --release mode
# 2. set OPENDP_TEST_RELEASE
# 3. uncomment these commands
#    set_default_int_type(i64)
#    set_default_float_type(f32)
# 4. run pytest, and sanity check the failures
