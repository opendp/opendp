import pytest

from opendp.typing import *
from opendp._convert import (
    py_to_c,
    c_to_py,
    _check_and_cast_scalar,
    _hashmap_to_slice,
    _numpy_dtype_for_rust_type,
    _numpy_to_slice,
    _py_to_slice,
    _scalar_to_slice,
    _slice_to_hashmap,
    _slice_to_numpy,
    _slice_to_scalar,
    _slice_to_vector,
    _vector_to_slice,
)
from opendp._lib import AnyObjectPtr, ctypes, FfiSlice, FfiSlicePtr
from opendp.extras.polars import Margin, Bound


@pytest.mark.parametrize(
    "val_type,val_in",
    [
        (int, 123),
        (float, 123.123),
        (str, "hello, world"),
        (list, [1, 2, 3]),
        (tuple, (1.0, 1e-7)),
    ],
)
def test_data_object(val_type, val_in):
    assert isinstance(val_in, val_type)
    obj = py_to_c(val_in, c_type=AnyObjectPtr)
    val_out = c_to_py(obj)
    assert val_out == val_in


def test_data_object_int_to_float():
    val_in = 123
    obj = py_to_c(val_in, c_type=AnyObjectPtr, type_name="f64")
    val_out = c_to_py(obj)
    assert val_out == val_in


def test_data_object_string_pointer():
    val_in = "hello, world"
    obj = py_to_c(val_in, c_type=ctypes.c_void_p, type_name="String")
    assert obj.value == val_in.encode()


def test_roundtrip_int():
    in_ = 23
    ptr = ctypes.POINTER(ctypes.c_int32)(ctypes.c_int32(in_))
    ptr = ctypes.byref(ctypes.c_int32(in_))  # type: ignore[assignment]

    int_void = ctypes.cast(ptr, ctypes.c_void_p)
    out = ctypes.cast(int_void, ctypes.POINTER(ctypes.c_int32)).contents.value
    assert in_ == out


def test_roundtrip_ffislice_int():
    in_ = 23
    ptr = ctypes.byref(ctypes.c_int32(in_))
    ffi_slice = FfiSlice(ctypes.cast(ptr, ctypes.c_void_p), 1)
    out = ctypes.cast(ffi_slice.ptr, ctypes.POINTER(ctypes.c_int32)).contents.value
    assert out == in_


def test_roundtrip_ffisliceptr_int():
    in_ = 23
    ptr = ctypes.byref(ctypes.c_int32(in_))
    ffi_slice = FfiSlice(ctypes.cast(ptr, ctypes.c_void_p), 1)
    ffi_slice_ptr = FfiSlicePtr(ffi_slice)
    ffi_slice = ffi_slice_ptr.contents
    out = ctypes.cast(ffi_slice.ptr, ctypes.POINTER(ctypes.c_int32)).contents.value
    assert out == in_


def test_roundtrip_ffisliceptr_int_lib():
    in_ = 23
    ffi_slice_ptr = _scalar_to_slice(in_, "i32")
    out = _slice_to_scalar(ffi_slice_ptr, "i32")
    assert out == in_


def test_vec_str():
    data = ["a", "bbb", "c"]
    # partial roundtrip
    slice = _vector_to_slice(data, type_name=Vec[str])
    assert _slice_to_vector(slice, type_name=Vec[str]) == data

    # complete roundtrip
    any = py_to_c(data, c_type=AnyObjectPtr)
    assert c_to_py(any) == data


def test_hashmap():
    data = {"A": 23, "B": 12, "C": 234}
    slice = _hashmap_to_slice(data, HashMap[str, int])
    assert _slice_to_hashmap(slice) == data

    # complete roundtrip
    any = py_to_c(data, c_type=AnyObjectPtr)
    assert c_to_py(any) == data


def test_overflow():
    with pytest.raises(ValueError):
        py_to_c(-1, AnyObjectPtr, u8)

    with pytest.raises(ValueError):
        py_to_c(256, AnyObjectPtr, u8)

    with pytest.raises(ValueError):
        py_to_c(-129, AnyObjectPtr, i8)


def test_check_and_cast_scalar():
    # Any number is cast to float, if f32 or f64 is expected:
    assert _check_and_cast_scalar("f32", 1.0) == 1.0
    assert _check_and_cast_scalar("f64", 1) == 1.0

    # Ints cast to ints, unsurprisingly:
    assert _check_and_cast_scalar("u8", 1) == 1

    # There are range checks for ints:
    with pytest.raises(ValueError, match="256 is not representable by u8"):
        _check_and_cast_scalar("u8", 256)

    # Floats don't have range checks because inf is a valid float:
    assert _check_and_cast_scalar("f32", 1e100) == 1e100

    # Floats cannot be cast to ints:
    with pytest.raises(TypeError, match="inferred type is f64, expected i32."):
        _check_and_cast_scalar("i32", 1.0)

    # Bools can only cast to bools:
    assert _check_and_cast_scalar("bool", True)

    # Bools cannot cast to ints:
    with pytest.raises(TypeError, match="inferred type is bool, expected u8."):
        _check_and_cast_scalar("u8", True)

    # Bools cannot cast to floats:
    with pytest.raises(TypeError, match="inferred type is bool, expected f64."):
        _check_and_cast_scalar("f64", True)

    with pytest.raises(TypeError, match="inferred type is i32, expected bool."):
        _check_and_cast_scalar("bool", 1)

    with pytest.raises(TypeError, match="inferred type is i32, expected fake."):
        _check_and_cast_scalar("fake", 1)


@pytest.mark.parametrize(
    "val_in, type_name",
    [
        (Margin(by=[]), "Margin"),
        (Bound(by=[]), "Bound"),
        ([Bound(by=[])], "Bounds"),
    ],
)
def test_extras_object(val_in, type_name):
    from opendp._convert import py_to_c, c_to_py
    from opendp._lib import AnyObjectPtr

    obj = py_to_c(val_in, c_type=AnyObjectPtr, type_name=type_name)
    val_out = c_to_py(obj)
    assert val_out == val_in
