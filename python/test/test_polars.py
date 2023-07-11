import pytest
import opendp.prelude as dp
import polars as pl

dp.enable_features("contrib", "honest-but-curious")

# domains
def series_domain():
    return [
        dp.series_domain("A", dp.atom_domain(T=float)),
        dp.series_domain("B", dp.atom_domain(T=int)),
        dp.series_domain("C", dp.option_domain(dp.atom_domain(T=str))),
    ], {
        "A": [1.0] * 50,
        "B": [1] * 50,
        "C": ["1"] * 50,
    }

def lazyframe_domain():
    domains, data = series_domain()
    return dp.lazyframe_domain(domains), pl.LazyFrame(data)

def dataframe_domain():
    domains, data = series_domain()
    return dp.dataframe_domain(domains), pl.DataFrame(data)

def lazyframe_domain_with_counts():
    counts = pl.LazyFrame({"B": [1], "counts": [50]}, schema_overrides={"B": pl.Int32, "counts": pl.UInt32})
    domain, data = lazyframe_domain()
    return domain.with_counts(counts), data

def dataframe_domain_with_counts():
    counts = pl.DataFrame({"B": [1], "counts": [50]}, schema_overrides={"B": pl.Int32, "counts": pl.UInt32})
    domain, data = dataframe_domain()
    return domain.with_counts(counts), data

def expr_domain():
    lf_domain = lazyframe_domain()[0]
    expr_domain = dp.expr_domain(lf_domain, grouping_columns=["A"], active_column="B")

    dp.expr_domain(lf_domain, context="filter", active_column="B")
    with pytest.raises(dp.OpenDPException):
        dp.expr_domain(lf_domain, context="X", active_column="B")

    # TODO: data loader for ExprDomain members
    return expr_domain, None


def test_domains():
    lazyframe_domain_with_counts()
    dataframe_domain_with_counts()
    expr_domain()


# data loaders
def test_series_ffi():
    """ensure that series can be passed to/from Rust"""
    domains, datas = series_domain()
    for domain, (name, data) in zip(domains, datas.items()):
        ident_trans = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
        print(ident_trans(pl.Series(name, data)))

def test_lazyframe_ffi():
    """ensure that lazyframes can be passed to/from Rust"""
    domain, data = lazyframe_domain()
    ident_trans = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
    print(ident_trans(data))

def test_dataframe_ffi():
    """ensure that dataframes can be passed to/from Rust"""
    domain, data = dataframe_domain()
    ident_trans = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
    print(ident_trans(data))

# constructors
def test_scan_csv():
    df_domain = dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T=float))])
    input_space = dp.csv_domain(df_domain), dp.symmetric_distance()

    scanner = input_space >> dp.t.then_scan_csv()
    with pytest.raises(dp.OpenDPException) as err:
        scanner("A/B.csv")
    

def test_collect_lazy():
    domain, data = lazyframe_domain()
    space = domain, dp.symmetric_distance()
    trans_lazy = space >> dp.t.then_collect() >> dp.t.then_lazy()
    trans_lazy(data)


def test_make_with_columns():
    domain, data = lazyframe_domain()
    metric = dp.symmetric_distance()
    expr_domain = dp.expr_domain(domain, context="with_columns")

    trans_lazy = (domain, metric) >> dp.t.then_with_columns([
        (expr_domain, metric) >> dp.t.then_col("A") >> dp.t.then_clamp_expr((1., 2.))
    ])
    trans_lazy(data)
