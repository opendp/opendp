"""Check typed scalar execution at the two-stage database boundary."""

import pickle
from types import SimpleNamespace

import pytest
import opendp.prelude as dp
from opendp.extras.ibis import _run_ffi_split_plan

pl = pytest.importorskip("polars")


def plugin(name, dtype, *, support=None, symbol="noise_plugin"):
    return SimpleNamespace(
        name=name,
        input_dtype=dtype,
        parameters={
            "symbol": symbol,
            "kwargs": list(
                pickle.dumps(
                    {
                        "distribution": "Laplace",
                        "support": support
                        or ("Integer" if dtype.is_integer() else "Float"),
                        # Zero is a deterministic conversion control only.
                        "scale": 0.0,
                    }
                )
            ),
        },
    )


def plan_and_connection(plugins, frame, *, postprocess=lambda frame: frame):
    table = object()
    plan = SimpleNamespace(ibis_table=table, plugins=plugins, postprocess=postprocess)
    sql_calls = []

    def to_polars(received):
        assert received is table
        sql_calls.append(received)
        return frame

    return plan, SimpleNamespace(to_polars=to_polars), sql_calls


@pytest.mark.parametrize(
    "dtype,value",
    [
        (pl.Int32, 2147483647),
        (pl.Int64, 2147483651),
        (pl.UInt32, 4294967295),
        (pl.UInt64, 18446744073709551615),
        (pl.Float64, 1.25),
    ],
)
def test_run_ffi_split_plan_preserves_native_carrier(dtype, value):
    observed = []

    def postprocess(frame):
        observed.append(frame.schema)
        return frame

    plan, connection, calls = plan_and_connection(
        [plugin("aggregate", dtype)],
        pl.DataFrame([pl.Series("aggregate", [value], dtype=dtype)]),
        postprocess=postprocess,
    )
    assert _run_ffi_split_plan(plan, connection) == [value]
    assert len(calls) == 1
    assert observed == [{"aggregate": dtype}]


def test_run_ffi_split_plan_validates_all_values_before_noise(monkeypatch):
    calls = []
    original_call = dp.Measurement.__call__

    def observed_call(measurement, value):
        calls.append(value)
        return original_call(measurement, value)

    monkeypatch.setattr(dp.Measurement, "__call__", observed_call)
    plan, connection, _ = plan_and_connection(
        [plugin("sum", pl.Int64), plugin("count", pl.UInt32)],
        pl.DataFrame({"sum": [10], "count": [-1]}),
    )
    with pytest.raises(dp.OpenDPException, match="native noise type"):
        _run_ffi_split_plan(plan, connection)
    assert not calls


@pytest.mark.parametrize("value", [None, True, 1.5, "synthetic-private-marker"])
def test_run_ffi_split_plan_rejects_noninteger_carrier(value):
    plan, connection, _ = plan_and_connection(
        [plugin("aggregate", pl.Int64)], pl.DataFrame({"aggregate": [value]})
    )
    with pytest.raises(dp.OpenDPException, match="native noise type") as error:
        _run_ffi_split_plan(plan, connection)
    assert "synthetic-private-marker" not in str(error.value)


@pytest.mark.parametrize(
    "record",
    [
        plugin("aggregate", pl.Int64, support="Float"),
        plugin("aggregate", pl.Float64, support="Integer"),
        plugin("aggregate", pl.String),
        plugin("aggregate", pl.Int8),
        plugin("aggregate", pl.Int16),
        plugin("aggregate", pl.Float32),
        plugin("aggregate", pl.Int64, symbol="other_plugin"),
    ],
)
def test_run_ffi_split_plan_rejects_metadata_before_sql(record):
    plan, connection, calls = plan_and_connection(
        [record], pl.DataFrame({"aggregate": [1]})
    )
    with pytest.raises(dp.OpenDPException):
        _run_ffi_split_plan(plan, connection)
    assert not calls


@pytest.mark.parametrize(
    "frame",
    [
        pl.DataFrame({"aggregate": []}),
        pl.DataFrame({"aggregate": [1, 2]}),
        pl.DataFrame({"wrong": [1]}),
        pl.DataFrame({"aggregate": [1], "extra": [2]}),
    ],
)
def test_run_ffi_split_plan_rejects_sql_shape(frame, monkeypatch):
    noise_calls = []
    monkeypatch.setattr(
        dp.Measurement, "__call__", lambda *args: noise_calls.append(args)
    )
    plan, connection, _ = plan_and_connection([plugin("aggregate", pl.Int64)], frame)
    with pytest.raises(dp.OpenDPException, match="not 1-1"):
        _run_ffi_split_plan(plan, connection)
    assert not noise_calls


def test_run_ffi_split_plan_postprocesses_separately_noised_values(monkeypatch):
    scripted = iter([16, 6])
    inputs = []

    def controlled_noise(measurement, value):
        inputs.append(value)
        return next(scripted)

    monkeypatch.setattr(dp.Measurement, "__call__", controlled_noise)
    plan, connection, _ = plan_and_connection(
        [plugin("sum", pl.Int64), plugin("count", pl.UInt32)],
        pl.DataFrame({"sum": [10], "count": [4]}),
        postprocess=lambda frame: frame.select(pl.col("sum") / pl.col("count")),
    )
    assert _run_ffi_split_plan(plan, connection) == [16 / 6]
    assert inputs == [10, 4]


def test_run_ffi_split_plan_rejects_invalid_mechanism_output(monkeypatch):
    monkeypatch.setattr(dp.Measurement, "__call__", lambda *args: -1)
    plan, connection, _ = plan_and_connection(
        [plugin("count", pl.UInt32)], pl.DataFrame({"count": [4]})
    )
    with pytest.raises(dp.OpenDPException, match="native noise type"):
        _run_ffi_split_plan(plan, connection)
