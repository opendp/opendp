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


def test_arrow_input_can_be_abandoned_before_import():
    import gc
    from opendp._convert import _series_to_slice
    from opendp._lib import FfiSlicePtr

    before = set(FfiSlicePtr._dependencies)
    series = pl.Series("abandoned λ", [1, None, 3])
    for _ in range(20):
        raw = _series_to_slice(series)
        del raw
        gc.collect()
    assert set(FfiSlicePtr._dependencies) == before
    assert series.to_list() == [1, None, 3]


@pytest.mark.parametrize("stage", ["to_arrow", "export"])
def test_arrow_input_failure_releases_all_buffers(monkeypatch, stage):
    import gc
    from opendp._convert import _series_to_slice
    from opendp._lib import FfiSlicePtr

    before = set(FfiSlicePtr._dependencies)
    series = pl.Series("failed export", [1, None, 3])

    class FailedExport:
        def _export_to_c(self, *args):
            raise ValueError("export failed")

    def to_arrow(self):
        if stage == "to_arrow":
            raise ValueError("to_arrow failed")
        return FailedExport()

    with monkeypatch.context() as patch:
        patch.setattr(pl.Series, "to_arrow", to_arrow)
        for _ in range(10):
            with pytest.raises(ValueError, match="failed"):
                _series_to_slice(series)
    gc.collect()
    assert set(FfiSlicePtr._dependencies) == before
    assert series.to_list() == [1, None, 3]


def test_partial_dataframe_input_releases_unconsumed_columns(monkeypatch):
    import gc
    from opendp import _convert
    from opendp._lib import FfiSlicePtr

    before = set(FfiSlicePtr._dependencies)
    frame = pl.DataFrame({"a": [1, 2], "b": [3, 4]})
    convert = _convert._series_to_slice

    def fail_second_column(series):
        if series.name == "b":
            raise ValueError("second column failed")
        return convert(series)

    with monkeypatch.context() as patch:
        patch.setattr(_convert, "_series_to_slice", fail_second_column)
        for _ in range(10):
            with pytest.raises(ValueError, match="second column failed"):
                _convert._dataframe_to_slice(frame)
    gc.collect()
    assert set(FfiSlicePtr._dependencies) == before


def test_arrow_input_array_is_consumed_at_most_once():
    import gc
    from opendp._convert import _series_to_slice, _wrap_in_slice
    from opendp._data import slice_as_object
    from opendp.mod import OpenDPException

    series = pl.Series("single import", [1, None, 3])
    raw = _series_to_slice(series)
    # Rejecting the header before import must leave the array available.
    with pytest.raises(OpenDPException, match="length 3"):
        slice_as_object(_wrap_in_slice(raw.contents.ptr, 2), "Series")
    obj = slice_as_object(raw, "Series")
    with pytest.raises(OpenDPException, match="already been consumed"):
        slice_as_object(raw, "Series")
    assert c_to_py(obj).equals(series)
    del raw, obj
    gc.collect()


def test_arrow_input_failure_after_array_transfer_is_not_freed_twice():
    import ctypes
    import gc
    from opendp._convert import _series_to_slice
    from opendp._data import slice_as_object
    from opendp.mod import OpenDPException

    # Standard Arrow C Data Interface prefix; schema payload ownership remains
    # with its real release callback while the format pointer is temporarily bad.
    class SchemaPrefix(ctypes.Structure):
        _fields_ = [("format", ctypes.c_void_p)]

    series = pl.Series("failed import", [1, None, 3])
    raw = _series_to_slice(series)
    parts = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    schema = ctypes.cast(parts[1], ctypes.POINTER(SchemaPrefix))
    original_format = schema.contents.format
    bad_format = ctypes.create_string_buffer(b"invalid arrow format")
    schema.contents.format = ctypes.addressof(bad_format)
    try:
        with pytest.raises(OpenDPException, match="failed to import field"):
            slice_as_object(raw, "Series")
        assert parts[0] is None
    finally:
        schema.contents.format = original_format
    with pytest.raises(OpenDPException, match="already been consumed"):
        slice_as_object(raw, "Series")
    del parts, schema, raw
    gc.collect()
    assert series.to_list() == [1, None, 3]


def test_extras_object_polars():
    from polars.testing import assert_frame_equal  # type: ignore
    from opendp._convert import py_to_c, c_to_py
    from opendp._lib import AnyObjectPtr

    val_in = pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String})
    type_name = "LazyFrame"
    obj = py_to_c(val_in, c_type=AnyObjectPtr, type_name=type_name)
    val_out = c_to_py(obj)
    assert_frame_equal(val_out, val_in)
