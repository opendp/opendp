import pytest
import opendp.prelude as dp
import polars as pl

dp.enable_features("contrib")

def test_scan_csv():
    df_domain = dp.lazy_frame_domain([dp.series_domain("A", dp.atom_domain(T=float))])
    input_space = dp.csv_domain(df_domain), dp.symmetric_distance()

    scanner = input_space >> dp.t.then_scan_csv()
    with pytest.raises(dp.OpenDPException) as err:
        scanner("A/B.csv")
    
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


def test_load_dataframe():
    domain, data = test_lazyframe_domain()
    trans_lazy = (
        (domain, dp.symmetric_distance())
        >> dp.t.then_collect()
        >> dp.t.then_lazy()
    )
    trans_lazy(data)

    trans_data = (domain, dp.symmetric_distance()) >> dp.t.then_collect()
    print(trans_data(data))
