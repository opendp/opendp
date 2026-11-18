"""Lifetime regressions for Rust-owned FFI output scratch views."""

import gc

import pytest

import opendp.prelude as dp
from opendp import _convert
from opendp._lib import AnyObjectPtr


@pytest.mark.parametrize(
    "value,type_name",
    [
        ("hello λ", "String"),
        (["", "hello", "λ"], "Vec<String>"),
        ([], "Vec<String>"),
        ((1.0, 2.0), "(f64, f64)"),
        ({"a": 1, "b": 2}, "HashMap<String, i32>"),
    ],
)
def test_owned_scratch_roundtrip_and_reuse(value, type_name):
    obj = _convert.py_to_c(value, AnyObjectPtr, type_name)
    for _ in range(10):
        assert _convert.c_to_py(obj) == value
    del obj
    gc.collect()


def test_profile_tuple_survives_scratch_and_source_release():
    dp.enable_features("contrib", "honest-but-curious")
    measurement = dp.m.make_user_measurement(
        dp.atom_domain(T=bool),
        dp.absolute_distance(T=float),
        dp.approximate(dp.zero_concentrated_divergence()),
        lambda x: x,
        lambda d: (d * 0.05, d * 1e-7),
    )
    converted = dp.c.make_zCDP_to_approxDP(measurement)
    for _ in range(10):
        profile, delta = converted.map(1.0)
        assert delta == 1e-7
        assert 0 <= profile.delta(1.0) <= 1
    del converted, measurement
    gc.collect()
    assert 0 <= profile.delta(1.0) <= 1


def test_owned_scratch_released_on_conversion_error(monkeypatch):
    obj = _convert.py_to_c(["a", "b"], AnyObjectPtr, "Vec<String>")
    release = _convert.lib.opendp_data__slice_owner_free
    released = []

    def track_release(owner):
        released.append(owner.value)
        return release(owner)

    def fail(*_):
        raise ValueError("conversion failed")

    with monkeypatch.context() as patch:
        patch.setattr(_convert.lib, "opendp_data__slice_owner_free", track_release)
        patch.setattr(_convert, "_slice_to_py", fail)
        with pytest.raises(ValueError, match="conversion failed"):
            _convert.c_to_py(obj)
    assert len(released) == 1
    assert _convert.c_to_py(obj) == ["a", "b"]
