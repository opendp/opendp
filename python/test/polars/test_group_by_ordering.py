"""Stable group-by key validation."""

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


def test_unordered_group_by_rejects_floating_key_representatives():
    domain, _ = _input()
    data = pl.LazyFrame({"key": [1, 1], "i": [1, 2], "x": [0.0, -0.0]})
    with pytest.raises(dp.OpenDPException, match="Floating-point grouping keys"):
        _stable(domain, dp.symmetric_distance(), data.group_by("x").agg(pl.len()))
