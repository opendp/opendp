"""Stable group-by key validation."""

import pytest

import opendp.prelude as dp

pl = pytest.importorskip("polars")


def test_unordered_group_by_rejects_floating_key_representatives():
    domain = dp.lazyframe_domain([dp.series_domain("x", dp.atom_domain(T=dp.f64))])
    data = pl.LazyFrame({"x": [0.0, -0.0]})
    with pytest.raises(dp.OpenDPException, match="Floating-point grouping keys"):
        dp.t.make_stable_lazyframe(
            domain,
            dp.symmetric_distance(),
            data.group_by("x").agg(pl.len()),
            MO="FrameDistance<SymmetricDistance>",
        )
