import pytest

import opendp.prelude as dp

np = pytest.importorskip("numpy")

@pytest.mark.parametrize(
    "value,type_name,dtype",
    [
        ([1, 2], "Vec<i32>", "int32"),
        (1, "i32", "int32"),
        ([1.0, 2.0], "Vec<f64>", "float64"),
        (1.0, "f64", None),
        ("A", "String", None),
    ],
)
def test_numpy_data(value, type_name, dtype):
    array = np.array(value, dtype=dtype) if dtype is not None else np.array(value)
    assert c_to_py(py_to_c(array, AnyObjectPtr, type_name=type_name)) == value


def test_as_array():
    result = dp.as_array()(np.array([1, 2], dtype=np.int32))
    assert isinstance(result, np.ndarray)
    assert np.array_equal(result, np.array([1, 2], dtype=np.int32))


def test_numpy_string_vector_roundtrip():
    # `Vec<String>` still goes through the standard vector path, not the atomic ndarray fast path.
    assert c_to_py(
        py_to_c(np.array(["A", "B"]), AnyObjectPtr, type_name="Vec<String>")
    ) == ["A", "B"]


def test_numpy_ndarray_roundtrip():
    type_name = RuntimeType("NDArray", ["i32"])

    raw = _py_to_slice(np.array([1, 2, 3], dtype=np.int32), type_name)
    result = _slice_to_numpy(raw, type_name)

    assert isinstance(result, np.ndarray)
    assert np.array_equal(result, np.array([1, 2, 3], dtype=np.int32))


def test_numpy_vec_input_uses_ndarray_loader():
    type_name = RuntimeType("Vec", ["i32"])

    raw = _vector_to_slice(np.array([1, 2, 3], dtype=np.int32), type_name)
    result = _slice_to_vector(raw, type_name)

    assert list(result) == [1, 2, 3]


def test_numpy_ndarray_validation():
    type_name = RuntimeType("NDArray", ["i32"])

    with pytest.raises(ValueError, match="unrecognized numpy dtype"):
        _numpy_dtype_for_rust_type("String")

    with pytest.raises(TypeError, match="Expected type is NDArray<i32>"):
        _numpy_to_slice([1, 2, 3], type_name)

    with pytest.raises(TypeError, match="Only 1d arrays are currently supported"):
        _numpy_to_slice(np.array([[1, 2], [3, 4]], dtype=np.int32), type_name)

    with pytest.raises(TypeError, match="Expected dtype int32, got int64"):
        _numpy_to_slice(np.array([1, 2, 3], dtype=np.int64), type_name)


def test_numpy_trans():
    assert (
        dp.t.make_sum(
            dp.vector_domain(dp.atom_domain(bounds=(0, 10))),
            dp.symmetric_distance(),
        )(np.array([1, 2, 3], dtype=np.int32))
        == 6
    )


def test_numpy_vector_output_from_rust_transformation():
    trans = dp.t.make_clamp(
        dp.vector_domain(dp.atom_domain(T=int)),
        dp.symmetric_distance(),
        bounds=(0, 10),
    )

    result = dp.as_array()(trans(np.array([-1, 2, 11], dtype=np.int32)))

    assert isinstance(result, np.ndarray)
    assert np.array_equal(result, np.array([0, 2, 10], dtype=np.int32))


def test_bitvec():
    for i in range(1, 20):
        data = np.packbits([1] * i)
        obj = py_to_c(data.tobytes(), AnyObjectPtr, "BitVector")
        val_out = np.frombuffer(c_to_py(obj), dtype=np.uint8)
        bits = np.unpackbits(val_out)
        assert (bits.tolist() + [0]).index(0) == i