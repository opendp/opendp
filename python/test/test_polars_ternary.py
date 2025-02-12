# This is more thoroughly tested than other Exprs only because
# Chuck is new to this, and is more comfortable testing in python than rust, for now.

import pytest
import opendp.prelude as dp


def example_series():
    pl = pytest.importorskip("polars")
    return [
        dp.series_domain("ones", dp.atom_domain(T=dp.f64)),
        dp.series_domain("twos", dp.atom_domain(T=dp.f64, bounds=(0, 10))),
        dp.series_domain("optional", dp.option_domain(dp.atom_domain(T=dp.f64))),
    ], [
        pl.Series("ones", [1.0] * 50, dtype=pl.Float64),
        pl.Series("twos", [2.0] * 50, dtype=pl.Float64),
        pl.Series("optional", [3.0, None] * 25, dtype=pl.Float64),
    ]


def example_lf(margin=None, **kwargs):
    pl = pytest.importorskip("polars")
    domains, series = example_series()
    lf_domain, lf = dp.lazyframe_domain(domains), pl.LazyFrame(series)
    if margin is not None:
        lf_domain = dp.with_margin(lf_domain, by=margin, **kwargs)
    return lf_domain, lf


def test_when_then_otherwise_const():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    m_lf = dp.t.make_stable_lazyframe(
        lf_domain,
        dp.symmetric_distance(),
        lf.select(
            pl.when(pl.col("ones") == 1).then(1).otherwise(0).alias('fifty'),
            pl.when(pl.col("ones") == 0).then(1).otherwise(0).alias('zero'),
        ),
    )
    assert m_lf.map(1) == 1
    results = m_lf(lf).collect().sum()
    assert results['fifty'].item() == 50
    assert results['zero'].item() == 0


def test_when_then_otherwise_col():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()

    non_dp = lf.select(
        pl.when(pl.col("ones") == 1)
        .then(pl.col("twos"))
        .otherwise(pl.col("optional"))
    )
    expected_name = non_dp.collect().columns[0]
    assert expected_name == "twos"

    m_lf = dp.t.make_stable_lazyframe(
        lf_domain,
        dp.symmetric_distance(),
        lf.select(
            pl.when(pl.col("ones") == 1)
            .then(pl.col("twos"))
            .otherwise(pl.col("optional"))
        ),
    )
    
    # Would prefer not to depend on string for assertions,
    # but I don't think the iternals are exposed in another way.
    # TODO: Shouldn't this be optional? Should we be able to see that here?
    assert str(m_lf.output_domain) == 'FrameDomain(twos: f64; margins=[])'

    df = m_lf(lf).collect()
    assert df.schema[expected_name].to_python() == float

    results = df.sum()
    assert results[expected_name].item() == 100


def test_when_then_otherwise_strings():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    m_lf = dp.t.make_stable_lazyframe(
        lf_domain,
        dp.symmetric_distance(),
        lf.select(
            pl.when(pl.col("ones") == 1).then(pl.lit("one")).otherwise(pl.lit("other")),
        ),
    )
    assert m_lf(lf).collect()['literal'][0] == 'one'


# TODO: Should we support this, by casting the int to float?
def test_when_then_otherwise_int_float_mismatch():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    with pytest.raises(dp.OpenDPException, match=r'output dtypes in ternary must match'):
        m_lf = dp.t.make_stable_lazyframe(
            lf_domain,
            dp.symmetric_distance(),
            lf.select(
                pl.when(pl.col("ones") == 1).then(1).otherwise(1.0),
            ),
        )


def test_when_then_otherwise_num_str_mismatch():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    with pytest.raises(dp.OpenDPException, match=r'output dtypes in ternary must match'):
        dp.t.make_stable_lazyframe(
            lf_domain,
            dp.symmetric_distance(),
            lf.select(
                pl.when(pl.col("ones") == 1).then(1).otherwise(pl.lit("!!!")).alias('fifty'),
            ),
        )


# TODO: Shouldn't error. More notes in rust code.
@pytest.mark.xfail(raises=dp.OpenDPException)
def test_when_then_otherwise_incomplete():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    dp.t.make_stable_lazyframe(
        lf_domain,
        dp.symmetric_distance(),
        lf.select(
            pl.when(pl.col("ones") == 1).then(1).alias('fifty'),
        ),
    )