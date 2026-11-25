"""Permutation-invariant sample-and-aggregate under unordered adjacency."""

import pytest

import opendp.prelude as dp

pl = pytest.importorskip("polars")


def _input():
    data = pl.LazyFrame(
        {"key": [1, 2, 1, 2], "i": [1, 2, 3, 4], "x": [1.0, 2.0, 3.0, 4.0]}
    )
    domain = dp.lazyframe_domain(
        [
            dp.series_domain("key", dp.atom_domain(T=dp.i64)),
            dp.series_domain("i", dp.atom_domain(T=dp.i64)),
            dp.series_domain("x", dp.atom_domain(T=dp.f64)),
        ]
    )
    return domain, data


def _stable(domain, metric, plan):
    return dp.t.make_stable_lazyframe(
        domain, metric, plan, MO="FrameDistance<SymmetricDistance>"
    )


def test_permute_cannot_be_combined_with_row_aligned_expression():
    domain, data = _input()
    value = pl.struct(pl.col("i").dp.permute().alias("shuffled"), pl.col("i")).first()
    with pytest.raises(dp.OpenDPException, match="permute cannot be combined"):
        _stable(domain, dp.symmetric_distance(), data.group_by("key").agg(value))


def test_group_by_rejects_nonpublic_quantile():
    domain, data = _input()
    plan = data.group_by("key").agg(
        pl.col("x").dp.permute().quantile(pl.col("x").first())
    )
    with pytest.raises(dp.OpenDPException, match="public numeric scalar literal"):
        _stable(domain, dp.symmetric_distance(), plan)


@pytest.mark.parametrize("maintain_order", [False, True])
def test_group_by_aggregation_permutation(maintain_order):
    domain, data = _input()
    x = pl.col("x")
    plan = data.group_by("key", maintain_order=maintain_order).agg(
        pl.col("i").sum().alias("integer"),
        x.dp.permute().mean().alias("explicit_mean"),
        x.abs().mean().alias("mean"),
        x.sum().alias("sum"),
        x.clip(0.0, 3.0).median().alias("median"),
        x.round(1).std().alias("std"),
        x.quantile(0.5, interpolation="linear").alias("quantile"),
        x.is_finite().sum().alias("finite"),
    )
    transformation = _stable(domain, dp.symmetric_distance(), plan)
    for frame in (data, data.reverse()):
        actual = transformation(frame).collect().sort("key")
        assert actual["integer"].to_list() == [4, 6]
        assert actual["explicit_mean"].to_list() == [2.0, 3.0]
        assert actual["mean"].to_list() == [2.0, 3.0]
        assert actual["sum"].to_list() == [4.0, 6.0]
        assert actual["median"].to_list() == [2.0, 2.5]
        assert actual["std"].to_list() == pytest.approx([2**0.5, 2**0.5])
        assert actual["quantile"].to_list() == [2.0, 3.0]
        assert actual["finite"].to_list() == [2, 2]


def test_unordered_group_by_rejects_floating_key_representatives():
    domain, _ = _input()
    data = pl.LazyFrame({"key": [1, 1], "i": [1, 2], "x": [0.0, -0.0]})
    with pytest.raises(dp.OpenDPException, match="Floating-point grouping keys"):
        _stable(domain, dp.symmetric_distance(), data.group_by("x").agg(pl.len()))


def test_change_one_remains_supported():
    domain, data = _input()
    plan = data.group_by("key").agg(pl.col("i").sum())
    transformation = _stable(domain, dp.change_one_distance(), plan)
    assert transformation.map(1)[0].per_group == 4
    assert transformation(data).collect().height == 2


def test_unordered_group_by_postprocessing():
    domain, data = _input()
    plan = (
        data.group_by("key")
        .agg(pl.col("i").sum())
        .select("key", "i")
        .filter(pl.col("i") > 0)
    )
    transformation = _stable(domain, dp.symmetric_distance(), plan)
    assert str(transformation.output_metric) == "FrameDistance(SymmetricDistance())"
    assert transformation.map(1)[0].per_group == 2
    assert transformation(data).collect().shape == (2, 2)


def test_string_join_cannot_hide_order_dependence_in_integer_sum():
    domain, data = _input()
    expr = (
        pl.col("i")
        .cast(pl.String, strict=False)
        .str.join("|")
        .str.contains("1|3", literal=True)
        .cast(pl.Int64, strict=False)
        .sum()
    )
    with pytest.raises(
        dp.OpenDPException, match="permutation behavior is not certified"
    ):
        _stable(domain, dp.symmetric_distance(), data.group_by("key").agg(expr))


@pytest.mark.parametrize("method", ["first", "last"])
def test_order_dependent_reducers_require_explicit_permutation(method):
    domain, data = _input()
    expr = getattr(pl.col("x"), method)()
    with pytest.raises(dp.OpenDPException, match="Order-sensitive aggregation"):
        _stable(domain, dp.symmetric_distance(), data.group_by("key").agg(expr))
