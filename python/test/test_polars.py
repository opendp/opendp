import pytest
import opendp.prelude as dp


dp.enable_features("contrib", "honest-but-curious")


def seed(schema):
    pl = pytest.importorskip("polars")
    return pl.DataFrame(None, schema, orient="row").lazy() # type: ignore[attr-defined]


def example_series():
    pl = pytest.importorskip("polars")
    return [
        dp.series_domain("A", dp.atom_domain(T=dp.f64)),
        dp.series_domain("B", dp.atom_domain(T=dp.i32)),
        dp.series_domain("C", dp.option_domain(dp.atom_domain(T=dp.String))),
        dp.series_domain("D", dp.atom_domain(T=dp.i32)),
    ], [
        pl.Series("A", [1.0] * 50, dtype=pl.Float64),
        pl.Series("B", [1, 2, 3, 4, 5] * 10, dtype=pl.Int32),
        pl.Series("C", ["1"] * 49 + [None], dtype=pl.String),
        pl.Series("D", [2] * 50, dtype=pl.Int32),
    ]

def test_infer():
    pl = pytest.importorskip("polars")
    series = pl.Series("A", [1] * 100)
    assert dp.RuntimeType.infer(series) == "Series"
    assert dp.RuntimeType.infer(pl.DataFrame(series)) == "DataFrame"
    assert dp.RuntimeType.infer(pl.LazyFrame(series)) == "LazyFrame"
    assert dp.RuntimeType.infer(pl.col("A")) == "Expr"


def example_lf(margin=None, **kwargs):
    pl = pytest.importorskip("polars")
    domains, series = example_series()
    lf_domain, lf = dp.lazyframe_domain(domains), pl.LazyFrame(series)
    if margin is not None:
        lf_domain = dp.with_margin(lf_domain, by=margin, **kwargs)
    return lf_domain, lf


def test_lazyframe_domain_infer():
    _, lf = example_lf()
    assert dp.infer_lazyframe_domain(lf).member(lf)


def test_domains():
    example_lf(margin=["B"])
    example_lf(margin=["B"], public_info="keys", max_partition_length=50)
    example_lf(
        margin=["B"], 
        public_info="lengths", 
        max_partition_length=50,
        max_num_partitions=3,
        max_partition_contributions=2,
        max_influenced_partitions=1)


# data loaders
@pytest.mark.parametrize("domain,series", zip(*example_series()))
def test_series_ffi(domain, series):
    """ensure that series can be passed to/from Rust"""
    t_ident = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
    assert t_ident(series).equals(series)


def test_lazyframe_ffi():
    """ensure that lazyframes can be passed to/from Rust"""
    lf_domain, lf = example_lf()
    t_ident = (lf_domain, dp.symmetric_distance()) >> dp.t.then_identity()
    assert t_ident(lf).collect().equals(lf.collect())

def test_expr_ffi():
    """ensure that lazyframes can be passed to/from Rust"""
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    expr_domain = dp.expr_domain(lf_domain, "select")
    t_ident = (expr_domain, dp.symmetric_distance()) >> dp.t.then_identity()
    assert str(t_ident((lf, pl.col("A")))[1]) == str(pl.col("A"))
