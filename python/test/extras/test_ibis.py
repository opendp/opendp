"""The native splitter, real SQL heads, and native randomized continuation."""

import io
import json
from pathlib import Path
import pickle
import re
import stat
import warnings

import pytest

import opendp.prelude as dp
from opendp.extras import ibis as dp_ibis
from opendp.extras.polars import _domain_from_dtype, _get_opendp_polars_lib_path

pl = pytest.importorskip("polars")
pl_testing = pytest.importorskip("polars.testing")
ibis = pytest.importorskip("ibis")
p2i = pytest.importorskip("polars_to_ibis")

pytestmark = [
    pytest.mark.filterwarnings("ignore:fetch_arrow_table.*:DeprecationWarning"),
    pytest.mark.filterwarnings("ignore:The 'future.infer_string' option is deprecated"),
    pytest.mark.filterwarnings(
        "ignore:'json' (de)?serialization format of LazyFrame is deprecated:UserWarning"
    ),
    pytest.mark.filterwarnings("ignore:.*to_polars.*:FutureWarning"),
]


def wire(frame):
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", UserWarning)
        return json.loads(frame.serialize(format="json"))


def restore(node):
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", UserWarning)
        return pl.LazyFrame.deserialize(io.StringIO(json.dumps(node)), format="json")


def walk(node):
    yield node
    if isinstance(node, dict):
        for child in node.values():
            yield from walk(child)
    elif isinstance(node, list):
        for child in node:
            yield from walk(child)


def sink_inputs(sinks):
    return [node["Sink"] for node in wire(sinks)["SinkMultiple"]["inputs"]]


def assert_equal(actual, expected):
    assert actual.schema == expected.schema
    columns = actual.columns
    pl_testing.assert_frame_equal(
        actual.sort(columns), expected.sort(columns), check_exact=True
    )


def resolve(source, query, *, public_keys=True, threshold=None):
    domains = []
    for name, dtype in source.schema.items():
        domain = _domain_from_dtype(dtype)
        if source[name].null_count():
            domain = dp.option_domain(domain)
        domains.append(dp.series_domain(name, domain))
    domain = dp.lazyframe_domain(domains)
    domain = dp.with_margin(domain, dp.polars.Margin(max_length=10, max_groups=1))
    if "key" in source.columns:
        domain = dp.with_margin(
            domain,
            dp.polars.Margin(
                by=[pl.col("key")],
                max_length=10,
                max_groups=2,
                invariant="keys" if public_keys else None,
            ),
        )
    return dp.m.make_private_lazyframe(
        domain,
        dp.symmetric_distance(),
        dp.approximate(dp.max_divergence()),
        query,
        global_scale=1.0,
        threshold=threshold,
    )(source.lazy())


def plugin(name, *args):
    from polars.plugins import register_plugin_function

    return register_plugin_function(
        plugin_path=_get_opendp_polars_lib_path(),
        function_name=name,
        args=args,
        returns_scalar=True,
        changes_length=True,
    )


def quantile(value=None, candidates=None):
    value = pl.col("value") if value is None else value
    candidates = [1, 3, 6] if candidates is None else candidates
    return value.dp.quantile(0.5, candidates, scale=0.0).alias("q")


def sum_expr():
    return pl.col("value").dp.sum((0, 5), scale=0.0).alias("sum")


def mean_expr():
    return pl.col("value").dp.mean((0, 5), scale=0.0).alias("mean")


def example():
    return pl.DataFrame(
        {"key": [0, 0, 1], "value": [1, 6, 3]},
        schema={"key": pl.Int32, "value": pl.Int32},
    )


@pytest.fixture
def connection():
    pytest.importorskip("duckdb")
    backend = ibis.duckdb.connect()
    yield backend
    backend.disconnect()


@pytest.fixture
def audit(monkeypatch):
    """Observe production execution, never replace the real converter or SQL."""
    captured: dict[str, list] = {"heads": [], "sql": [], "directories": []}
    convert = p2i.convert_polars_to_ibis
    materialize = dp_ibis._materialize_sinks

    def checked_convert(head, table_name, backend):
        query = convert(head, table_name=table_name, backend=backend)
        sql = str(query.to_sql())
        captured["sql"].append(sql)
        assert not re.search(r"\b(?:ARRAY_AGG|LIST|UNNEST)\s*\(", sql, re.IGNORECASE), (
            sql
        )
        assert not re.search(
            r"\b(?:discrete_quantile_score_plugin|discrete_quantile_histogram_plugin|"
            r"histogram_quantile_score_plugin|noise_plugin|noisy_max_plugin|"
            r"index_candidates_plugin)\s*\(",
            sql,
            re.IGNORECASE,
        ), sql
        plan = wire(head)
        if "GroupBy" in plan and "HStack" in plan["GroupBy"]["input"]:
            assert re.search(r"\bCOUNT\s*\(\s*\*\s*\)", sql, re.IGNORECASE), sql
            assert len(re.findall(r"\bGROUP\s+BY\b", sql, re.IGNORECASE)) == 1, sql
        assert_equal(backend.to_polars(query, limit=None), head.collect())
        return query

    def checked_materialize(sinks, backend, table_name):
        nodes = sink_inputs(sinks)
        paths = [Path(node["payload"]["File"]["target"]["Local"]) for node in nodes]
        assert len(set(paths)) == len(paths)
        directory = paths[0].parent
        captured["directories"].append(directory)
        assert stat.S_IMODE(directory.stat().st_mode) == 0o700
        assert all(path.parent == directory and not path.exists() for path in paths)
        references = [restore(node["input"]).collect() for node in nodes]
        captured["heads"].extend(references)
        materialize(sinks, backend, table_name)
        for path, reference in zip(paths, references):
            assert_equal(pl.read_ipc(path), reference)

    monkeypatch.setattr(p2i, "convert_polars_to_ibis", checked_convert)
    monkeypatch.setattr(dp_ibis, "_materialize_sinks", checked_materialize)
    yield captured
    assert all(not directory.exists() for directory in captured["directories"])


CASES = [
    "mixed",
    "difference",
    "median",
    "independent",
    "empty-global",
    "empty-grouped",
    "public",
    "public-empty",
    "null-keys",
    "null-nan",
    "u64",
    "i64",
    "threshold",
    "collision",
    "overflow-global",
    "overflow-grouped",
    "f32-weak-global",
    "f32-weak-grouped",
    "f32-typed-global",
    "f32-typed-grouped",
    "noise-filter",
    "sum",
]


def make_case(case):
    source = example()
    public_keys, threshold = True, None
    expressions = [quantile(), sum_expr(), mean_expr()]
    grouped = True
    if case.startswith("empty") or case == "public-empty":
        source = source.clear()
    if case.startswith("empty"):
        expressions = [quantile(candidates=[3]), sum_expr()]
        grouped = case == "empty-grouped"
    elif case.startswith("public"):
        expressions = [quantile(candidates=[3]), sum_expr(), mean_expr()]
    elif case == "null-keys":
        source = source.with_columns(pl.Series("key", [None, None, 1], dtype=pl.Int32))
    elif case == "null-nan":
        source = pl.DataFrame(
            {"key": [0, 0, 0, 1, 1], "value": [1.0, None, float("nan"), 3.0, 6.0]},
            schema={"key": pl.Int32, "value": pl.Float64},
        )
        expressions = [quantile(candidates=[3.0]), sum_expr(), mean_expr()]
    elif case in {"u64", "i64"}:
        dtype = pl.UInt64 if case == "u64" else pl.Int64
        values = (
            [0, 2**53, 2**53 + 1, 2**64 - 2, 2**64 - 1]
            if case == "u64"
            else [-(2**63), -(2**63) + 1, 0, 2**63 - 2, 2**63 - 1]
        )
        candidates = pl.Series(
            [2**53, 2**53 + 1, 2**64 - 2]
            if case == "u64"
            else [-(2**63), 0, 2**63 - 1],
            dtype=dtype,
        )
        source = pl.DataFrame(
            {
                "key": pl.Series([0] * 5, dtype=pl.Int32),
                "value": pl.Series(values, dtype=dtype),
            }
        )
        scores = plugin(
            "discrete_quantile_score", pl.col("value"), pl.lit(0.5), pl.lit(candidates)
        )
        index = plugin("noisy_max", scores, pl.lit(True), pl.lit(0.0))
        expressions = [plugin("index_candidates", index, pl.lit(candidates)).alias("q")]
        grouped = False
    elif case == "threshold":
        expressions = [quantile(), sum_expr(), pl.len().dp.noise(0.0).alias("count")]
        public_keys, threshold = False, 1
    elif case == "collision":
        expressions = [
            quantile().alias("__opendp_split_0"),
            sum_expr().alias("__opendp_split_1"),
        ]
    elif case.startswith("overflow"):
        source = pl.DataFrame(
            {"key": [0, 0, 0, 1, 1], "value": [128, -129, 0, 300, -300]},
            schema={"key": pl.Int32, "value": pl.Int32},
        )
        expressions = [quantile(pl.col("value").cast(pl.Int8).fill_null(0), [-1, 0, 1])]
        grouped = case.endswith("grouped")
    elif case.startswith("f32"):
        source = pl.DataFrame(
            {
                "key": pl.Series([0, 0, 0, 1, 1], dtype=pl.Int32),
                "value": pl.Series(
                    [None, 0.1, float("nan"), 0.1, 0.1], dtype=pl.Float32
                ),
            }
        )
        if "typed" in case:
            value = (
                pl.col("value").fill_null(pl.lit(0.1, dtype=pl.Float32))
                - pl.lit(0.1, dtype=pl.Float32)
            ).fill_nan(pl.lit(0.0, dtype=pl.Float32))
            candidates = [-1e-10, 0.0, 1e-10]
        else:
            value = pl.col("value").fill_null(0.1).fill_nan(0.1)
            candidates = [0.0, 0.1, 1.0]
        expressions = [quantile(value, candidates)]
        grouped = case.endswith("grouped")
    elif case == "difference":
        expressions, grouped = [(quantile() - mean_expr()).alias("difference")], False
    elif case == "median":
        expressions, grouped = (
            [pl.col("value").dp.median([1, 3, 6], scale=0.0).alias("median")],
            False,
        )
    elif case == "independent":
        expressions.append(quantile().alias("independent"))
    elif case == "noise-filter":
        total = pl.col("value").clip(0, 10).sum()
        expressions = [
            total.dp.noise(0.0).alias("sum"),
            (total.dp.noise(0.0) / pl.len().dp.noise(0.0)).alias("mean"),
        ]
    elif case == "sum":
        expressions, grouped = [sum_expr()], False
    query = (
        source.lazy().group_by("key").agg(expressions)
        if grouped
        else source.lazy().select(expressions)
    )
    if case.startswith("public"):
        public = pl.DataFrame({"key": [2, 0, 2, 1]}, schema={"key": pl.Int32}).lazy()
        query = query.join(public, on="key", how="right")
        public_keys = False
    elif case == "collision":
        public = pl.DataFrame(
            {"key": [0, 1], "__opendp_split_2": [4, 5]},
            schema={"key": pl.Int32, "__opendp_split_2": pl.Int32},
        ).lazy()
        query = query.join(public, on="key", how="right")
        public_keys = False
    elif case == "noise-filter":
        query = query.filter(pl.col("sum") > 4)
    return source, resolve(source, query, public_keys=public_keys, threshold=threshold)


@pytest.mark.parametrize("case", CASES)
def test_native_heads_through_real_sql(case, connection, audit):
    source, onceframe = make_case(case)
    native = onceframe.lazy()
    plan = wire(native)
    if case.startswith("overflow"):
        casts = [
            node["Cast"]
            for node in walk(plan)
            if isinstance(node, dict) and "Cast" in node
        ]
        assert any(
            cast["options"] == "NonStrict" and "Int8" in str(cast["dtype"])
            for cast in casts
        )
    if case.startswith("f32"):
        scorers = [
            node["FfiPlugin"]
            for node in walk(plan)
            if isinstance(node, dict)
            and "FfiPlugin" in node
            and node["FfiPlugin"]["symbol"] == "discrete_quantile_score_plugin"
        ]
        assert len(scorers) == 1
        # Only inspect trusted native arguments; never recreate a scorer or noise.
        candidates = pickle.loads(bytes(scorers[0]["kwargs"]))["candidates"]
        assert (
            pl.DataFrame.deserialize(io.BytesIO(candidates)).to_series().dtype
            == pl.Float32
        )
    expected = native.collect()
    connection.create_table("source", source, overwrite=True)
    result = dp_ibis._run_on_database(onceframe, connection, "source")
    assert isinstance(result, pl.DataFrame)
    assert_equal(result, expected)
    assert audit["sql"]
    expected_heads = (
        3
        if case == "independent"
        else 2
        if case
        in {
            "mixed",
            "difference",
            "empty-global",
            "empty-grouped",
            "null-keys",
            "null-nan",
            "threshold",
        }
        else 1
    )
    if case.startswith("public") or case == "collision":
        expected_heads = 3
    assert len(audit["heads"]) == expected_heads
    ordered = result.sort("key") if "key" in result.columns else result
    if case in {"mixed", "independent"}:
        expected_dict = {"key": [0, 1], "q": [3, 3], "sum": [6, 3], "mean": [3.0, 3.0]}
        if case == "independent":
            expected_dict["independent"] = [3, 3]
        assert ordered.to_dict(as_series=False) == expected_dict
    if case == "independent":
        assert (
            sum(
                node.get("symbol") == "noisy_max_plugin"
                for node in walk(plan)
                if isinstance(node, dict)
            )
            == 2
        )
    if case in {"median", "difference", "empty-global", "noise-filter", "sum"}:
        assert (
            ordered.to_dict(as_series=False)
            == {
                "median": {"median": [3]},
                "difference": {"difference": [0.0]},
                "empty-global": {"q": [3], "sum": [0]},
                "noise-filter": {"key": [0], "sum": [7], "mean": [3.5]},
                "sum": {"sum": [9]},
            }[case]
        )
    if case.startswith("public"):
        assert ordered["key"].to_list() == [0, 1, 2]
        assert ordered["q"].to_list() == [3, 3, 3]
        assert ordered["sum"].to_list() == (
            [0, 0, 0] if case == "public-empty" else [6, 3, 0]
        )
        assert ordered["mean"].is_nan().to_list() == (
            [True] * 3 if case == "public-empty" else [False, False, True]
        )


def assert_consumed(onceframe, path):
    from opendp._data import onceframe_split

    for operation in [
        onceframe.collect,
        onceframe.lazy,
        lambda: onceframe_split(onceframe.queryable, str(path)),
    ]:
        with pytest.raises(dp.OpenDPException, match="exhaust|collect|consum"):
            operation()


@pytest.mark.parametrize("case", ["sum", "median", "independent"])
def test_ffi_serialization_before_ipc(case, tmp_path):
    from opendp._data import onceframe_split

    _, onceframe = make_case(case)
    expected = onceframe.lazy().collect()
    directory = tmp_path / "not-created"
    sinks, continuation = onceframe_split(onceframe.queryable, str(directory))
    assert isinstance(sinks, pl.LazyFrame) and isinstance(continuation, pl.LazyFrame)
    assert not directory.exists()
    restored_sinks = pl.LazyFrame.deserialize(io.BytesIO(sinks.serialize()))
    restored_continuation = pl.LazyFrame.deserialize(
        io.BytesIO(continuation.serialize())
    )
    assert sink_inputs(restored_sinks) == sink_inputs(sinks)
    nodes = sink_inputs(restored_sinks)
    paths = [Path(node["payload"]["File"]["target"]["Local"]) for node in nodes]
    assert all(path.parent == directory and not path.exists() for path in paths)
    assert all(str(path) in json.dumps(wire(restored_continuation)) for path in paths)
    assert not directory.exists()
    assert_consumed(onceframe, directory)
    directory.mkdir()
    for node in nodes:
        restore({"Sink": node}).collect()
    assert all(path.exists() for path in paths)
    assert_equal(restored_continuation.collect(), expected)


def test_ffi_split_failure_still_consumes(tmp_path):
    from opendp._data import onceframe_split

    source = example()
    query = source.lazy().select(quantile(pl.lit(3, dtype=pl.Int32)))
    onceframe = resolve(source, query)
    directory = tmp_path / "absent"
    with pytest.raises(dp.OpenDPException, match="literal"):
        onceframe_split(onceframe.queryable, str(directory))
    assert not directory.exists()
    assert_consumed(onceframe, directory)


@pytest.mark.parametrize("backend", ["sqlite", "duckdb"])
def test_public_run_on_database(backend):
    if backend == "duckdb":
        pytest.importorskip("duckdb")
    connection = dp_ibis.get_connection(backend)
    try:
        source = example()
        connection.create_table("source", source, overwrite=True)
        context = dp.Context.compositor(
            data=dp_ibis.scan_database(connection, "source"),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
            margins=[dp.polars.Margin(max_length=10)],
        )
        query = context.query().select(dp.len(signed=False))
        result = dp_ibis.run_on_database(query, connection, "source")
        assert isinstance(result, pl.DataFrame)
        assert result.shape == (1, 1)
        assert result.schema == {"len": pl.UInt32}
    finally:
        connection.disconnect()


@pytest.mark.parametrize("failure", ["sql", "continuation"])
def test_cleanup_and_rollback_on_failure(failure, connection, monkeypatch):
    source, onceframe = make_case("mixed")
    connection.create_table("source", source, overwrite=True)
    directories, statements = [], []
    materialize, execute, raw_sql = (
        dp_ibis._materialize_sinks,
        connection.to_polars,
        connection.raw_sql,
    )

    def record_sql(statement, *args, **kwargs):
        statements.append(statement)
        return raw_sql(statement, *args, **kwargs)

    def fail_sql(*args, **kwargs):
        execute(*args, **kwargs)
        raise RuntimeError("after real SQL")

    def fail_materialize(sinks, backend, table_name):
        paths = [
            Path(node["payload"]["File"]["target"]["Local"])
            for node in sink_inputs(sinks)
        ]
        directories.append(paths[0].parent)
        assert stat.S_IMODE(paths[0].parent.stat().st_mode) == 0o700
        raw_sql("INSERT INTO source VALUES (9, 9)")
        materialize(sinks, backend, table_name)
        for path in paths:
            path.unlink()

    monkeypatch.setattr(connection, "raw_sql", record_sql)
    monkeypatch.setattr(dp_ibis, "_materialize_sinks", fail_materialize)
    if failure == "sql":
        monkeypatch.setattr(connection, "to_polars", fail_sql)
    with pytest.raises(
        Exception, match="after real SQL|No such file|not found|cannot find"
    ):
        dp_ibis._run_on_database(onceframe, connection, "source")
    assert statements == ["BEGIN TRANSACTION", "ROLLBACK"]
    assert directories and all(not directory.exists() for directory in directories)
    assert_consumed(onceframe, directories[0])
    assert execute(connection.table("source"), limit=None).height == source.height


def test_failed_begin_consumes_without_rollback(connection, monkeypatch):
    _, onceframe = make_case("sum")
    statements = []

    def fail_begin(statement):
        statements.append(statement)
        raise RuntimeError("begin failed")

    monkeypatch.setattr(connection, "raw_sql", fail_begin)
    with pytest.raises(RuntimeError, match="begin failed"):
        dp_ibis._run_on_database(onceframe, connection, "source")
    assert statements == ["BEGIN TRANSACTION"]
    assert_consumed(onceframe, Path("unused"))


def test_unsupported_backend_fails_closed():
    from types import SimpleNamespace

    _, onceframe = make_case("sum")
    with pytest.raises(NotImplementedError, match="snapshot"):
        dp_ibis._run_on_database(
            onceframe, SimpleNamespace(name="unsupported"), "source"
        )
    # Rejection occurs before FFI consumption.
    assert onceframe.collect().item() == 9


def test_heads_share_a_database_snapshot(tmp_path, monkeypatch):
    pytest.importorskip("duckdb")
    database = str(tmp_path / "snapshot.duckdb")
    connection = ibis.duckdb.connect(database)
    writer = ibis.duckdb.connect(database)
    try:
        source, onceframe = make_case("mixed")
        expected = onceframe.lazy().collect()
        connection.create_table("source", source, overwrite=True)
        execute = connection.to_polars
        calls = []

        def update_after_first_head(*args, **kwargs):
            result = execute(*args, **kwargs)
            calls.append(result)
            if len(calls) == 1:
                writer.raw_sql("INSERT INTO source VALUES (0, 5)")
            return result

        monkeypatch.setattr(connection, "to_polars", update_after_first_head)
        result = dp_ibis._run_on_database(onceframe, connection, "source")
        assert len(calls) == 2
        assert_equal(result, expected)
        assert writer.to_polars(writer.table("source"), limit=None).height == 4
    finally:
        writer.disconnect()
        connection.disconnect()


def test_complete_final_result_is_shuffled(connection, monkeypatch):
    source, onceframe = make_case("mixed")
    expected = onceframe.lazy().collect()
    connection.create_table("source", source, overwrite=True)
    sample = pl.DataFrame.sample
    calls = []

    def record(frame, *args, **kwargs):
        calls.append((frame.clone(), args, kwargs))
        return sample(frame, *args, **kwargs)

    monkeypatch.setattr(pl.DataFrame, "sample", record)
    result = dp_ibis._run_on_database(onceframe, connection, "source")
    assert len(calls) == 1
    frame, args, kwargs = calls[0]
    assert_equal(frame, expected)
    assert args == ()
    assert kwargs == {"fraction": 1.0, "shuffle": True}
    assert_equal(result, expected)
