import pytest
import opendp.prelude as dp
dp.enable_features("contrib")

def test_scan_csv():
    df_domain = dp.lazy_frame_domain([dp.series_domain("A", dp.atom_domain(T=float))])
    input_space = dp.csv_domain(df_domain), dp.symmetric_distance()

    scanner = input_space >> dp.t.then_scan_csv()
    with pytest.raises(dp.OpenDPException) as err:
        scanner("A/B.csv")
    
def test_series_domain():
    print(dp.series_domain("A", dp.atom_domain(T=float)))
    print(dp.series_domain("A", dp.option_domain(dp.atom_domain(T=int))))


def test_lazy_frame_domain():
    print(dp.lazy_frame_domain([
        dp.series_domain("A", dp.atom_domain(T=float)),
        dp.series_domain("B", dp.atom_domain(T=int)),
        dp.series_domain("C", dp.atom_domain(T=str))
    ]))
