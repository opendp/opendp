import pytest

from opendp.extras.polars import Bound, Margin

from opendp._convert import py_to_c, c_to_py
from opendp._lib import AnyObjectPtr

@pytest.mark.parametrize("val_in, type_name", [
    (Margin(by=[]), "Margin"),
    (Bound(by=[]), "Bound"),
    ([Bound(by=[])], "Bounds"),
])
def test_extras_object(val_in, type_name):
    obj = py_to_c(val_in, c_type=AnyObjectPtr, type_name=type_name)
    val_out = c_to_py(obj)
    assert val_out == val_in


def test_extras_object_polars():
    pl = pytest.importorskip("polars")
    from polars.testing import assert_frame_equal  # type: ignore

    val_in = pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String})
    type_name = "LazyFrame"
    obj = py_to_c(val_in, c_type=AnyObjectPtr, type_name=type_name)
    val_out = c_to_py(obj)
    assert_frame_equal(val_out, val_in)
