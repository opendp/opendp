"""Ownership regressions; also run these tests under Valgrind."""

import gc

import pytest

import opendp.prelude as dp
from opendp import _convert
from opendp._lib import AnyObjectPtr


def test_callback_argument_conversion_error_does_not_free_borrowed_object():
    dp.enable_features("contrib", "honest-but-curious")
    callback = dp.new_function(lambda _: 0, int)
    for _ in range(3):
        # Vec<tuple> can be constructed, but not copied back out of Rust.
        with pytest.raises(dp.OpenDPException):
            callback([(1.0, 2.0)])
        assert callback(1) == 0
    gc.collect()


def test_transition_argument_conversion_error_does_not_free_borrowed_object():
    dp.enable_features("contrib")
    queryable = dp.new_queryable(lambda _: 0, Q="Vec<(f64, f64)>", A=int)
    for _ in range(3):
        with pytest.raises(dp.OpenDPException):
            queryable([(1.0, 2.0)])
    gc.collect()


def test_nested_conversion_error_does_not_free_source(monkeypatch):
    obj = _convert.py_to_c(42, AnyObjectPtr, "i32")
    raw = _convert._wrap_in_slice(obj, 1)

    def fail(_):
        raise ValueError("conversion failed")

    with monkeypatch.context() as patch:
        patch.setattr(_convert, "c_to_py", fail)
        with pytest.raises(ValueError, match="conversion failed"):
            _convert._slice_to_anyobject(raw)
    assert _convert.c_to_py(obj) == 42


def test_hashmap_conversion_error_does_not_free_input_slice(monkeypatch):
    raw = _convert._hashmap_to_slice({"a": 1}, dp.HashMap[str, int])

    def fail(_):
        raise ValueError("conversion failed")

    with monkeypatch.context() as patch:
        patch.setattr(_convert, "c_to_py", fail)
        with pytest.raises(ValueError, match="conversion failed"):
            _convert._slice_to_hashmap(raw)
    assert _convert._slice_to_hashmap(raw) == {"a": 1}
