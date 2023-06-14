from opendp._convert import *
from opendp._convert import (
    _scalar_to_slice, _slice_to_scalar,
    _vector_to_slice, _slice_to_vector,
    _hashmap_to_slice, _slice_to_hashmap,
)
from opendp.typing import *
import pytest
import sys
try:
    import numpy as np
except ImportError:
    pass


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


@pytest.mark.skipif('numpy' not in sys.modules,
                    reason="requires the Numpy library")
def test_numpy_data():
    def roundtrip(value, type_name, dtype=None):
        print(c_to_py(py_to_c(np.array(value, dtype=dtype), AnyObjectPtr, type_name=type_name)))
    roundtrip([1, 2], "Vec<i32>", dtype=np.int32)
    roundtrip(1, "i32", dtype=np.int32)
    roundtrip([1., 2.], "Vec<f64>")
    roundtrip(1., "f64")
    roundtrip(["A", "B"], "Vec<String>")
    roundtrip("A", "String")

@pytest.mark.skipif('numpy' not in sys.modules,
                    reason="requires the Numpy library")
def test_numpy_trans():
    import opendp.prelude as dp
    dp.enable_features("contrib")
    assert dp.t.make_sum(
        dp.vector_domain(dp.atom_domain(bounds=(0, 10))), 
        dp.symmetric_distance(),
    )(np.array([1, 2, 3], dtype=np.int32)) == 6


def test_overflow():
    import pytest
    with pytest.raises(ValueError):
        py_to_c(-1, AnyObjectPtr, u8)

    with pytest.raises(ValueError):
        py_to_c(256, AnyObjectPtr, u8)
  
    with pytest.raises(ValueError):
        py_to_c(-129, AnyObjectPtr, i8)
