<<<<<<< HEAD
from opendp.transformations import *
from opendp.domains import lazy_frame_domain, series_domain, atom_domain
from opendp.metrics import symmetric_distance
from opendp.mod import enable_features
enable_features("contrib")


def test_series_domain():
    print(series_domain("A", atom_domain(T=float)))


def test_lazy_frame_domain():
    print(lazy_frame_domain([
        series_domain("A", atom_domain(T=float)),
        series_domain("B", atom_domain(T=int)),
        series_domain("C", atom_domain(T=str))
    ]))


def test_scan_csv():

    lfd = lazy_frame_domain([
        series_domain("A", atom_domain(T=float)),
        series_domain("B", atom_domain(T=int)),
    ])
    scanner = make_scan_csv(lfd, symmetric_distance())
    sinker = make_sink_csv(scanner.output_domain, symmetric_distance(), "output.csv")

    with open("input.csv", "w") as f:
        f.write("A,B\n1.0,2\n3.0,4\n")
    
    (scanner >> sinker)("input.csv")

    with open("output.csv", "r") as f:
        assert f.readlines() == ["A,B\n", "1.0,2\n", "3.0,4\n"]

    import os
    os.remove("input.csv")
    os.remove("output.csv")
    print(scanner)
    print(sinker)

=======
import opendp.prelude as dp
dp.enable_features("contrib")

def test_scan_csv():
    df_domain = dp.lazy_frame_domain([dp.series_domain("A", dp.atom_domain(T=float))])
    input_space = dp.csv_domain(df_domain), dp.symmetric_distance()

    _scanner = input_space >> dp.t.part_scan_csv()

def test_series_domain():
    print(dp.series_domain("A", dp.atom_domain(T=float)))
    print(dp.series_domain("A", dp.option_domain(dp.atom_domain(T=int))))


def test_lazy_frame_domain():
    print(dp.lazy_frame_domain([
        dp.series_domain("A", dp.atom_domain(T=float)),
        dp.series_domain("B", dp.atom_domain(T=int)),
        dp.series_domain("C", dp.atom_domain(T=str))
    ]))
>>>>>>> remotes/origin/773-sum-metrics
