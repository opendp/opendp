# This is more thoroughly tested than other Exprs only because
# Chuck is new to this, and is more comfortable testing in python than rust, for now.

import pytest
import opendp.prelude as dp


def example_lf(margin=None, **kwargs):
    pl = pytest.importorskip("polars")
    domains, series = example_series()
    lf_domain, lf = dp.lazyframe_domain(domains), pl.LazyFrame(series)
    if margin is not None:
        lf_domain = dp.with_margin(lf_domain, by=margin, **kwargs)
    return lf_domain, lf


def example_series():
    pl = pytest.importorskip("polars")
    return [
        dp.series_domain("A", dp.option_domain(dp.atom_domain(T=dp.f64))),
        # dp.series_domain("B", dp.atom_domain(T=dp.i32)),
        # dp.series_domain("C", dp.option_domain(dp.atom_domain(T=dp.String))),
        # dp.series_domain("D", dp.atom_domain(T=dp.i32)),
    ], [
        pl.Series("A", [1.0] * 50, dtype=pl.Float64),
        # pl.Series("B", [1, 2, 3, 4, 5] * 10, dtype=pl.Int32),
        # pl.Series("C", ["1"] * 49 + [None], dtype=pl.String),
        # pl.Series("D", [2] * 50, dtype=pl.Int32),
    ]


def test_when_then_otherwise():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    m_lf = dp.t.make_stable_lazyframe(
        lf_domain,
        dp.symmetric_distance(),
        lf.select(
            pl.when(pl.col("A") == 1).then(1).otherwise(0).alias('fifty'),
            pl.when(pl.col("A") == 0).then(1).otherwise(0).alias('zero'),
        ),
    )
    results = m_lf(lf).collect().sum()
    assert results['fifty'].item() == 50
    assert results['zero'].item() == 0


def test_when_then_otherwise_strings():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    m_lf = dp.t.make_stable_lazyframe(
        lf_domain,
        dp.symmetric_distance(),
        lf.select(
            pl.when(pl.col("A") == 1).then(pl.lit("one")).otherwise(pl.lit("other")),
        ),
    )
    assert m_lf(lf).collect()['literal'][0] == 'one'


def test_when_then_otherwise_mismatch_types():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    with pytest.raises(dp.OpenDPException, match=r'output domains in ternary must match'):
        dp.t.make_stable_lazyframe(
            lf_domain,
            dp.symmetric_distance(),
            lf.select(
                pl.when(pl.col("A") == 1).then(1).otherwise(pl.lit("!!!")).alias('fifty'),
            ),
        )


def test_when_then_otherwise_incomplete():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    with pytest.raises(Exception, match=r'unsupported literal value: null'):
        dp.t.make_stable_lazyframe(
            lf_domain,
            dp.symmetric_distance(),
            lf.select(
                pl.when(pl.col("A") == 1).then(1).alias('fifty'),
            ),
        )
    # TODO: Should there be a better error message?