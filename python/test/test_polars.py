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
    counts = pl.LazyFrame({"B": [1], "counts": [50]}, schema_overrides={"B": pl.Int32})
    domain, data = lazyframe_domain()
    return domain.with_counts(counts), data

def dataframe_domain_with_counts():
    counts = pl.DataFrame({"B": [1], "counts": [50]}, schema_overrides={"B": pl.Int32})
    domain, data = dataframe_domain()
    return domain.with_counts(counts), data

def test_domains():
    lazyframe_domain_with_counts()
    dataframe_domain_with_counts()


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

