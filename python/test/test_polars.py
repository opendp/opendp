import pytest
import opendp.prelude as dp
import polars as pl

dp.enable_features("contrib", "honest-but-curious")

# domains
def test_series_domain():
    return [
        dp.series_domain("A", dp.atom_domain(T=float)),
        dp.series_domain("B", dp.atom_domain(T=int)),
        dp.series_domain("C", dp.option_domain(dp.atom_domain(T=str))),
    ], {
        "A": [1.0] * 50,
        "B": [1] * 50,
        "C": ["1"] * 50,
    }

def test_lazyframe_domain():
    domains, data = test_series_domain()
    return dp.lazyframe_domain(domains), pl.LazyFrame(data)

def test_dataframe_domain():
    domains, data = test_series_domain()
    return dp.dataframe_domain(domains), pl.DataFrame(data)

# data loaders
def test_series_ffi():
    """ensure that series can be passed to/from Rust"""
    domains, datas = test_series_domain()
    for domain, (name, data) in zip(domains, datas.items()):
        ident_trans = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
        print(ident_trans(pl.Series(name, data)))

def test_lazyframe_ffi():
    """ensure that lazyframes can be passed to/from Rust"""
    domain, data = test_lazyframe_domain()
    ident_trans = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
    print(ident_trans(data))

def test_dataframe_ffi():
    """ensure that dataframes can be passed to/from Rust"""
    domain, data = test_dataframe_domain()
    ident_trans = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
    print(ident_trans(data))
