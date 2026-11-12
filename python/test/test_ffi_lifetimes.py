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


@pytest.mark.parametrize("gaussian", [False, True])
def test_noise_dispatch_type_strings_on_success_and_error(gaussian):
    dp.enable_features("contrib", "honest-but-curious")
    measure = dp.zero_concentrated_divergence() if gaussian else dp.max_divergence()
    distance = dp.absolute_distance(T=float)
    threshold_metric = (
        dp.l02inf_distance(distance) if gaussian else dp.l01inf_distance(distance)
    )
    domain = dp.atom_domain(T=float, nan=False)
    map_domain = dp.map_domain(dp.atom_domain(T=str), domain)
    for _ in range(10):
        assert dp.m.make_noise(domain, distance, measure, 1.0).map(1.0) > 0
        dp.m.make_noise_threshold(
            map_domain, threshold_metric, dp.approximate(measure), 1.0, 10.0
        )
        with pytest.raises(dp.OpenDPException):
            dp.m.make_noise(domain, distance, measure, -1.0)
        with pytest.raises(dp.OpenDPException):
            dp.m.make_noise_threshold(
                map_domain, threshold_metric, dp.approximate(measure), -1.0, 10.0
            )


@pytest.mark.parametrize("kind", ["object", "metric", "measure", "function"])
def test_finalizers_release_native_references_without_imports(kind, monkeypatch):
    import builtins
    import weakref

    dp.enable_features("contrib", "honest-but-curious")

    class Payload:
        pass

    def make_owner(value):
        if kind == "object":
            return _convert.py_to_c(value, AnyObjectPtr, "ExtrinsicObject")
        if kind == "metric":
            return dp.user_distance("lifetime", descriptor=value)
        if kind == "measure":
            return dp.user_divergence("lifetime", descriptor=value)
        return dp.new_function(lambda _, payload=value: payload, "ExtrinsicObject")

    value = Payload()
    reference = weakref.ref(value)
    owner = make_owner(value)
    del value
    assert reference() is not None
    original_import = builtins.__import__

    def unavailable(name, *args, **kwargs):
        if name.startswith("opendp"):
            raise ImportError("module teardown")
        return original_import(name, *args, **kwargs)

    with monkeypatch.context() as patch:
        patch.setattr(builtins, "__import__", unavailable)
        del owner
        gc.collect()
    assert reference() is None


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


@pytest.mark.parametrize("fail", [False, True])
def test_legacy_pointer_array_is_released_without_consuming_elements(monkeypatch, fail):
    obj = _convert.py_to_c(42, AnyObjectPtr, "i32")
    # One AnyObject is also a valid contiguous one-element AnyObject array.
    raw = _convert._wrap_in_slice(obj, 1)
    release = _convert.lib.opendp_data__ffislice_of_anyobjectptrs_free
    releases = []

    def track_release(view):
        releases.append(view.contents.len)
        return release(view)

    def failed_conversion(*_):
        raise ValueError("nested conversion failed")

    with monkeypatch.context() as patch:
        patch.setattr(
            _convert.lib, "opendp_data__ffislice_of_anyobjectptrs_free", track_release
        )
        if fail:
            patch.setattr(_convert, "_slice_to_py", failed_conversion)
            with pytest.raises(ValueError, match="nested conversion failed"):
                _convert._slice_to_vector(raw, dp.Vec["AnyObject"])
        else:
            assert _convert._slice_to_vector(raw, dp.Vec["AnyObject"]) == [42]
    assert releases == [1]
    assert _convert.c_to_py(obj) == 42
    assert (
        _convert._slice_to_vector(_convert._wrap_in_slice(obj, 0), dp.Vec["AnyObject"])
        == []
    )


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
