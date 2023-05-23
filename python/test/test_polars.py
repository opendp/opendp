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

