from opendp._convert import *
from opendp._convert import _scalar_to_slice, _slice_to_scalar


def test_data_object_int():
    val_in = 123
    obj = py_to_c(val_in, c_type=AnyObjectPtr)
    print(obj)
    val_out = c_to_py(obj)
    assert val_out == val_in


def test_data_object_float():
    val_in = 123.123
    obj = py_to_c(val_in, c_type=AnyObjectPtr)
    val_out = c_to_py(obj)
    assert val_out == val_in


def test_data_object_str():
    val_in = "hello, world"
    obj = py_to_c(val_in, c_type=AnyObjectPtr)
    val_out = c_to_py(obj)
    assert val_out == val_in


def test_data_object_list():
    val_in = [1, 2, 3]
    obj = py_to_c(val_in, c_type=AnyObjectPtr)
    val_out = c_to_py(obj)
    assert val_out == val_in


def test_data_object_tuple():
    val_in = (1., 1e-7)
    obj = py_to_c(val_in, c_type=AnyObjectPtr)
    val_out = c_to_py(obj)
    assert val_out == val_in


def test_roundtrip_int():
    in_ = 23
    ptr = ctypes.POINTER(ctypes.c_int32)(ctypes.c_int32(in_))
    ptr = ctypes.byref(ctypes.c_int32(in_))

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
    print(out)
    assert out == in_


def test_roundtrip_ffisliceptr_int_lib():
    in_ = 23
    ffi_slice_ptr = _scalar_to_slice(in_, 'i32')
    out = _slice_to_scalar(ffi_slice_ptr, 'i32')
    print(out)
    assert out == in_