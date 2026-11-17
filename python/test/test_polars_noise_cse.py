"""Check that optimizations do not reuse independently sampled DP noise."""

import pytest

import opendp.prelude as dp
from opendp.extras.polars import Margin

pl = pytest.importorskip("polars")


@pytest.mark.parametrize("use_lazyframe", [False, True])
def test_independent_noise_for_imputed_public_keys(use_lazyframe):
    """CSE must not merge the noise draws for two absent-group imputations."""
    data = pl.LazyFrame(
        {"key": [1], "x": [0.25], "y": [0.75]},
        schema_overrides={"key": pl.Int32},
    )
    public_keys = pl.LazyFrame({"key": pl.Series([1, 2], dtype=pl.Int32)})
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

    pairs = []
    for _ in range(4):
        onceframe = measurement(data)
        if use_lazyframe:
            result = onceframe.lazy().collect(
                optimizations=pl.QueryOptFlags(comm_subexpr_elim=False)
            )
        else:
            result = onceframe.collect()
        absent = result.filter(pl.col("key") == 2)
        assert absent.height == 1
        pairs.append((absent["sum_x"][0], absent["sum_y"][0]))

    # Exact equality of independent continuous draws is negligibly likely.
    assert all(x != y for x, y in pairs), (
        f"Missing-group noise reused (via lazyframe={use_lazyframe}): {pairs}"
    )
