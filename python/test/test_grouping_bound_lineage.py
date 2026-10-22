"""Contribution bounds must describe grouping keys in the current query stage."""

import math

import opendp.prelude as dp
import pytest

pl = pytest.importorskip("polars")

NUM_GROUPS = 100
TARGET_KEY = 500


def _data(include_user=True):
    indices = list(range(NUM_GROUPS)) if include_user else []
    return pl.LazyFrame(
        {
            "id": [7] * len(indices) + [8, 9],
            "A": [TARGET_KEY - 2 * index for index in indices]
            + [TARGET_KEY - 1, TARGET_KEY - 1],
            "B": indices + [1, 1],
        },
        schema={"id": pl.Int32, "A": pl.Int32, "B": pl.Int32},
    )


def _query(
    bound_source,
    key_mode,
    loss,
    filter_at=None,
    include_user=True,
    global_truncation=False,
):
    data = _data(include_user)
    key = pl.col("A") + pl.col("B")
    if key_mode == "precomputed":
        data = data.with_columns(key.alias("key"))
        key = pl.col("key")
    elif key_mode == "aliased":
        key = key.alias("key")

    context = dp.Context.compositor(
        data=data,
        privacy_unit=dp.unit_of(contributions=1, identifier="id"),
        privacy_loss=dp.loss_of(**loss, delta=1e-7),
        split_evenly_over=1,
        margins=(
            [dp.polars.Margin(by=[key], max_groups=NUM_GROUPS)]
            if bound_source == "margin"
            else []
        ),
    )
    query = context.query()
    if global_truncation:
        query = query.truncate_per_group(NUM_GROUPS, by=[], keep="first")
    if filter_at == "before":
        query = query.filter(pl.col("B") >= 0)
    if bound_source == "truncate":
        query = query.truncate_num_groups(NUM_GROUPS, by=[key], keep="first")
    query = query.group_by(pl.col("id"), key).agg(pl.col("B").sum())
    if filter_at == "after":
        query = query.filter(pl.col("B") >= 0)

    # The first aggregation writes the expression result into A (or its alias).
    # Re-evaluating A + B therefore does not retain the original grouping.
    output_key = {
        "reused": pl.col("A") + pl.col("B"),
        "materialized": pl.col("A"),
        "aliased": pl.col("key"),
        "precomputed": pl.col("key"),
    }[key_mode]
    return query.group_by(output_key).agg(dp.len(signed=True))


def _missing_contribution_bound(query):
    with pytest.raises(dp.OpenDPException) as error:
        query.summarize()
    assert error.value.variant == "FailedMap"
    message = error.value.message
    assert message is not None
    assert (
        "num_groups (None), total contributions (None), and per_group (None)" in message
    )
    assert "are not sufficiently well-defined" in message
    return message


def test_reused_grouping_expression_collapses_one_users_groups():
    """Deterministic count difference from the example reported in #3082."""
    key = pl.col("A") + pl.col("B")

    def count(include_user):
        return (
            _data(include_user)
            .group_by(pl.col("id"), key)
            .agg(pl.col("B").sum())
            .group_by(key)
            .agg(pl.len())
            .filter(pl.col("A") == TARGET_KEY)
            .collect()["len"]
            .sum()
        )

    assert count(True) - count(False) == NUM_GROUPS


@pytest.mark.parametrize("bound_source", ["truncate", "margin"])
@pytest.mark.parametrize(
    "key_mode", ["reused", "materialized", "aliased", "precomputed"]
)
@pytest.mark.parametrize("loss", [{"rho": 0.5}, {"epsilon": 1.0}])
def test_grouping_bounds_follow_materialized_keys(bound_source, key_mode, loss):
    query = _query(bound_source, key_mode, loss)
    if key_mode == "reused":
        _missing_contribution_bound(query)
        return

    summary = query.summarize().row(0, named=True)
    expected_scale = NUM_GROUPS
    if "rho" in loss:
        expected_scale = math.sqrt(NUM_GROUPS)

    assert summary["scale"] == pytest.approx(expected_scale)
    expected_distribution = "Integer Gaussian" if "rho" in loss else "Integer Laplace"
    assert summary["distribution"] == expected_distribution
    assert summary["threshold"] > 1


@pytest.mark.parametrize("bound_source", ["truncate", "margin"])
@pytest.mark.parametrize("filter_at", ["before", "after"])
@pytest.mark.parametrize("key_mode", ["reused", "materialized"])
def test_grouping_bounds_through_row_filters(bound_source, filter_at, key_mode):
    query = _query(bound_source, key_mode, {"rho": 0.5}, filter_at=filter_at)
    if key_mode == "reused":
        _missing_contribution_bound(query)
        return
    summary = query.summarize().row(0, named=True)
    assert summary["scale"] == pytest.approx(math.sqrt(NUM_GROUPS))


@pytest.mark.parametrize("bound_source", ["truncate", "margin"])
@pytest.mark.parametrize(
    "key_mode", ["reused", "materialized", "aliased", "precomputed"]
)
def test_grouping_calibration_does_not_depend_on_one_users_presence(
    bound_source, key_mode
):
    queries = [
        _query(bound_source, key_mode, {"rho": 0.5}, include_user=include_user)
        for include_user in [True, False]
    ]
    if key_mode == "reused":
        errors = [_missing_contribution_bound(query) for query in queries]
        assert errors[0] == errors[1]
        return
    summaries = [query.summarize().to_dicts() for query in queries]
    assert summaries[0] == summaries[1]
    assert summaries[0][0]["scale"] == pytest.approx(math.sqrt(NUM_GROUPS))


@pytest.mark.parametrize("bound_source", ["truncate", "margin"])
@pytest.mark.parametrize("loss", [{"rho": 0.5}, {"epsilon": 1.0}])
def test_reused_grouping_expression_with_global_truncation(bound_source, loss):
    summary = (
        _query(bound_source, "reused", loss, global_truncation=True)
        .summarize()
        .row(0, named=True)
    )
    assert summary["scale"] == pytest.approx(NUM_GROUPS)
    assert summary["threshold"] > NUM_GROUPS

    # In this fixture each (id, A + B) group contains one row, so the two
    # grouping steps assign each original row to A + 2 * B.
    # An independent context with a total row bound needs the same calibration.
    reference_context = dp.Context.compositor(
        data=_data().with_columns((pl.col("A") + 2 * pl.col("B")).alias("key")),
        privacy_unit=dp.unit_of(contributions=NUM_GROUPS),
        privacy_loss=dp.loss_of(**loss, delta=1e-7),
        split_evenly_over=1,
    )
    reference = (
        reference_context.query()
        .group_by("key")
        .agg(dp.len(signed=True))
        .summarize()
        .row(0, named=True)
    )
    assert summary["scale"] == pytest.approx(reference["scale"])
    assert summary["distribution"] == reference["distribution"]
    assert summary["threshold"] == reference["threshold"]
