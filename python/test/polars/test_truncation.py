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


@pytest.mark.parametrize("k", [0, 1, 2, 3, 5])
def test_truncate_per_group_sort_by_rank_counts(k):
    data = pl.LazyFrame({"id": [1, 1, 1, 1, 2, 2], "score": [1, 1, 1, 2, 1, 2]})
    for frame in [data, data.reverse()]:
        query = (
            _truncation_context(frame)
            .query()
            .truncate_per_group(k, keep=dp.polars.SortBy("score"))
        )
        actual = _run_truncation(frame, query)
        assert actual.filter(pl.col("id") == 1).height == min(k, 4)
        assert actual.filter(pl.col("id") == 2).height == min(k, 2)


def test_truncate_per_group_sort_by_multiple_keys():
    data = pl.LazyFrame({"id": [1, 1, 1], "score": [1, 1, 2], "secondary": [1, 9, 3]})
    keep = dp.polars.SortBy(
        by=["score", "secondary"],
        descending=[False, False],
        nulls_last=[False, False],
        maintain_order=True,
    )
    for frame in [data, data.reverse()]:
        query = _truncation_context(frame).query().truncate_per_group(1, keep=keep)
        actual = _run_truncation(frame, query)
        assert actual["secondary"].to_list() == [1]


@pytest.mark.parametrize(
    "options, message",
    [
        ({"descending": [False, True]}, "same sort direction"),
        ({"descending": [False]}, "same sort direction"),
        ({"nulls_last": True}, "custom null placement"),
        ({"nulls_last": [False, True]}, "custom null placement"),
    ],
)
def test_truncate_per_group_sort_by_rejects_unsupported_options(options, message):
    data = pl.LazyFrame({"id": [1, 1], "a": [1, 2], "b": [2, 1]})
    query = _truncation_context(data).query()
    with pytest.raises(ValueError, match=message):
        query.truncate_per_group(1, keep=dp.polars.SortBy(["a", "b"], **options))


def test_truncate_per_group_sort_by_ignores_sorted_metadata():
    # A sorted-state hint must not let Polars skip the priority ordering.
    data = pl.DataFrame(
        {"id": [1, 1, 1], "score": pl.Series([3, 1, 2]).set_sorted()}
    ).lazy()
    query = (
        _truncation_context(data)
        .query()
        .truncate_per_group(1, keep=dp.polars.SortBy(pl.col("score")))
    )
    assert _run_truncation(data, query)["score"].to_list() == [1]


def test_truncate_per_group_sort_by_rejects_positional_key():
    data = pl.LazyFrame({"id": [1, 1, 1], "x": [2, 1, 3]})
    query = (
        _truncation_context(data)
        .query()
        .truncate_per_group(1, keep=dp.polars.SortBy(pl.int_range(pl.len())))
    )
    with pytest.raises(
        dp.OpenDPException, match="SortBy truncation keys must be permutation-equivariant"
    ):
        _run_truncation(data, query)


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
    context = dp.Context.compositor(
        data=pl.LazyFrame({"id": [1, 1], "value": [1, 2]}),
        privacy_unit=dp.unit_of(contributions=1, identifier="id"),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    with pytest.raises(ValueError, match="keep must be 'sample' or SortBy"):
        context.query().truncate_per_group(1, keep=keep)


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


@pytest.mark.parametrize("k", [0, 1, 2, 4])
@pytest.mark.parametrize("keep", ["sample", "first", "last"])
def test_truncate_num_groups_keeps_k_groups(k, keep):
    data = pl.LazyFrame({"id": [1, 1, 1, 1, 2, 2], "group": [0, 1, 1, 2, 0, 1]})
    for frame in [data, data.reverse()]:
        query = (
            _truncation_context(frame)
            .query()
            .truncate_num_groups(k, by=["group"], keep=keep)
        )
        actual = _run_truncation(frame, query)
        for identifier, total_groups in [(1, 3), (2, 2)]:
            selected = actual.filter(pl.col("id") == identifier)
            assert selected["group"].n_unique() == min(k, total_groups)
            # Entire groups are retained, including duplicate keys.
            assert selected.filter(pl.col("group") == 1).height in (
                0,
                2 if identifier == 1 else 1,
            )


@pytest.mark.parametrize("k", [0, 1, 3, 10])
def test_sample_truncation_multiple_groups(k):
    data = pl.LazyFrame(
        {
            "id": [1, 1, 1, 1, 2, 2],
            "group": [0, 0, 0, 1, 0, 1],
            "payload": [10, 11, 12, 13, 14, 15],
        }
    )
    for frame in [data, data.reverse()]:
        query = _truncation_context(frame).query().truncate_per_group(k, by=["group"])
        input_domain, input_metric = query._query._chain
        transformation = dp.t.make_stable_lazyframe(
            input_domain,
            input_metric,
            query.polars_plan,
            MO="FrameDistance<SymmetricDistance>",
        )
        actual = transformation(frame).collect()
        expected_counts = {
            (1, 0): min(k, 3),
            (1, 1): min(k, 1),
            (2, 0): min(k, 1),
            (2, 1): min(k, 1),
        }
        for (identifier, group), count in expected_counts.items():
            assert (
                actual.filter(
                    (pl.col("id") == identifier) & (pl.col("group") == group)
                ).height
                == count
            )
        rows = actual.rows()
        assert len(set(rows)) == len(rows)
        assert set(rows) <= set(data.collect().rows())


@pytest.mark.parametrize(
    "kind",
    [
        "physical",
        "sorted_indices",
        "inverse_ranks",
        "shuffle",
        "reverse",
    ],
)
def test_truncation_rejects_unsafe_enumerations(kind):
    query = _truncation_context(pl.LazyFrame({"id": [1] * 3, "x": [3, 1, 2]})).query()
    indexes = pl.int_range(pl.len())
    if kind in ["sorted_indices", "inverse_ranks", "random_ties"]:
        keys = [pl.col("x")]
        if kind == "random_ties":
            keys.append(indexes.dp.permute())
        indexes = indexes.sort_by(keys)
        if kind != "sorted_indices":
            indexes = indexes.arg_sort()
    elif kind == "shuffle":
        indexes = indexes.shuffle()
    elif kind == "reverse":
        indexes = indexes.reverse()
    query = query.filter(indexes.over("id") < 1)
    with pytest.raises(dp.OpenDPException):
        query.select(dp.len(signed=True)).summarize()
