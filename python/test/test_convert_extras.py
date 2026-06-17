import pytest


import polars as pl
from polars.testing import assert_frame_equal


import opendp.prelude as dp
from opendp.extras.polars import Bound, Margin

from opendp._convert import py_to_c, c_to_py
from opendp._lib import AnyObjectPtr
from opendp.typing import RuntimeType

@pytest.mark.parametrize("val_in, type_name", [
    (Margin(by=[]), "Margin"),
    (Bound(by=[]), "Bound"),
    ([Bound(by=[])], "Bounds"),


])
def test_extras_object(val_in, type_name):
    obj = py_to_c(val_in, c_type=AnyObjectPtr, type_name=type_name)
    val_out = c_to_py(obj)
    assert val_out == val_in


@pytest.mark.parametrize("val_in, type_name", [
    (pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String}), "LazyFrame")
])
def test_extras_object_polars(val_in, type_name):
    obj = py_to_c(val_in, c_type=AnyObjectPtr, type_name=type_name)
    val_out = c_to_py(obj)
    assert_frame_equal(val_out, val_in)

