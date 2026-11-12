import opendp.prelude as dp


def test_dp_mean():
    data = [0.0, 17000.0, 9100.0, 350000.0]
    bounds = (0.0, 10000.0)
    n = 1000
    scale = 1.0
    space = (
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        dp.symmetric_distance(),
    )
    preprocessor = (
        space
        >> dp.t.then_clamp(bounds)
        >> dp.t.then_resize(n, 0.0)
        >> dp.t.then_mean()
        >> dp.m.then_laplace(scale)
    )
    res = preprocessor(data)
    assert isinstance(res, float)
