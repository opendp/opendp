import opendp.prelude as dp
import pytest
import re


def test_lazyframe_bounded_dp_truncation():
    pl = pytest.importorskip("polars")

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
        .select(dp.len())
    )
    assert query.summarize()["scale"][0] == 6.000000000000001  # type: ignore[index]


def test_unnecessary_lazyframe_truncation():
    pl = pytest.importorskip("polars")

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
    pl = pytest.importorskip("polars")

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
        .agg(dp.len())
    )
    # ...therefore sensitivity of count is 2 * 2
    assert query.summarize()["scale"][0] == 4.000000000000001  # type: ignore[index]


@pytest.mark.parametrize("keep", ["first", "last", "sample"])
def test_truncate_per_group(keep):
    pl = pytest.importorskip("polars")

    context = dp.Context.compositor(
        data=pl.LazyFrame({"alpha": ["A", "B", "C"] * 100, "id": [1, 2, 3] * 100}),
        privacy_unit=dp.unit_of(contributions=1, identifier="id"),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=1,
    )

    query = context.query().truncate_per_group(2, keep=keep).select(dp.len())
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
        .truncate_per_group(2, by=["alpha"], keep=keep)
        .group_by("alpha")
        .agg(dp.len())
    )
    assert query.summarize()["scale"][0] == 2.0000000000000004  # type: ignore[index]


def test_truncate_per_group_sort_by():
    pl = pytest.importorskip("polars")

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
        context.query()
        .truncate_per_group(2, keep=dp.polars.SortBy(pl.col("sort")))
        .select(dp.len())
    )
    assert query.summarize()["scale"][0] == 2.0000000000000004  # type: ignore[index]


def test_truncate_error_messages():
    pl = pytest.importorskip("polars")

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

    query = context.query().truncate_num_groups(1, by=["alpha"]).select(dp.len())
    with pytest.raises(
        dp.OpenDPException,
        match="`per_group` contributions is unknown. This is likely due to a missing truncation",
    ):
        query.summarize()

    query = (
        context.query()
        .truncate_num_groups(1, by=["alpha"])
        .group_by("sort")
        .agg(dp.len())
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
    pl = pytest.importorskip("polars")

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
        .agg(dp.len())
    )
    assert query.summarize()["scale"][0] == 2.0000000000000004  # type: ignore[index]


def test_truncation_contingency():
    pl = pytest.importorskip("polars")
    synth_context = dp.Context.compositor(
        data=pl.scan_csv(
            dp.examples.get_france_lfs_path(), encoding="utf8-lossy", ignore_errors=True
        ),
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
