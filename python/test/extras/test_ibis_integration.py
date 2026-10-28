"""Run a synthetic aggregate through the real Polars/Ibis/native boundary."""

import json
import math
import pickle
import re
import sys
from uuid import uuid4
from types import SimpleNamespace

import pytest
import opendp.prelude as dp
from ..helpers import unsupported_native_carrier_error

pl = pytest.importorskip("polars")
polars_to_ibis = pytest.importorskip("polars_to_ibis")


@pytest.fixture(params=["sqlite", "duckdb"])
def database(request):
    if request.param == "duckdb":
        pytest.importorskip("duckdb")

    connection = dp.ibis.get_connection(request.param)
    table_name = f"noise_metadata_{uuid4().hex}"
    try:
        # Load only synthetic values, without registering an Ibis memtable.
        connection.create_table(table_name, schema={"value": "int64"})
        connection.raw_sql(
            f'INSERT INTO "{table_name}" ("value") VALUES (1), (2), (3), (4)'
        )
        yield connection, table_name
    finally:
        connection.disconnect()


# The translator currently uses Polars' deprecated JSON plan serialization.
@pytest.mark.filterwarnings(
    "ignore:'json' serialization format of LazyFrame is deprecated:UserWarning"
)
@pytest.mark.parametrize("distribution", ["Laplace", "Gaussian"])
def test_run_on_database_native_noise_parameters(monkeypatch, database, distribution):
    from opendp import _data
    from opendp.extras.ibis import run_on_database

    connection, table_name = database
    loss = {"epsilon": 1.0} if distribution == "Laplace" else {"rho": 0.5}
    context = dp.Context.compositor(
        data=dp.ibis.scan_database(connection, table_name),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(**loss),
        split_evenly_over=1,
        margins=[dp.polars.Margin(max_length=4)],
    )
    query = context.query().select(
        dp.len(signed=False),
        pl.col("value").fill_null(0).dp.sum((0, 10)).alias("sum_value"),
    )

    summary_frame = query.summarize()
    assert isinstance(summary_frame, pl.DataFrame)
    expected_summary = summary_frame.to_dicts()

    aggregates = []
    original_to_polars = connection.to_polars

    def observed_to_polars(*args, **kwargs):
        result = original_to_polars(*args, **kwargs)
        aggregates.append(list(result.row(0)))
        return result

    parameters = []
    original_decode = _data.decode_noise_plugin_parameters

    def observed_decode(*args, **kwargs):
        result = original_decode(*args, **kwargs)
        parameters.append(json.loads(result))
        return result

    original_loads = pickle.loads

    def guarded_loads(*args, **kwargs):
        # Other libraries may legitimately use pickle internally.
        if sys._getframe(1).f_code is run_on_database.__code__:
            pytest.fail("run_on_database must not load pickled Python objects")
        return original_loads(*args, **kwargs)

    monkeypatch.setattr(connection, "to_polars", observed_to_polars)
    monkeypatch.setattr(_data, "decode_noise_plugin_parameters", observed_decode)
    monkeypatch.setattr(pickle, "loads", guarded_loads)

    error_message = (
        unsupported_native_carrier_error("i64")
        if hasattr(polars_to_ibis, "split_polars_on_ffi_plan")
        else None
    )
    if error_message is not None:
        with pytest.raises(dp.OpenDPException, match=re.escape(error_message)):
            run_on_database(query, connection, table_name)
        assert not aggregates
        return

    results = run_on_database(query, connection, table_name)

    # Check the actual SQL results separately from the randomized outputs.
    assert aggregates == [[4, 10]]
    assert len(results) == 2
    assert all(type(result) is int and math.isfinite(result) for result in results)
    assert len(parameters) == 2
    assert len(expected_summary) == 2
    for parameter, summary in zip(parameters, expected_summary):
        assert parameter["distribution"] == distribution
        assert parameter["support"] == "Integer"
        assert summary["distribution"] == f"Integer {distribution}"
        assert parameter["scale"] == summary["scale"]
        assert math.isfinite(parameter["scale"]) and parameter["scale"] > 0


@pytest.mark.filterwarnings(
    "ignore:'json' serialization format of LazyFrame is deprecated:UserWarning"
)
@pytest.mark.parametrize("distribution", ["Laplace", "Gaussian"])
def test_run_on_database_mean_postprocessing(monkeypatch, database, distribution):
    import polars_to_ibis
    from opendp import _data

    split = getattr(polars_to_ibis, "split_polars_on_ffi_plan", None)
    if split is None:
        pytest.skip("Requires the two-stage translator API")

    connection, table_name = database
    loss = {"epsilon": 1.0} if distribution == "Laplace" else {"rho": 0.5}
    context = dp.Context.compositor(
        data=dp.ibis.scan_database(connection, table_name),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(**loss),
        split_evenly_over=1,
        margins=[dp.polars.Margin(max_length=4)],
    )
    query = context.query().select(
        pl.col("value").fill_null(0).dp.mean((0, 10)).alias("mean_value")
    )
    summary_frame = query.summarize()
    assert isinstance(summary_frame, pl.DataFrame)
    summary = summary_frame.to_dicts()

    aggregates = []
    original_to_polars = connection.to_polars

    def observed_to_polars(*args, **kwargs):
        frame = original_to_polars(*args, **kwargs)
        aggregates.append(list(frame.row(0)))
        return frame

    parameters = []
    original_decode = _data.decode_noise_plugin_parameters

    def observed_decode(*args, **kwargs):
        decoded = original_decode(*args, **kwargs)
        parameters.append(json.loads(decoded))
        return decoded

    plans = []
    noised = []
    postprocessed = []

    def observed_split(*args, **kwargs):
        plan = split(*args, **kwargs)
        plans.append(plan)

        def postprocess(frame):
            noised.append(frame)
            result = plan.postprocess(frame)
            postprocessed.append(result)
            return result

        return SimpleNamespace(
            ibis_table=plan.ibis_table, plugins=plan.plugins, postprocess=postprocess
        )

    monkeypatch.setattr(connection, "to_polars", observed_to_polars)
    monkeypatch.setattr(_data, "decode_noise_plugin_parameters", observed_decode)
    monkeypatch.setattr(polars_to_ibis, "split_polars_on_ffi_plan", observed_split)

    error_message = unsupported_native_carrier_error("i64")
    if error_message is not None:
        with pytest.raises(dp.OpenDPException, match=re.escape(error_message)):
            dp.ibis.run_on_database(query, connection, table_name)
        assert not aggregates
        assert not noised
        assert not postprocessed
        return

    [result] = dp.ibis.run_on_database(query, connection, table_name)

    assert aggregates == [[10, 4]]
    assert len(plans) == len(noised) == len(postprocessed) == 1
    assert [plugin.input_dtype for plugin in plans[0].plugins] == [pl.Int64, pl.UInt32]
    assert list(noised[0].schema.values()) == [pl.Int64, pl.UInt32]
    assert postprocessed[0].columns == ["mean_value"]
    assert len(parameters) == len(summary) == 2
    for parameter, row in zip(parameters, summary):
        assert parameter["distribution"] == distribution
        assert parameter["support"] == "Integer"
        assert parameter["scale"] == row["scale"]

    numerator, denominator = noised[0].row(0)
    # Compare arithmetic on the actual two noised values, not random draws.
    if denominator:
        expected = float(numerator) / float(denominator)
    elif numerator:
        expected = math.copysign(math.inf, numerator)
    else:
        expected = math.nan
    if math.isnan(expected):
        assert math.isnan(result)
    else:
        assert result == expected
