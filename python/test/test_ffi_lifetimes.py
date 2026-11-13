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


def test_retained_callback_queryable_outlives_producing_measurement():
    dp.enable_features("contrib", "honest-but-curious")
    rr = dp.m.make_randomized_response_bool(0.75)
    saved = []
    for _ in range(3):
        adaptive = dp.c.make_adaptive_composition(
            rr.input_domain, rr.input_metric, rr.output_measure, 1, [2.0, 2.0]
        )
        post = dp.new_function(lambda q: saved.append(q) or 0, int)
        chained = dp.c.make_chain_pm(post, adaptive)
        assert chained(True) == 0
        del chained, post, adaptive
        gc.collect()
        assert isinstance(saved[0](rr), bool)
        assert isinstance(saved[0](rr), bool)
        with pytest.raises(dp.OpenDPException):
            saved[0](rr)
        saved.clear()
        gc.collect()


def test_borrowed_profile_conversion_owns_an_independent_clone():
    import ctypes
    from opendp._lib import AnyObject, AnyObjectPtr

    dp.enable_features("contrib", "honest-but-curious")
    profile = dp.new_privacy_profile(lambda epsilon: 1.0 if epsilon < 0.5 else 0.0)
    borrowed = ctypes.cast(profile.curve, ctypes.POINTER(AnyObject))
    copied = _convert.c_to_py(borrowed)
    assert isinstance(copied.curve, AnyObjectPtr)
    del borrowed, profile
    gc.collect()
    assert copied.delta(0.0) == 1.0
    assert copied.delta(1.0) == 0.0


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
