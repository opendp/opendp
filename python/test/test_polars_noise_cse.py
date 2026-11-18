"""Check that optimizations do not reuse independently sampled DP noise."""

import pytest

import opendp.prelude as dp
from opendp.extras.polars import Margin

pl = pytest.importorskip("polars")


@pytest.mark.parametrize(
    "collection_mode",
    [
        "onceframe",
        "lazy_without_cse",
        pytest.param(
            "lazy_with_cse",
            marks=pytest.mark.xfail(
                strict=True,
                reason="Polars CSE currently merges independent DP noise draws",
            ),
        ),
    ],
)
def test_independent_noise_for_imputed_public_keys(collection_mode):
    """Imputed public keys need independent noise across groups and aggregates."""
    data = pl.LazyFrame(
        {"key": [1], "x": [0.25], "y": [0.75]},
        schema_overrides={"key": pl.Int32},
    )
    public_keys = pl.LazyFrame({"key": pl.Series([1, 2, 3], dtype=pl.Int32)})
    domain = dp.with_margin(
        dp.lazyframe_domain(
            [
                dp.series_domain("key", dp.atom_domain(T=dp.i32)),
                dp.series_domain("x", dp.atom_domain(T=dp.f64)),
                dp.series_domain("y", dp.atom_domain(T=dp.f64)),
            ]
        ),
        Margin(by=[pl.col("key")], max_groups=1, max_length=1),
    )
    query = (
        pl.LazyFrame(schema=data.collect_schema())
        .group_by("key")
        .agg(
            pl.col("x").dp.sum((0.0, 1.0), scale=10.0).alias("sum_x"),
            pl.col("y").dp.sum((0.0, 1.0), scale=10.0).alias("sum_y"),
        )
        .join(public_keys, on="key", how="right")
    )
    measurement = dp.m.make_private_lazyframe(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        query,
        global_scale=1.0,
    )

    for _ in range(4):
        onceframe = measurement(data)
        if collection_mode == "onceframe":
            result = onceframe.collect()
        else:
            # Explicitly re-enable CSE for the xfail, bypassing the OnceFrame guard.
            result = onceframe.lazy().collect(
                optimizations=pl.QueryOptFlags(
                    comm_subexpr_elim=collection_mode == "lazy_with_cse"
                )
            )

        absent = result.filter(pl.col("key").is_in([2, 3])).sort("key")
        assert absent["key"].to_list() == [2, 3]
        draws = absent["sum_x"].to_list() + absent["sum_y"].to_list()

        # All four noise draws must be independent: across keys and columns.
        # Strict xfail makes a future CSE fix visible as an XPASS failure.
        assert len(set(draws)) == 4, (
            f"Missing-group noise reused (via {collection_mode}): {draws}"
        )
