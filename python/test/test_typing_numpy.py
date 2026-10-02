import pytest
from opendp.typing import RuntimeType


np = pytest.importorskip("numpy")

def test_numpy_function():
    assert str(RuntimeType.infer(np.array([1, 2, 3]))) == "Vec<i64>"
    assert str(RuntimeType.infer(np.array(1))) == "i32"
    assert str(RuntimeType.infer(np.array(1.0))) == "f64"
    assert str(RuntimeType.infer(np.array("A"))) == "String"
    assert str(RuntimeType.infer(np.array(["A", "B"]))) == "Vec<String>"
