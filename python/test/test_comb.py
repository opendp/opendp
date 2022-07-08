from opendp.mod import enable_features

enable_features("floating-point", "contrib")


def test_amplification():
    from opendp.trans import make_sized_bounded_mean
    from opendp.meas import make_base_laplace
    from opendp.comb import make_population_amplification

    meas = make_sized_bounded_mean(size=10, bounds=(0., 10.)) >> make_base_laplace(scale=0.5)

    amplified = make_population_amplification(meas, population_size=100)
    print("amplified base laplace:", amplified([1.] * 10))
    assert meas.check(2, 2.)
    assert not meas.check(2, 1.999)
    assert amplified.check(2, 1.494)
    assert not amplified.check(2, .494)

def test_fix_delta():
    from opendp.comb import make_fix_delta
    from opendp.meas import make_base_gaussian

    base_gaussian = make_base_gaussian(10.)
    print(base_gaussian.map(1.).epsilon(1e-6))
    fixed_base_gaussian = make_fix_delta(base_gaussian, 1e-6)

    print(fixed_base_gaussian.map(1.))
