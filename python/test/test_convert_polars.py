import pytest

from opendp._convert import py_to_c, c_to_py

from .test_convert import AnyObjectPtr

pl = pytest.importorskip("polars")


def test_polars_dataframe():
    val_in = pl.DataFrame(
        {
            "A": [1] * 100,
            "B": ["X"] * 100,
            "C": [True] * 100,
        }
    )
    obj = py_to_c(val_in, AnyObjectPtr, "DataFrame")
    val_out = c_to_py(obj)
    assert val_out.equals(val_in)


def test_polars_expr():
    val_in = pl.all()
    obj = py_to_c(val_in, AnyObjectPtr, "Expr")
    val_out = c_to_py(obj)
    assert str(val_out) == str(val_in)


def test_extras_object_polars():
    from polars.testing import assert_frame_equal  # type: ignore
    from opendp._convert import py_to_c, c_to_py
    from opendp._lib import AnyObjectPtr

    val_in = pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String})
    type_name = "LazyFrame"
    obj = py_to_c(val_in, c_type=AnyObjectPtr, type_name=type_name)
    val_out = c_to_py(obj)
    assert_frame_equal(val_out, val_in)
