"""Focused tests for the restricted OpenDP deterministic-head lowerer."""

import ibis
import polars as pl
import pytest
from polars.testing import assert_frame_equal

from opendp.extras.ibis._lower import convert_polars_to_ibis


@pytest.fixture
def source():
    pytest.importorskip("duckdb")
    connection = ibis.duckdb.connect()
    frame = pl.DataFrame(
        {
            "key": pl.Series([0, 0, 1, None], dtype=pl.Int32),
            "value": pl.Series([1, 2, 3, 4], dtype=pl.Int32),
        }
    )
    connection.create_table("source", frame)
    try:
        yield frame, connection, connection.table("source")
    finally:
        connection.disconnect()


def test_global_reductions(source):
    frame, connection, table = source
    query = frame.lazy().select(
        pl.len().alias("length"),
        pl.col("value").sum().alias("sum"),
        pl.col("value").min().alias("minimum"),
    )
    expr = convert_polars_to_ibis(query, table=table)
    assert_frame_equal(connection.to_polars(expr), query.collect())


def test_grouped_reductions(source):
    frame, connection, table = source
    query = frame.lazy().group_by("key").agg(
        pl.col("value").clip(0, 3).sum().alias("sum"),
        pl.col("value").count().alias("count"),
    )
    expr = convert_polars_to_ibis(query, table=table)
    assert_frame_equal(
        connection.to_polars(expr).sort("key"),
        query.collect().sort("key"),
    )


def test_histogram_buckets(source):
    frame, connection, table = source
    bucket = (
        pl.when(pl.col("value") < 2)
        .then(pl.lit(0, dtype=pl.UInt64))
        .otherwise(
            pl.when(pl.col("value") == 2)
            .then(pl.lit(1, dtype=pl.UInt64))
            .otherwise(pl.lit(2, dtype=pl.UInt64))
        )
    )
    query = (
        frame.lazy()
        .with_columns(bucket.alias("bucket"))
        .group_by("key", "bucket")
        .agg(pl.len().cast(pl.UInt64).alias("count"))
    )
    expr = convert_polars_to_ibis(query, table=table)
    assert_frame_equal(
        connection.to_polars(expr).sort("key", "bucket"),
        query.collect().sort("key", "bucket"),
    )


def test_scan_dtype_is_preserved(source):
    frame, connection, table = source
    assert convert_polars_to_ibis(frame.lazy().select("value"), table=table).schema()[
        "value"
    ] == ibis.dtype("int32")


@pytest.mark.parametrize(
    "query",
    [
        lambda lf: lf.sort("value"),
        lambda lf: lf.select(pl.col("value").median()),
        lambda lf: lf.with_columns(pl.col("value").shift(1)),
        lambda lf: lf.join(lf, on="key"),
    ],
)
def test_unrelated_polars_ops_are_rejected(source, query):
    frame, _, table = source
    with pytest.raises(NotImplementedError):
        convert_polars_to_ibis(query(frame.lazy()), table=table)


def test_schema_mismatch_rejected(source):
    frame, _, table = source
    wrong = frame.lazy().with_columns(pl.col("value").cast(pl.Int64)).select("value")
    # The cast operates on the same input schema, so it is allowed.
    assert convert_polars_to_ibis(wrong, table=table) is not None
    with pytest.raises(ValueError, match="schema"):
        convert_polars_to_ibis(
            pl.LazyFrame(schema={"key": pl.Int64, "value": pl.Int32}).select("value"),
            table=table,
        )
