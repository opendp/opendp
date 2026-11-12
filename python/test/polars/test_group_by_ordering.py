"""Permutation-invariant sample-and-aggregate under unordered adjacency."""

import pytest

import opendp.prelude as dp

pl = pytest.importorskip("polars")
pl_testing = pytest.importorskip("polars.testing")


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


@pytest.mark.parametrize("method", ["first", "sum"])
def test_unordered_group_by_requires_invariant_aggregate(method):
    domain, data = _input()
    with pytest.raises(dp.OpenDPException, match="Order-sensitive aggregation"):
        _stable(
            domain,
            dp.symmetric_distance(),
            data.group_by("key").agg(getattr(pl.col("x"), method)()),
        )



@pytest.mark.parametrize(
    "value",
    [
        lambda: pl.col("i").abs().min(),
        lambda: pl.col("i").fill_null(0).sum(),
        lambda: pl.col("i").is_null().sum(),
        lambda: pl.when(pl.col("i") > 2).then(pl.col("i")).otherwise(0).max(),
        lambda: pl.col("i").drop_nulls().min(),
    ],
)
def test_rowwise_functions_remain_supported(value):
    domain, data = _input()
    plan = data.group_by("key").agg(value())
    transformed = _stable(domain, dp.symmetric_distance(), plan)
    for frame in (data, data.reverse()):
        assert transformed(frame).collect().sort("key").height == 2


@pytest.mark.parametrize(
    "value",
    [
        lambda: pl.col("i").shift(1).sum(),
        lambda: pl.col("i").reverse().min(),
        lambda: (pl.col("i").dp.permute() + pl.col("i")).sum(),
    ],
)
def test_order_dependent_inputs_remain_rejected(value):
    domain, data = _input()
    with pytest.raises(dp.OpenDPException, match="Order-sensitive aggregation"):
        _stable(domain, dp.symmetric_distance(), data.group_by("key").agg(value()))

def test_group_by_rejects_positional_literals():
    domain, data = _input()
    weights = pl.lit(pl.Series([1, 0, 0, 0]))
    plan = data.group_by("key").agg((pl.col("i") * weights).sum())
    metric = dp.symmetric_distance()
    with pytest.raises(dp.OpenDPException, match="Only scalar literals"):
        _stable(domain, metric, plan)


def test_group_by_rejects_physical_group_indices():
    domain, data = _input()
    metric = dp.symmetric_distance()
    with pytest.warns(DeprecationWarning, match=r"agg_groups\(\) is deprecated"):
        plan = data.group_by("key").agg(pl.col("x").sort().agg_groups())
    with pytest.raises(dp.OpenDPException, match="physical row indices"):
        _stable(domain, metric, plan)


@pytest.mark.parametrize("quantile", ["private", float("nan")])
def test_group_by_rejects_nonpublic_or_invalid_quantile(quantile):
    domain, data = _input()
    quantile = pl.col("x").first() if quantile == "private" else pl.lit(quantile)
    plan = data.group_by("key").agg(pl.col("x").sort().quantile(quantile))
    with pytest.raises(dp.OpenDPException, match="public numeric scalar literal"):
        _stable(domain, dp.symmetric_distance(), plan)


@pytest.mark.parametrize("maintain_order", [False, True])
def test_unordered_group_by_accepts_permute(maintain_order):
    domain, data = _input()
    values = pl.col("x").dp.permute()  # type: ignore[attr-defined]
    plan = data.group_by("key", maintain_order=maintain_order).agg(
        (pl.col("i").sum() + 1).alias("integer"), values.mean().alias("mean")
    )
    transformation = _stable(domain, dp.symmetric_distance(), plan)
    expected = pl.DataFrame({"key": [1, 2], "integer": [5, 7], "mean": [2.0, 3.0]})
    for frame in [data, data.reverse()]:
        pl_testing.assert_frame_equal(
            transformation(frame).collect().sort("key"), expected
        )


def test_integer_sort_does_not_certify_sorted_metadata():
    domain, _ = _input()
    # Polars may use this metadata to skip sort; it isn't an ordering proof.
    data = pl.DataFrame(
        {
            "key": [1, 1, 1],
            "i": pl.Series([3, 1, 2], dtype=pl.Int64).set_sorted(),
            "x": [1.0, 2.0, 3.0],
        }
    ).lazy()
    plan = data.group_by("key").agg(pl.col("i").sort().first())
    with pytest.raises(dp.OpenDPException, match="Order-sensitive aggregation"):
        _stable(domain, dp.symmetric_distance(), plan)


def test_unordered_group_by_rejects_floating_key_representatives():
    domain, _ = _input()
    for values in [[0.0, -0.0], [-0.0, 0.0]]:
        data = pl.LazyFrame({"key": [1, 1], "i": [1, 2], "x": values})
        with pytest.raises(dp.OpenDPException, match="Floating-point grouping keys"):
            _stable(domain, dp.symmetric_distance(), data.group_by("x").agg(pl.len()))


@pytest.mark.parametrize("method", ["min", "max"])
def test_float_reducers_reject_sorted_metadata_fast_paths(method):
    domain, _ = _input()
    for values in [[0.0, -0.0], [-0.0, 0.0]]:
        data = pl.DataFrame(
            {"key": [1, 1], "i": [1, 2], "x": pl.Series(values).set_sorted()}
        ).lazy()
        plan = data.group_by("key").agg(getattr(pl.col("x"), method)())
        with pytest.raises(dp.OpenDPException, match="Order-sensitive aggregation"):
            _stable(domain, dp.symmetric_distance(), plan)
        safe_plan = data.group_by("key").agg(
            getattr(pl.col("x").dp.permute(), method)()
        )
        assert (
            _stable(domain, dp.symmetric_distance(), safe_plan)(data).collect().height
            == 1
        )


@pytest.mark.parametrize("metric", ["insert_delete", "hamming"])
def test_sample_and_aggregate_rejects_ordered_input(metric):
    domain, data = _input()
    input_metric, output_metric = {
        "insert_delete": (
            dp.insert_delete_distance(),
            "FrameDistance<InsertDeleteDistance>",
        ),
        "hamming": (dp.hamming_distance(), "FrameDistance<InsertDeleteDistance>"),
    }[metric]
    plan = data.group_by("key").agg(pl.col("i").sum())
    with pytest.raises(dp.OpenDPException, match="requires an unordered input metric"):
        dp.t.make_stable_lazyframe(domain, input_metric, plan, MO=output_metric)


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
