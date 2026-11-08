"""Database execution, Arrow IPC, and the interactive OnceFrame postprocessor."""

import io
from pathlib import Path
import stat
from types import SimpleNamespace

import pytest

import opendp.prelude as dp
from opendp.extras import ibis as dp_ibis
from opendp.extras.polars import _domain_from_dtype

pl = pytest.importorskip("polars")
assert_frame_equal = pytest.importorskip("polars.testing").assert_frame_equal
pa = pytest.importorskip("pyarrow")
ibis = pytest.importorskip("ibis")
pytest.importorskip("polars_to_ibis")

pytestmark = [
    pytest.mark.filterwarnings("ignore:fetch_arrow_table.*:DeprecationWarning"),
    pytest.mark.filterwarnings("ignore:The 'future.infer_string' option is deprecated"),
    pytest.mark.filterwarnings("ignore:.*to_pyarrow_batches.*:FutureWarning"),
]


def measurement(source, plan, *, public_keys=True):
    columns = []
    for name, dtype in source.schema.items():
        domain = _domain_from_dtype(dtype)
        if source[name].null_count():
            domain = dp.option_domain(domain)
        columns.append(dp.series_domain(name, domain))
    domain = dp.with_margin(
        dp.lazyframe_domain(columns), dp.polars.Margin(max_length=10)
    )
    if "key" in source.columns:
        domain = dp.with_margin(
            domain,
            dp.polars.Margin(
                by=[pl.col("key")],
                max_length=10,
                max_groups=4,
                invariant="keys" if public_keys else None,
            ),
        )
    return dp.m.make_private_lazyframe(
        domain,
        dp.symmetric_distance(),
        dp.approximate(dp.max_divergence()),
        plan,
        global_scale=1.0,
    )


def quantile(candidates=(1, 3, 6)):
    return pl.col("value").dp.quantile(0.5, candidates, scale=0.0).alias("q")


@pytest.fixture
def connection():
    pytest.importorskip("duckdb")
    connection = ibis.duckdb.connect()
    yield connection
    connection.disconnect()


@pytest.fixture
def execution(monkeypatch):
    """Observe only the native boundary and IPC schemas, never execute a head in Polars."""
    from opendp import _data

    calls: list[tuple[Path, list, list]] = []
    directories: list[Path] = []
    split, collect = _data.onceframe_split, pl.LazyFrame.collect

    def record_split(onceframe, work_dir):
        directory = Path(work_dir)
        directories.append(directory)
        assert stat.S_IMODE(directory.stat().st_mode) == 0o700
        assert not list(directory.iterdir())
        plans = split(onceframe, work_dir)
        calls.append((directory, plans, []))
        return plans

    def collect_continuation(plan, *args, **kwargs):
        if not calls or not calls[-1][0].exists():
            return collect(plan, *args, **kwargs)
        directory, plans, schemas = calls[-1]
        assert plan is plans[-1]
        assert {path.name for path in directory.iterdir()} == {
            f"{i}.arrow" for i in range(len(plans) - 1)
        }
        for i in range(len(plans) - 1):
            with pa.memory_map(str(directory / f"{i}.arrow"), "r") as file:
                schemas.append(pa.ipc.open_file(file).schema)
        return collect(plan, *args, **kwargs)

    monkeypatch.setattr(_data, "onceframe_split", record_split)
    monkeypatch.setattr(pl.LazyFrame, "collect", collect_continuation)
    yield calls
    assert all(not directory.exists() for directory in directories)


def assert_consumed(onceframe):
    with pytest.raises(dp.OpenDPException, match="exhaust"):
        onceframe.collect()


def test_grouped_quantiles_and_mixed_statistics(connection, execution, monkeypatch):
    source = pl.DataFrame(
        {"key": [0, 0, 1, None], "value": [3, 3, 6, 1]},
        schema={"key": pl.Int32, "value": pl.Int32},
    )
    connection.create_table("source", source)
    plan = (
        source.lazy()
        .group_by("key")
        .agg(
            quantile(),
            quantile((1, 3, 6, 8)).alias("other"),
            pl.col("value").dp.sum((0, 6), scale=0.0).alias("sum"),
            pl.col("value").dp.mean((0, 6), scale=0.0).alias("mean"),
        )
    )
    onceframe = measurement(source, plan)(source.lazy())
    samples = []
    sample = pl.DataFrame.sample

    def shuffled(frame, **kwargs):
        samples.append((frame.height, kwargs))
        return sample(frame, **kwargs)

    monkeypatch.setattr(pl.DataFrame, "sample", shuffled)
    executor = dp_ibis.make_executor(connection, "source")
    assert isinstance(executor, dp.Function)
    result = executor(onceframe)
    expected = pl.DataFrame(
        {
            "key": [None, 0, 1],
            "q": [1, 3, 6],
            "other": [1, 3, 6],
            "sum": [1, 6, 6],
            "mean": [1.0, 3.0, 6.0],
        },
        schema={
            "key": pl.Int32,
            "q": pl.Int64,
            "other": pl.Int64,
            "sum": pl.Int32,
            "mean": pl.Float64,
        },
    )
    assert_frame_equal(result.sort("key"), expected, check_exact=True)
    assert (
        len(execution[0][1]) == 4
    )  # two independent histograms, scalar head, continuation
    assert len(execution[0][2]) == 3
    assert samples == [(3, {"fraction": 1.0, "shuffle": True})]
    assert_consumed(onceframe)


@pytest.mark.parametrize("empty", [False, True])
def test_arrow_schema_and_empty_results(empty, connection, execution):
    source = pl.DataFrame(
        {"key": [2**53 + 1, 2**64 - 1], "value": [1.0, 3.0]},
        schema={"key": pl.UInt64, "value": pl.Float32},
    )
    if empty:
        source = source.clear()
    connection.create_table("source", source)
    candidates = pl.Series([1.0, 3.0], dtype=pl.Float32)
    plan = (
        source.lazy()
        .group_by("key")
        .agg(
            quantile(candidates),
            pl.col("value")
            .fill_nan(pl.lit(0.0, dtype=pl.Float32))
            .clip(pl.lit(0.0, dtype=pl.Float32), pl.lit(3.0, dtype=pl.Float32))
            .sum()
            .dp.noise(0.0)
            .alias("sum"),
        )
    )
    onceframe = measurement(source, plan)(source.lazy())
    # A capability-only proxy deliberately has no backend name or transaction API.
    backend = SimpleNamespace(
        has_operation=connection.has_operation,
        to_pyarrow_batches=connection.to_pyarrow_batches,
    )

    result = dp_ibis.make_executor(backend, "source")(onceframe)
    expected = source.rename({"value": "q"}).with_columns(pl.col("q").alias("sum"))
    assert_frame_equal(result.sort("key"), expected, check_exact=True)
    schema = execution[0][2][0]
    assert schema.field("key").type == pa.uint64()
    assert all(field.type == pa.uint64() for field in schema)
    assert execution[0][2][1].types == [pa.uint64(), pa.float32()]
    assert result.schema == {"key": pl.UInt64, "q": pl.Float32, "sum": pl.Float32}


def test_ffi_ordered_vector_and_serialization(tmp_path):
    from opendp._data import onceframe_split

    source = pl.DataFrame({"value": [3, 3]}, schema={"value": pl.Int32})
    plan = source.lazy().select(
        quantile(), pl.col("value").dp.sum((0, 6), scale=0.0).alias("sum")
    )
    onceframe = measurement(source, plan)(source.lazy())
    directory = tmp_path / "absent"
    plans = onceframe_split(onceframe.queryable, str(directory))
    assert len(plans) == 3 and all(isinstance(plan, pl.LazyFrame) for plan in plans)
    restored = [
        pl.LazyFrame.deserialize(io.BytesIO(plan.serialize())) for plan in plans
    ]
    assert restored[0].collect_schema().dtypes() == [pl.UInt64, pl.UInt64]
    assert restored[1].collect_schema().dtypes() == [pl.Int32]
    assert not directory.exists()
    assert_consumed(onceframe)


def test_additional_public_source_is_rejected(connection, execution, monkeypatch):
    source = pl.DataFrame(
        {"key": [0], "value": [3]}, schema={"key": pl.Int32, "value": pl.Int32}
    )
    plan = (
        source.lazy()
        .group_by("key")
        .agg(quantile())
        .join(
            pl.LazyFrame({"key": [0, 1]}).cast({"key": pl.Int32}), on="key", how="right"
        )
    )
    onceframe = measurement(source, plan, public_keys=False)(source.lazy())

    def forbid_execution(*args, **kwargs):
        raise AssertionError("an additional source must fail before database execution")

    monkeypatch.setattr(connection, "to_pyarrow_batches", forbid_execution)
    with pytest.raises(
        dp.OpenDPException, match="additional.*source|in-memory|multiple.*source"
    ):
        dp_ibis.make_executor(connection, "source")(onceframe)
    assert_consumed(onceframe)


@pytest.mark.parametrize("failure", ["query", "batches", "continuation"])
def test_failure_cleanup_and_consumption(failure, connection, execution, monkeypatch):
    source = pl.DataFrame({"value": [3]}, schema={"value": pl.Int32})
    connection.create_table("source", source)
    onceframe = measurement(source, source.lazy().select(quantile()))(source.lazy())
    faults = []
    if failure == "batches":
        stream = connection.to_pyarrow_batches

        def broken_stream(expression, **kwargs):
            batches = stream(expression, **kwargs)

            def fail_after_batches():
                with batches:
                    yield from batches
                    faults.append("batches")
                    raise RuntimeError("private intermediate: sentinel batch value")

            return pa.RecordBatchReader.from_batches(
                batches.schema, fail_after_batches()
            )

        monkeypatch.setattr(connection, "to_pyarrow_batches", broken_stream)
    elif failure == "continuation":

        def broken_collect(*args, **kwargs):
            faults.append("continuation")
            raise RuntimeError("private intermediate: sentinel continuation value")

        monkeypatch.setattr(pl.LazyFrame, "collect", broken_collect)
    table = "missing" if failure == "query" else "source"
    with pytest.raises(dp.OpenDPException, match="Database execution failed") as error:
        dp_ibis.make_executor(connection, table)(onceframe)
    assert "sentinel" not in str(error.value)
    assert "missing" not in str(error.value)
    assert faults == ([] if failure == "query" else [failure])
    assert_consumed(onceframe)


@pytest.mark.parametrize("backend", ["sqlite", "duckdb"])
def test_released_query_and_function_composition(backend):
    if backend == "duckdb":
        pytest.importorskip("duckdb")
    connection = getattr(ibis, backend).connect()
    try:
        source = pl.DataFrame({"value": [1, 2, 3]})
        connection.create_table("source", source)
        context = dp.Context.compositor(
            data=dp_ibis.scan_database(connection, "source"),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
            margins=[dp.polars.Margin(max_length=10)],
        )
        query = context.query().select(dp.len(signed=False))
        executor = dp_ibis.make_executor(connection, "source")
        onceframe = query.release()
        result = executor(onceframe)
        assert result.shape == (1, 1) and result.schema == {"len": pl.UInt32}
        assert_consumed(onceframe)
        plan = source.lazy().select(pl.len().dp.noise(1.0).alias("count"))
        private = measurement(source, plan)
        composed = private >> executor
        assert composed.map(1) == private.map(1)
        assert composed(source.lazy()).schema == {"count": pl.UInt32}
    finally:
        connection.disconnect()


def test_executor_requires_trusted_execution():
    dp.disable_features("contrib")
    try:
        with pytest.raises(dp.OpenDPException, match="contrib"):
            dp_ibis.make_executor(None, "source")
    finally:
        dp.enable_features("contrib")
