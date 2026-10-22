"""Exercise the native parameter boundary without running a database server."""

import json
import math
import pickle
import sys
from types import SimpleNamespace

import pytest
import opendp.prelude as dp
from opendp.extras.ibis import run_on_database

pl = pytest.importorskip("polars")
polars_to_ibis = pytest.importorskip("polars_to_ibis")


def parameters(distribution="Laplace", support="Integer", scale=1.25):
    return dict(distribution=distribution, support=support, scale=scale)


def plugin(kwargs, symbol="noise_plugin"):
    return dict(symbol=symbol, kwargs=list(kwargs))


def query_and_connection(monkeypatch, plugins, values=(42,)):
    """Stub translation and SQL execution; retain the real FFI and noise mechanisms."""
    query = SimpleNamespace(release=lambda: SimpleNamespace(lazy=lambda: object()))
    table = object()
    monkeypatch.setattr(
        polars_to_ibis, "split_polars_on_ffi", lambda *a, **kw: (table, plugins)
    )
    records = tuple(
        SimpleNamespace(
            name=f"c{i}",
            parameters=plugin,
            input_dtype=(
                pl.Float64 if i < len(values) and type(values[i]) is float else pl.Int32
            ),
        )
        for i, plugin in enumerate(plugins)
    )
    plan = SimpleNamespace(
        ibis_table=table, plugins=records, postprocess=lambda frame: frame
    )
    monkeypatch.setattr(
        polars_to_ibis,
        "split_polars_on_ffi_plan",
        lambda *a, **kw: plan,
        raising=False,
    )
    calls = []

    def to_polars(received_table):
        assert received_table is table
        calls.append(received_table)
        return pl.DataFrame({f"c{i}": [value] for i, value in enumerate(values)})

    return query, SimpleNamespace(to_polars=to_polars), calls


@pytest.mark.parametrize("protocol", range(pickle.HIGHEST_PROTOCOL + 1))
@pytest.mark.parametrize("distribution", ["Laplace", "Gaussian"])
@pytest.mark.parametrize("support", ["Integer", "Float"])
def test_decode_noise_plugin_parameters(protocol, distribution, support):
    from opendp._data import decode_noise_plugin_parameters

    expected = parameters(distribution, support)
    decoded = decode_noise_plugin_parameters(list(pickle.dumps(expected, protocol)))
    assert json.loads(decoded) == expected


@pytest.mark.parametrize(
    "scale", [0.0, -0.0, math.ulp(0.0), sys.float_info.min, sys.float_info.max]
)
def test_decode_noise_plugin_parameters_float_roundtrip(scale):
    from opendp._data import decode_noise_plugin_parameters

    decoded = json.loads(
        decode_noise_plugin_parameters(list(pickle.dumps(parameters(scale=scale))))
    )
    # Do not sample at extreme scales: this checks only the Rust-to-Python boundary.
    assert decoded["scale"].hex() == scale.hex()


@pytest.mark.parametrize("distribution", ["Laplace", "Gaussian"])
@pytest.mark.parametrize("support,value", [("Integer", 42), ("Float", 42.5)])
@pytest.mark.parametrize("scale", [0.0, 1.25])
def test_run_on_database_without_python_unpickling(
    monkeypatch, distribution, support, value, scale
):
    kwargs = parameters(distribution, support, scale)
    payload = plugin(pickle.dumps(kwargs))
    query, connection, calls = query_and_connection(monkeypatch, [payload], [value])

    def forbidden(*args, **kwargs):
        pytest.fail("run_on_database must not load pickled Python objects")

    monkeypatch.setattr(pickle, "loads", forbidden)
    [result] = run_on_database(query, connection, "synthetic")
    assert len(calls) == 1
    assert type(result) is type(value)
    assert math.isfinite(result)
    if scale == 0:
        # A deterministic compatibility control, not a privacy-budget claim.
        assert result == value


@pytest.mark.parametrize(
    "kwargs",
    [
        None,
        [],
        {},
        {"distribution": "Laplace", "support": "Integer"},
        parameters(distribution="Unknown"),
        parameters(support="Unknown"),
        parameters(scale="1.0"),
        parameters(scale=True),
        parameters(scale=float("nan")),
        parameters(scale=float("inf")),
        parameters(scale=-1.0),
    ],
    ids=[
        "none",
        "list",
        "empty",
        "missing-scale",
        "distribution",
        "support",
        "text-scale",
        "bool-scale",
        "nan-scale",
        "inf-scale",
        "negative-scale",
    ],
)
def test_run_on_database_rejects_invalid_parameters_before_sql(monkeypatch, kwargs):
    valid = plugin(pickle.dumps(parameters()))
    invalid = plugin(pickle.dumps(kwargs))
    # Check the whole list before executing, even when the first plugin is valid.
    query, connection, calls = query_and_connection(monkeypatch, [valid, invalid])
    with pytest.raises(dp.OpenDPException):
        run_on_database(query, connection, "synthetic")
    assert not calls


@pytest.mark.parametrize(
    "payload",
    [
        b"",
        b"not a pickle",
        pickle.dumps(parameters())[:-1],
        pickle.dumps(parameters()) + b"trailing data",
        # A benign Python global is not a NoisePlugin parameter dictionary.
        pickle.dumps(str),
    ],
    ids=["empty", "invalid", "truncated", "trailing", "python-global"],
)
def test_run_on_database_rejects_invalid_encoding_before_sql(monkeypatch, payload):
    query, connection, calls = query_and_connection(monkeypatch, [plugin(payload)])
    with pytest.raises(dp.OpenDPException):
        run_on_database(query, connection, "synthetic")
    assert not calls


def test_run_on_database_rejects_other_plugins_before_sql(monkeypatch):
    payload = plugin(pickle.dumps(parameters()), symbol="index_candidates_plugin")
    query, connection, calls = query_and_connection(monkeypatch, [payload])
    with pytest.raises(dp.OpenDPException, match="Only OpenDP noise plugins"):
        run_on_database(query, connection, "synthetic")
    assert not calls


def test_run_on_database_preserves_scalar_parameter_alignment_error(monkeypatch):
    query, connection, calls = query_and_connection(
        monkeypatch, [plugin(pickle.dumps(parameters()))], [1, 2]
    )
    with pytest.raises(dp.OpenDPException, match="not 1-1"):
        run_on_database(query, connection, "synthetic")
    assert len(calls) == 1


def test_run_on_database_preserves_scalar_order(monkeypatch):
    plugins = [
        plugin(pickle.dumps(parameters("Laplace", "Integer", 0.0))),
        plugin(pickle.dumps(parameters("Gaussian", "Float", 0.0))),
    ]
    query, connection, calls = query_and_connection(monkeypatch, plugins, [7, 12.5])
    assert run_on_database(query, connection, "synthetic") == [7, 12.5]
    assert len(calls) == 1


def test_run_on_database_with_older_translator(monkeypatch):
    query, connection, calls = query_and_connection(
        monkeypatch, [plugin(pickle.dumps(parameters(scale=0.0)))], [42]
    )
    monkeypatch.delattr(polars_to_ibis, "split_polars_on_ffi_plan")
    assert run_on_database(query, connection, "synthetic") == [42]
    assert len(calls) == 1
