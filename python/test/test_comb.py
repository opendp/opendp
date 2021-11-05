from opendp.mod import enable_features

enable_features("floating-point", "contrib")


def test_amplification():
    from opendp.trans import make_sized_bounded_mean
    from opendp.meas import make_base_laplace
    from opendp.comb import make_population_amplification
    # prior info
    size = 10
    bounds = (0., 10.)

    scale = 0.5
    meas = make_sized_bounded_mean(size=size, bounds=bounds) >> make_base_laplace(scale=scale)
    ideal_epsilon = (bounds[1] - bounds[0]) / size / scale
    assert meas.check(2, ideal_epsilon)
    assert not meas.check(2, ideal_epsilon - 1e-6)
    amplified = make_population_amplification(meas, population_size=size * 10)
    print("amplified base laplace:", amplified([1.] * 10))
    assert amplified.check(2, ideal_epsilon / 4.)
    assert not amplified.check(2, ideal_epsilon / 5.)
