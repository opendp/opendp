"""Ibis Context integration, trusted collection, and the native split boundary."""

from concurrent.futures import ThreadPoolExecutor
from copy import copy
import gc
import io
from pathlib import Path
import stat
from threading import Event
import weakref

import pytest

import opendp.prelude as dp
from opendp.extras import ibis as dp_ibis
from opendp.extras.polars import (
    LazyFrameQuery,
    OnceFrame,
    _lazyframe_domain_from_schema,
)

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


def compositor(
    connection, source, *, contributions=0, releases=1, public_keys=True, ordered=False
):
    """Zero adjacency makes transport assertions exact; private tests use one."""
    connection.create_table("source", source, overwrite=True)
    margins = [dp.polars.Margin(max_length=10)]
    if "key" in source.columns:
        margins.append(
            dp.polars.Margin(
                by=[pl.col("key")],
                max_length=10,
                max_groups=4,
                invariant="keys" if public_keys else None,
            )
        )
    return dp.Context.compositor(
        data=dp_ibis.scan_database(connection, "source"),
        privacy_unit=dp.unit_of(contributions=contributions, ordered=ordered),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-6),
        split_evenly_over=releases,
        margins=margins,
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
    """Observe native/IPC boundaries; fail if a deterministic head runs in Polars."""
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
    with pytest.raises(RuntimeError, match="exhaust"):
        onceframe.collect()


def test_grouped_quantiles_and_mixed_statistics(connection, execution, monkeypatch):
    source = pl.DataFrame(
        {"key": [0, 0, 1, None], "value": [3, 3, 6, 1]},
        schema={"key": pl.Int32, "value": pl.Int32},
    )
    context = compositor(connection, source)
    onceframe = (
        context.query()
        .group_by("key")
        .agg(
            quantile(),
            quantile((1, 3, 6, 8)).alias("other"),
            pl.col("value").dp.sum((0, 6), scale=0.0).alias("sum"),
            pl.col("value").dp.mean((0, 6), scale=0.0).alias("mean"),
        )
        .release()
    )
    samples = []
    sample = pl.DataFrame.sample

    def shuffled(frame, **kwargs):
        samples.append((frame.height, kwargs))
        return sample(frame, **kwargs)

    monkeypatch.setattr(pl.DataFrame, "sample", shuffled)
    result = onceframe.collect()
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
    )  # independent histograms, scalar head, continuation
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
    # Ordered zero adjacency avoids floating-point sum permutation relaxation.
    context = compositor(connection, source, ordered=True)
    candidates = pl.Series([1.0, 3.0], dtype=pl.Float32)
    onceframe = (
        context.query()
        .group_by("key")
        .agg(
            quantile(candidates),
            pl.col("value")
            .fill_null(pl.lit(0.0, dtype=pl.Float32))
            .fill_nan(pl.lit(0.0, dtype=pl.Float32))
            .clip(pl.lit(0.0, dtype=pl.Float32), pl.lit(3.0, dtype=pl.Float32))
            .sum()
            .dp.noise(0.0)
            .alias("sum"),
        )
        .release()
    )
    result = onceframe.collect()
    expected = source.rename({"value": "q"}).with_columns(pl.col("q").alias("sum"))
    assert_frame_equal(result.sort("key"), expected, check_exact=True)
    schema = execution[0][2][0]
    assert schema.field("key").type == pa.uint64()
    assert all(field.type == pa.uint64() for field in schema)
    assert execution[0][2][1].types == [pa.uint64(), pa.float32()]
    assert result.schema == {"key": pl.UInt64, "q": pl.Float32, "sum": pl.Float32}


def test_ffi_ordered_vector_and_serialization(tmp_path):
    from opendp._data import onceframe_split

    source = pl.LazyFrame(schema={"value": pl.Int32})
    domain = dp.with_margin(
        _lazyframe_domain_from_schema(source.collect_schema()),
        dp.polars.Margin(max_length=10),
    )
    plan = source.select(
        quantile(), pl.col("value").dp.sum((0, 6), scale=0.0).alias("sum")
    )
    native = dp.m.make_private_lazyframe(
        domain, dp.symmetric_distance(), dp.max_divergence(), plan, global_scale=1.0
    )(source)
    directory = tmp_path / "absent"
    dp.disable_features("honest-but-curious")
    try:
        plans = onceframe_split(native.queryable, str(directory))
    finally:
        dp.enable_features("honest-but-curious")
    assert len(plans) == 3 and all(isinstance(plan, pl.LazyFrame) for plan in plans)
    restored = [
        pl.LazyFrame.deserialize(io.BytesIO(plan.serialize())) for plan in plans
    ]
    assert restored[0].collect_schema().dtypes() == [pl.UInt64, pl.UInt64]
    assert restored[1].collect_schema().dtypes() == [pl.Int32]
    assert not directory.exists()
    with pytest.raises(dp.OpenDPException, match="exhaust"):
        native.collect()


def test_additional_public_source_is_rejected(connection, execution, monkeypatch):
    source = pl.DataFrame(
        {"key": [0], "value": [3]}, schema={"key": pl.Int32, "value": pl.Int32}
    )
    context = compositor(connection, source, public_keys=False)
    onceframe = (
        context.query()
        .group_by("key")
        .agg(quantile())
        .join(
            pl.LazyFrame({"key": [0, 1]}).cast({"key": pl.Int32}), on="key", how="right"
        )
        .release()
    )

    def forbid_execution(*args, **kwargs):
        raise AssertionError("an additional source must fail before database execution")

    monkeypatch.setattr(connection, "to_pyarrow_batches", forbid_execution)
    with pytest.raises(
        dp.OpenDPException, match="additional.*source|in-memory|multiple.*source"
    ):
        onceframe.collect()
    assert_consumed(onceframe)


@pytest.mark.parametrize("failure", ["query", "batches", "continuation"])
def test_failure_cleanup_and_consumption(failure, connection, execution, monkeypatch):
    context = compositor(
        connection, pl.DataFrame({"value": [3]}, schema={"value": pl.Int32})
    )
    onceframe = context.query().select(quantile()).release()
    faults = []
    if failure == "query":
        connection.drop_table("source")
    elif failure == "batches":
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
    else:

        def broken_collect(*args, **kwargs):
            faults.append("continuation")
            raise RuntimeError("private intermediate: sentinel continuation value")

        monkeypatch.setattr(pl.LazyFrame, "collect", broken_collect)
    with pytest.raises(RuntimeError, match="Database execution failed") as error:
        onceframe.collect()
    assert "sentinel" not in str(error.value)
    assert error.value.__suppress_context__
    assert error.value.__context__ is None
    traceback = error.value.__traceback__
    while traceback is not None:
        assert traceback.tb_frame.f_code.co_name != "_execute_plans"
        traceback = traceback.tb_next
    assert faults == ([] if failure == "query" else [failure])
    assert_consumed(onceframe)


@pytest.mark.parametrize("backend", ["sqlite", "duckdb"])
def test_private_count_and_budget(backend, monkeypatch):
    if backend == "duckdb":
        pytest.importorskip("duckdb")
    connection = getattr(ibis, backend).connect()
    try:
        connection.create_table("records", pl.DataFrame({"value": [1, 2, 3]}))
        calls, stream = [], connection.to_pyarrow_batches

        def record(expression, **kwargs):
            calls.append(kwargs)
            return stream(expression, **kwargs)

        monkeypatch.setattr(connection, "to_pyarrow_batches", record)
        dp.disable_features("honest-but-curious")
        try:
            data = dp.ibis.scan_database(connection, "records")
            margin = dp.polars.Margin(max_length=10)
            context = dp.Context.compositor(
                data=data,
                privacy_unit=dp.unit_of(contributions=1),
                privacy_loss=dp.loss_of(epsilon=1.0),
                split_evenly_over=2,
                margins=[margin],
            )
            assert context.accountant.input_domain.type == "ExtrinsicDomain"
            assert context.query_space is not None
            domain, metric = context.query_space
            assert domain.type == "LazyFrameDomain"
            assert domain == dp.with_margin(
                _lazyframe_domain_from_schema(data.schema), margin
            )
            assert context.accountant.input_metric == metric
            query = context.query().select(dp.len(signed=False))
            resolved = query.resolve()
            native = resolved._polars_measurement
            assert resolved.map(1) == native.map(1)
            assert 0 < resolved.map(1) <= 0.5
            assert query.summarize().height == 1
            onceframe = query.release()
            assert isinstance(onceframe, OnceFrame)
            assert not hasattr(onceframe, "queryable")
            assert (
                calls == []
            )  # construction, calibration, checking, release are SQL-free
            for hbc in [False, True]:
                if hbc:
                    dp.enable_features("honest-but-curious")
                with pytest.raises(NotImplementedError, match="unsupported"):
                    onceframe.lazy()
            dp.disable_features("honest-but-curious")
            another = context.query().select(dp.len(signed=False)).release()
            assert calls == []
            assert len(context.current_privacy_loss()) == 2
            result = onceframe.collect()
            assert result.shape == (1, 1) and result.schema == {"len": pl.UInt32}
            assert_consumed(onceframe)
            assert another.collect().shape == (1, 1)
            assert calls == [{"limit": None}, {"limit": None}]
            assert len(context.current_privacy_loss()) == 2
            assert context.remaining_privacy_loss() == []
            # The Rust accountant also rejects a direct submission past its budget.
            with pytest.raises(dp.OpenDPException, match="out of queries"):
                context.queryable(resolved)
            with pytest.raises(ValueError, match="exhaust"):
                context.query()
        finally:
            dp.enable_features("honest-but-curious")
    finally:
        connection.disconnect()


def test_compositor_and_release_retain_bound_relation(connection):
    connection.create_table("source", pl.DataFrame({"value": [1, 2, 3]}))
    source = dp_ibis.scan_database(connection, "source")
    retained = weakref.ref(source.table)
    context = dp.Context.compositor(
        source,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    del source
    gc.collect()
    assert retained() is not None
    onceframe = context.query().select(dp.len(signed=False)).release()
    del context
    gc.collect()
    assert retained() is not None
    assert onceframe.collect().shape == (1, 1)
    del onceframe
    gc.collect()
    assert retained() is None


def test_aliases_and_concurrent_collection(connection, monkeypatch):
    context = compositor(connection, pl.DataFrame({"value": [3]}), contributions=1)
    onceframe = context.query().select(dp.len(signed=False)).release()
    alias = copy(onceframe)
    started, finish = Event(), Event()
    calls, stream = [], connection.to_pyarrow_batches

    def blocked(expression, **kwargs):
        calls.append(kwargs)
        started.set()
        assert finish.wait(10), "collection did not unblock"
        return stream(expression, **kwargs)

    monkeypatch.setattr(connection, "to_pyarrow_batches", blocked)
    with ThreadPoolExecutor(max_workers=2) as pool:
        first = pool.submit(onceframe.collect)
        try:
            assert started.wait(10), "collection never started"
            second = pool.submit(alias.collect)
            with pytest.raises(RuntimeError, match="exhaust"):
                second.result(timeout=10)
        finally:
            finish.set()
        assert first.result(timeout=10).shape == (1, 1)
    assert calls == [{"limit": None}]
    assert_consumed(onceframe)
    assert_consumed(alias)


def test_explicit_analysis_domain_and_schema_mismatch(connection):
    connection.create_table(
        "source", pl.DataFrame({"value": [3]}, schema={"value": pl.Int32})
    )
    data = dp_ibis.scan_database(connection, "source")
    domain = dp.lazyframe_domain([dp.series_domain("value", dp.atom_domain(T="i32"))])
    context = dp.Context.compositor(
        data,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
        domain=domain,
        margins=[dp.polars.Margin(max_length=10)],
    )
    assert context.query_space is not None
    assert context.query_space[0] == dp.with_margin(
        domain, dp.polars.Margin(max_length=10)
    )
    assert context.query().select(dp.len(signed=False)).release().collect().shape == (
        1,
        1,
    )
    wrong = dp.lazyframe_domain([dp.series_domain("value", dp.atom_domain(T="i64"))])
    with pytest.raises(ValueError, match="schema"):
        dp.Context.compositor(
            data, dp.unit_of(contributions=1), dp.loss_of(epsilon=1.0), domain=wrong
        )


def test_bound_relation_is_not_replaced_by_its_base_table(connection):
    connection.create_table("source", pl.DataFrame({"value": [1, 2, 3]}))
    table = connection.table("source")
    source = dp_ibis._IbisSource(table.filter(table.value > 1))
    context = dp.Context.compositor(
        source,
        dp.unit_of(contributions=0),
        dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    assert context.query().select(dp.len(signed=False)).release().collect().item() == 2
    assert not hasattr(dp_ibis, "make_executor")


def test_approximate_threshold_search_with_frame_distance(connection, monkeypatch):
    connection.create_table(
        "source", pl.DataFrame({"key": [0, 0, 1], "value": [1, 2, 3]})
    )
    unit = dp.unit_of(contributions=[dp.polars.Bound(per_group=1)])
    context = dp.Context.compositor(
        dp_ibis.scan_database(connection, "source"),
        unit,
        dp.loss_of(epsilon=1.0, delta=1e-6),
        split_evenly_over=1,
        margins=[dp.polars.Margin(by=[pl.col("key")], max_length=10, max_groups=4)],
    )
    calls, stream = [], connection.to_pyarrow_batches
    collecting = False

    def guarded(expression, **kwargs):
        assert collecting, "calibration/release must not read SQL results"
        calls.append(kwargs)
        return stream(expression, **kwargs)

    def forbidden(*args, **kwargs):
        raise AssertionError("do not materialize private rows outside Arrow streaming")

    monkeypatch.setattr(connection, "to_pyarrow_batches", guarded)
    monkeypatch.setattr(connection, "execute", forbidden)
    monkeypatch.setattr(connection, "to_polars", forbidden)
    query = context.query().group_by("key").agg(dp.len(signed=True))
    assert isinstance(query, LazyFrameQuery)
    resolved = query.resolve()
    assert resolved.input_domain == context.accountant.input_domain
    assert resolved.input_metric == unit[0]
    assert resolved.map(unit[1]) == resolved._polars_measurement.map(unit[1])
    assert context.d_mids is not None
    assert resolved.check(unit[1], context.d_mids[0])
    assert query.summarize()["threshold"].item() > 0
    onceframe = query.release()
    assert calls == []
    collecting = True
    result = onceframe.collect()
    assert result.schema == {"key": pl.Int64, "len": pl.Int64}
    assert calls == [{"limit": None}]


def test_context_domain_cannot_be_replaced_by_another_analysis(connection):
    first = compositor(
        connection, pl.DataFrame({"value": [1]}, schema={"value": pl.Int32})
    )
    second = compositor(
        connection, pl.DataFrame({"value": [1]}, schema={"value": pl.Int32})
    )
    foreign = second.query().select(dp.len(signed=False)).resolve()
    assert foreign.input_domain != first.accountant.input_domain
    with pytest.raises(dp.OpenDPException, match="DomainMismatch"):
        first(foreign)
    assert len(first.remaining_privacy_loss()) == 1
    wrong = dp.lazyframe_domain([dp.series_domain("value", dp.atom_domain(T="i64"))])
    with pytest.raises(ValueError, match="analysis domain"):
        dp_ibis.make_private_ibis_lazyframe(
            wrong,
            first.accountant.input_metric,
            first.accountant.output_measure,
            pl.LazyFrame(schema={"value": pl.Int64}).select(dp.len(signed=False)),
            global_scale=1.0,
            source_domain=first.accountant.input_domain,
        )
