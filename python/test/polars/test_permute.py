"""Tests for the OpenDP uniform random permutation expression."""

import pytest

import opendp.extras.polars  # noqa: F401 - registers the dp expression namespace

pl = pytest.importorskip("polars")


def test_permute_all():
    data = pl.DataFrame({"x": list(range(40))})
    result = data.select(pl.col("x").dp.permute())["x"].to_list()
    assert sorted(result) == list(range(40))


def test_permute_struct_alignment():
    data = pl.DataFrame({"x": list(range(30)), "y": [n * 17 for n in range(30)]})
    sample = data.select(pl.struct("x", "y").dp.permute()).to_series()
    rows = sample.to_list()
    assert len(rows) == 30
    assert all(row["y"] == 17 * row["x"] for row in rows)
    assert len({row["x"] for row in rows}) == 30


@pytest.mark.parametrize("values", [[], [None], [None, 1, 1, 2]])
def test_permute_nullable_and_empty(values):
    data = pl.DataFrame({"x": pl.Series(values, dtype=pl.Int64)})
    result = data.select(pl.col("x").dp.permute())["x"]
    assert result.dtype == pl.Int64
    assert len(result) == len(values)
    assert result.sort().to_list() == data["x"].sort().to_list()


def test_permute_grouped_struct():
    data = pl.DataFrame({"g": [0, 0, 0, 1, 1], "x": [1, 2, 3, 4, 5]})
    data = data.with_columns(y=pl.col("x") * 17)
    result = data.group_by("g").agg(pl.struct("x", "y").dp.permute().alias("sample"))
    for group, rows in result.iter_rows():
        expected = data.filter(pl.col("g") == group)["x"].to_list()
        assert len(rows) == len(expected)
        assert len({row["x"] for row in rows}) == len(rows)
        assert all(row["x"] in expected and row["y"] == 17 * row["x"] for row in rows)
