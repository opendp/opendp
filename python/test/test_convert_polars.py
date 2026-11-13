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


@pytest.mark.parametrize("type_name", ["Vec<Expr>", "Vec<Bound>", "Bounds"])
@pytest.mark.parametrize("empty", [False, True])
def test_owned_polars_vector_views(type_name, empty):
    from opendp.extras.polars import Bound

    if type_name == "Vec<Expr>":
        values = [] if empty else [pl.col("a"), pl.col("b") + 1]
    else:
        values = (
            [] if empty else [Bound(by=["a"], per_group=2), Bound(by=[], num_groups=3)]
        )
    obj = py_to_c(values, AnyObjectPtr, type_name)
    for _ in range(10):
        out = c_to_py(obj)
        if type_name == "Vec<Expr>":
            assert list(map(str, out)) == list(map(str, values))
        else:
            assert out == values


def test_owned_polars_nested_views_release_without_consuming_source(monkeypatch):
    import gc
    from opendp import _convert
    from opendp.extras.polars import Bound, Margin

    for value, type_name in [
        (Bound(by=["a"], per_group=2, num_groups=3), "Bound"),
        (Margin(by=["a"], max_length=4, max_groups=2, invariant="keys"), "Margin"),
    ]:
        obj = py_to_c(value, AnyObjectPtr, type_name)
        convert = _convert._slice_to_py

        def fail_nested(raw, kind):
            if str(kind) == "Vec<Expr>":
                raise ValueError("nested conversion failed")
            return convert(raw, kind)

        with monkeypatch.context() as patch:
            patch.setattr(_convert, "_slice_to_py", fail_nested)
            with pytest.raises(ValueError, match="nested conversion failed"):
                c_to_py(obj)
        assert c_to_py(obj) == value
        del obj
        gc.collect()


@pytest.mark.parametrize("name", ["", "λ", "long name" * 100])
def test_owned_arrow_output_outlives_source(name):
    import gc

    series = pl.concat(
        [pl.Series(name, [1, None]), pl.Series(name, [3, 4])], rechunk=False
    )
    obj = py_to_c(series, AnyObjectPtr, "Series")
    out = c_to_py(obj)
    del obj
    gc.collect()
    assert out.equals(series)

    frame = pl.DataFrame({"numbers": series, "text": ["a", None, "b", "λ"]})
    obj = py_to_c(frame, AnyObjectPtr, "DataFrame")
    out = c_to_py(obj)
    del obj
    gc.collect()
    assert out.equals(frame)


def test_extras_object_polars():
    from polars.testing import assert_frame_equal  # type: ignore
    from opendp._convert import py_to_c, c_to_py
    from opendp._lib import AnyObjectPtr

    val_in = pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String})
    type_name = "LazyFrame"
    obj = py_to_c(val_in, c_type=AnyObjectPtr, type_name=type_name)
    val_out = c_to_py(obj)
    assert_frame_equal(val_out, val_in)
