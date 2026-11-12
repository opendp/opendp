import opendp.prelude as dp


def test_sized_bounded_variance():
    """known-n bounded float variance (assuming n is public)"""
    size = 200
    bounds = (0.0, 20.0)
    space = (
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        dp.symmetric_distance(),
    )
    preprocess = (
        space
        >> dp.t.then_clamp(bounds)
        >> dp.t.then_resize(size, 0.0)
        >> dp.t.then_variance()
    )
    noisy_variance = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s), d_in=1, d_out=1.0
    )
    assert noisy_variance.check(1, 1.0)
    print(noisy_variance([1.0] * size))
