"""The native collector protocol and AnyQueryable callback return boundary."""

from copy import copy
import gc
import weakref

import pytest

import opendp.prelude as dp
from opendp._convert import c_to_py, py_to_c
from opendp._internal import _make_measurement
from opendp._lib import AnyObjectPtr
from opendp.extras.polars import OnceFrame

pl = pytest.importorskip("polars")


def test_collector_measurement_returns_native_onceframe(tmp_path):
    from opendp._data import onceframe_split, onceframe_with_collector
    from opendp.core import queryable_query_type

    calls = []

    def collect(unused):
        assert unused is False
        calls.append(True)
        return pl.DataFrame({"value": [7]})

    original = onceframe_with_collector(collect)
    alias = copy(original)
    measurement = _make_measurement(
        dp.atom_domain(T="i32"),
        dp.absolute_distance(T="i32"),
        dp.max_divergence(),
        lambda _: original,
        float,
        TO="AnyQueryable",
    )
    frame = measurement(1)
    assert type(frame) is OnceFrame
    assert queryable_query_type(frame.queryable) == "OnceFrameQuery"
    assert type(original.queryable) is AnyObjectPtr
    # Rejected extraction does not invoke or consume the collector.
    with pytest.raises(dp.OpenDPException, match="no LazyFrame"):
        frame.lazy()
    with pytest.raises(dp.OpenDPException, match="no LazyFrame"):
        onceframe_split(frame.queryable, str(tmp_path / "absent"))
    assert calls == []
    assert frame.collect().to_dict(as_series=False) == {"value": [7]}
    assert calls == [True]
    for used in [frame, original, alias]:
        with pytest.raises(dp.OpenDPException, match="exhaust"):
            used.collect()


def test_collector_failure_propagates_and_exhausts():
    from opendp._data import onceframe_with_collector

    calls = []

    def fail(_):
        calls.append(True)
        raise ValueError("collector failure sentinel")

    frame = onceframe_with_collector(fail)
    alias = copy(frame)
    with pytest.raises(dp.OpenDPException, match="collector failure sentinel"):
        frame.collect()
    for used in [frame, alias]:
        with pytest.raises(dp.OpenDPException, match="exhaust"):
            used.collect()
    assert calls == [True]


def test_cloned_queryable_owns_callback_until_consumption():
    from opendp._data import onceframe_with_collector

    class Collector:
        def __call__(self, _):
            return pl.DataFrame({"value": [1]})

    callback = Collector()
    retained = weakref.ref(callback)
    original = onceframe_with_collector(callback)
    frame = c_to_py(py_to_c(original, AnyObjectPtr, "AnyQueryable"))
    del original, callback
    gc.collect()
    assert retained() is not None
    assert frame.collect().item() == 1
    gc.collect()
    assert retained() is None
