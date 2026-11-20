"""Independent FFI cleanup and lifetime regressions."""

import gc

import pytest

import opendp.prelude as dp
from opendp import _convert
from opendp._lib import AnyObjectPtr


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


def test_ffi_slice_dependencies_have_local_lifetimes():
    import weakref

    class Marker:
        pass

    from opendp._lib import FfiSlice, FfiSlicePtr

    view = FfiSlicePtr(FfiSlice())
    marker = Marker()
    marker_ref = weakref.ref(marker)
    view.depends_on(marker)
    del marker
    gc.collect()
    assert marker_ref() is not None

    # Dependencies belong to their pointer, not a global id-indexed registry.
    assert view._keepalive
    del view
    gc.collect()
    assert marker_ref() is None
