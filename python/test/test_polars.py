import pytest
import opendp.prelude as dp

dp.enable_features("contrib", "honest-but-curious")


def example_series():
    pl = pytest.importorskip("polars")
    return [
        dp.series_domain("A", dp.atom_domain(T=float)),
        dp.series_domain("B", dp.atom_domain(T=int)),
        dp.series_domain("C", dp.option_domain(dp.atom_domain(T=str))),
        dp.series_domain("D", dp.atom_domain(T=int)),
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


# data loaders
@pytest.mark.parametrize("domain,series", zip(*example_series()))
def test_series_ffi(domain, series):
    """ensure that series can be passed to/from Rust"""
    t_ident = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
    assert t_ident(series).equals(series)
