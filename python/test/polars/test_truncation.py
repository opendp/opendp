from collections import Counter

import opendp.prelude as dp
import pytest
import re

pl = pytest.importorskip("polars")


def test_lazyframe_bounded_dp_truncation():
    context = dp.Context.compositor(
        data=pl.LazyFrame({"alpha": ["A", "B", "C"] * 100, "id": [1, 2, 3] * 100}),
        privacy_unit=dp.unit_of(changes=1, identifier="id"),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=1,
    )

    # the unused with_columns is for test coverage of .truncate,
    # which retrieves input domain from prior query
    query = (
        context.query()
        .with_columns(x=pl.lit(10))
        .truncate_per_group(3)
        .select(dp.len(signed=True))
    )
    assert query.summarize()["scale"][0] == 6.000000000000001  # type: ignore[index]


def test_unnecessary_lazyframe_truncation():
    context = dp.Context.compositor(
        data=pl.LazyFrame({"alpha": ["A", "B", "C"] * 100, "id": [1, 2, 3] * 100}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=1,
        margins=[dp.polars.Margin(by=(), max_length=300)],
    )

    with pytest.raises(ValueError, match="truncation is only valid when"):
        context.query().truncate_per_group(3)
    with pytest.raises(ValueError, match="truncation is only valid when"):
        context.query().truncate_num_groups(3, by=["alpha"])


def test_frame_distance():
    context = dp.Context.compositor(
        data=pl.LazyFrame({"alpha": ["A", "B", "C"] * 100, "id": range(300)}),
        privacy_unit=dp.unit_of(
            contributions=[dp.polars.Bound(per_group=2)], identifier="id"
        ),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-8),
        split_evenly_over=1,
    )

    query = (
        context.query()
        # user can contribute one record per id per group (2 records per group)
        .truncate_per_group(1, by=["alpha"])
        # user can contribute one group per id (2 groups total)
        .truncate_num_groups(1, by=["alpha"])
        .group_by("alpha")
        .agg(dp.len(signed=True))
    )
    # ...therefore sensitivity of count is 2 * 2
    assert query.summarize()["scale"][0] == 4.000000000000001  # type: ignore[index]


def test_truncate_per_group():
    context = dp.Context.compositor(
        data=pl.LazyFrame({"alpha": ["A", "B", "C"] * 100, "id": [1, 2, 3] * 100}),
        privacy_unit=dp.unit_of(contributions=1, identifier="id"),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=1,
    )

    query = context.query().truncate_per_group(2).select(dp.len(signed=True))
    assert query.summarize()["scale"][0] == 2.0000000000000004  # type: ignore[index]

    context = dp.Context.compositor(
        data=pl.LazyFrame(
            {"alpha": ["A", "B", "C", "D"] * 100, "id": list(range(100)) * 4}
        ),
        privacy_unit=dp.unit_of(
            contributions=[
                dp.polars.Bound(
                    by=["alpha"],
                    num_groups=1,
                    per_group=1,
                )
            ],
            identifier="id",
        ),
        privacy_loss=dp.loss_of(rho=0.5, delta=1e-7),
        split_evenly_over=1,
    )

    query = (
        context.query()
        .truncate_per_group(2, by=["alpha"])
        .group_by("alpha")
        .agg(dp.len(signed=True))
    )
    assert query.summarize()["scale"][0] == 2.0000000000000004  # type: ignore[index]


@pytest.mark.parametrize("descending", [False, True])
def test_truncate_per_group_sort_by(descending):
    data = pl.LazyFrame(
        {
            "id": [1, 1, 1, 1, 2, 2],
            "group": [0, 0, 0, 1, 0, 0],
            "score": [3, 1, 1, 5, 2, 9],
        }
    )
    for frame in [data, data.reverse()]:
        query = (
            _truncation_context(frame)
            .query()
            .truncate_per_group(
                1,
                by=["group"],
                keep=dp.polars.SortBy(pl.col("score"), descending=descending),
            )
        )
        actual = _run_truncation(frame, query)
        assert actual.height == 3
        scores = {
            (identifier, group): score
            for identifier, group, score in actual.iter_rows()
        }
        assert scores == {
            (1, 0): 3 if descending else 1,
            (1, 1): 5,
            (2, 0): 9 if descending else 2,
        }


def test_truncate_per_group_sort_by_negated_computed_key():
    data = pl.LazyFrame({"id": [1, 1, 1, 2, 2], "score": [1, 3, 2, 8, 7]})
    for frame in [data, data.reverse()]:
        query = (
            _truncation_context(frame)
            .query()
            .with_columns(priority=pl.col("score").cast(pl.Float64))
            .truncate_per_group(1, keep=dp.polars.SortBy(-pl.col("priority")))
        )
        actual = _run_truncation(frame, query)
        assert dict(actual.select("id", "score").iter_rows()) == {1: 3, 2: 8}


def test_truncate_per_group_sort_by_rejects_unsupported_options():
    data = pl.LazyFrame({"id": [1, 1], "a": [1, 2], "b": [2, 1]})
    query = _truncation_context(data).query()
    with pytest.raises(ValueError, match="Mixed descending directions"):
        query.truncate_per_group(
            1, keep=dp.polars.SortBy(["a", "b"], descending=[False, True])
        )


@pytest.mark.parametrize("descending, expected", [(False, 100), (True, 3)])
def test_truncate_per_group_sort_by_native_null_order_per_key(descending, expected):
    data = pl.LazyFrame(
        {
            "id": [1, 1, 1, 1],
            "first": [1, 1, None, 1],
            "second": [None, 3, 100, 2],
        }
    )
    for frame in [data, data.reverse()]:
        query = (
            _truncation_context(frame)
            .query()
            .truncate_per_group(
                1,
                keep=dp.polars.SortBy(["first", "second"], descending=descending),
            )
        )
        actual = _run_truncation(frame, query)
        assert actual["second"].to_list() == [expected]


@pytest.mark.parametrize(
    "scores",
    [
        [-0.0, 0.0, 1.0],
        [float("nan"), 0.0, 1.0],
    ],
)
def test_truncate_per_group_sort_by_float_edge_cases(scores):
    data = pl.LazyFrame({"id": [1, 1, 1], "score": scores, "row": [0, 1, 2]})
    for frame in [data, data.reverse()]:
        query = (
            _truncation_context(frame)
            .query()
            .truncate_per_group(2, keep=dp.polars.SortBy("score"))
        )
        actual = _run_truncation(frame, query)
        expected = set(data.sort("score").head(2).collect()["row"].to_list())
        assert set(actual["row"].to_list()) == expected


def test_truncate_error_messages():
    context = dp.Context.compositor(
        data=pl.LazyFrame(
            {
                "alpha": ["A", "B", "C"] * 100,
                "id": [1, 2, 3] * 100,
                "sort": [3, 2, 1] * 100,
            }
        ),
        privacy_unit=dp.unit_of(contributions=1, identifier="id"),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=1,
    )

    query = (
        context.query().truncate_num_groups(1, by=["alpha"]).select(dp.len(signed=True))
    )
    with pytest.raises(
        dp.OpenDPException,
        match="`per_group` contributions is unknown. This is likely due to a missing truncation",
    ):
        query.summarize()

    query = (
        context.query()
        .truncate_num_groups(1, by=["alpha"])
        .group_by("sort")
        .agg(dp.len(signed=True))
    )
    with pytest.raises(
        dp.OpenDPException,
        match=re.escape(
            'To bound `num_groups` in the Context API, try using `.truncate_num_groups(num_groups, by=[col("sort")])`. To bound `per_group` in the Context API, try using `.truncate_per_group(per_group, by=[col("sort")])`.'
        ),
    ):
        query.summarize()


@pytest.mark.parametrize("keep", ["first", "last"])
def test_truncate_num_groups(keep):
    context = dp.Context.compositor(
        data=pl.LazyFrame({"alpha": ["A", "B", "C"] * 100, "id": [1, 2, 3] * 100}),
        privacy_unit=dp.unit_of(contributions=1, identifier="id"),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=1,
    )

    query = (
        context.query()
        .truncate_per_group(2)
        .truncate_num_groups(1, keep=keep, by=["alpha"])
        .group_by("alpha")
        .agg(dp.len(signed=True))
    )
    assert query.summarize()["scale"][0] == 2.0000000000000004  # type: ignore[index]


def test_truncation_contingency():
    synth_context = dp.Context.compositor(
        data=pl.scan_csv(dp.examples.get_france_lfs_path(), encoding="utf8-lossy"),
        privacy_unit=dp.unit_of(contributions=1, identifier="PIDENT"),
        privacy_loss=dp.loss_of(epsilon=1),
        split_evenly_over=1,
    )

    synth_query = (
        synth_context.query()
        .truncate_per_group(1)
        .select("ILOSTAT")
        .contingency_table(
            keys={"ILOSTAT": []},
            # makes it run fast!
            algorithm=dp.mbi.Fixed(queries=[dp.mbi.Count(("ILOSTAT",))]),
        )
    )
    synth_query.release()


@pytest.mark.parametrize("keep", ["first", "last"])
def test_truncate_per_group_rejects_physical_order(keep):
    data = pl.LazyFrame({"id": [1, 1], "value": [1, 2]})
    with pytest.raises(ValueError, match="keep must be 'sample' or SortBy"):
        _truncation_context(data).query().truncate_per_group(1, keep=keep)


def _truncation_context(data):
    return dp.Context.compositor(
        data=data,
        privacy_unit=dp.unit_of(contributions=1, identifier="id"),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )


def _run_truncation(data, query):
    input_domain, input_metric = query._query._chain
    transformation = dp.t.make_stable_lazyframe(
        input_domain,
        input_metric,
        query.polars_plan,
        MO="FrameDistance<SymmetricDistance>",
    )
    return transformation(data).collect()


@pytest.mark.parametrize("k, sort", [(0, False), (1, False), (10, False), (2, True)])
def test_truncate_per_group_counts(k, sort):
    data = pl.LazyFrame(
        {
            "id": [1, 1, 1, 1, 2, 2],
            "group": [0, 0, 0, 1, 0, 1],
            "payload": [10, 11, 12, 13, 14, 15],
        }
    )
    # All priorities within each group tie. Ordinal ranks must still keep k rows.
    keep = dp.polars.SortBy("group") if sort else "sample"
    expected = {
        (1, 0): min(k, 3),
        (1, 1): min(k, 1),
        (2, 0): min(k, 1),
        (2, 1): min(k, 1),
    }
    query = (
        _truncation_context(data).query().truncate_per_group(k, by=["group"], keep=keep)
    )
    actual = _run_truncation(data, query)
    assert Counter(actual.select("id", "group").iter_rows()) == Counter(expected)
    rows = actual.rows()
    assert len(set(rows)) == len(rows)
    assert set(rows) <= set(data.collect().rows())


def test_group_by_truncation_automatic_permutation():
    data = pl.LazyFrame({"id": [1, 1, 2], "x": [1.0, 3.0, 5.0]})
    query = _truncation_context(data).query().group_by("id").agg(pl.col("x").mean())
    actual = _run_truncation(data, query).sort("id")
    assert actual["x"].to_list() == [2.0, 5.0]
