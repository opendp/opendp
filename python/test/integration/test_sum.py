import opendp.prelude as dp


def test_sized_bounded_float_sum():
    """known-n bounded float sum (assuming n is public)"""
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
        >> dp.t.then_sum()
    )
    laplace_sum = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s), d_in=1, d_out=1.0
    )
    gaussian_sum = dp.binary_search_chain(
        lambda s: dp.c.make_fix_delta(
            dp.c.make_zCDP_to_approxDP(preprocess >> dp.m.then_gaussian(s)), 1e-5
        ),
        d_in=1,
        d_out=(1.0, 1e-5),
    )

    assert laplace_sum.check(1, 1.0)
    data = [1.0] * size
    print(laplace_sum(data))
    print(gaussian_sum(data))


def test_sized_bounded_int_sum():
    """known-n bounded int sum (assuming n is public)"""
    size = 200
    bounds = (0, 20)
    space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    preprocess = (
        space >> dp.t.then_clamp(bounds) >> dp.t.then_resize(size, 0) >> dp.t.then_sum()
    )
    noisy_sum = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s), d_in=1, d_out=1.0
    )
    assert noisy_sum.check(1, 1.0)
    print(noisy_sum([1] * size))


def test_bounded_float_sum():
    """bounded float sum (assuming n is unknown)"""
    bounds = (0.0, 20.0)
    space = (
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        dp.symmetric_distance(),
    )
    preprocess = space >> dp.t.then_clamp(bounds) >> dp.t.then_sum()
    laplace_sum = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s), d_in=1, d_out=1.0
    )
    gaussian_sum = dp.binary_search_chain(
        lambda s: dp.c.make_fix_delta(
            dp.c.make_zCDP_to_approxDP(preprocess >> dp.m.then_gaussian(s)), 1e-5
        ),
        d_in=1,
        d_out=(1.0, 1e-5),
    )
    assert laplace_sum.check(1, 1.0)
    data = [1.0] * 100
    print(laplace_sum(data))
    print(gaussian_sum(data))


def test_bounded_int_sum():
    """bounded int sum (assuming n is unknown)"""
    bounds = (0, 20)
    space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    preprocess = space >> dp.t.then_clamp(bounds) >> dp.t.then_sum()
    noisy_sum = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s), d_in=1, d_out=1.0
    )
    assert noisy_sum.check(1, 1.0)
    print(noisy_sum([1] * 100))
