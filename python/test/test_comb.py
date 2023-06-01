import opendp.prelude as dp
import pytest

dp.enable_features("floating-point", "contrib", "honest-but-curious")


def test_amplification():
    meas = dp.t.make_sized_bounded_mean(size=10, bounds=(0., 10.)) >> dp.m.make_base_laplace(scale=0.5)

    amplified = dp.c.make_population_amplification(meas, population_size=100)
    print("amplified base laplace:", amplified([1.] * 10))
    assert meas.check(2, 2. + 1e-6)
    assert not meas.check(2, 2.)
    assert amplified.check(2, 1.494)
    assert not amplified.check(2, .494)

def test_fix_delta():
    base_gaussian = dp.c.make_zCDP_to_approxDP(dp.m.make_base_gaussian(10.))
    print(base_gaussian.map(1.).epsilon(1e-6))
    fixed_base_gaussian = dp.c.make_fix_delta(base_gaussian, 1e-6)

    print(fixed_base_gaussian.map(1.))


def test_make_basic_composition():
    composed = dp.c.make_basic_composition([
        dp.t.make_count(TIA=int, TO=int) >> dp.c.make_basic_composition([
            dp.m.make_base_discrete_laplace(scale=2.), 
            dp.m.make_base_discrete_laplace(scale=200.)
        ]),
        dp.t.make_cast_default(int, bool) >> dp.t.make_cast_default(bool, int) >> dp.t.make_count(TIA=int, TO=int) >> dp.m.make_base_discrete_laplace(scale=2.), 
        dp.t.make_cast_default(int, float) >> dp.t.part_clamp((0., 10.)) >> dp.t.make_bounded_sum((0., 10.)) >> dp.m.make_base_laplace(scale=2.), 

        dp.c.make_basic_composition([
            dp.t.make_count(TIA=int, TO=int) >> dp.m.make_base_discrete_laplace(scale=2.), 
            dp.t.make_count(TIA=int, TO=float) >> dp.m.make_base_laplace(scale=2.),
            (
                dp.t.make_cast_default(int, str) >> 
                dp.t.make_count_by_categories(categories=["0", "12", "22"]) >> 
                dp.m.make_base_discrete_laplace(scale=2., D=dp.VectorDomain[dp.AtomDomain[int]])
            )
        ])
    ])

    print("Check:", composed.check(1, 2.))
    print("Forward map:", composed.map(3))
    print("Invocation:", composed.invoke([22, 12]))


@pytest.mark.skip(reason="long-running process to detect potential memory leaks")
def test_make_basic_composition_leak():

    # choose a vector-valued mechanism that should run quickly for large inputs
    # we want to add as little noise as possible, so that execution time is small
    meas = dp.m.make_base_discrete_laplace(scale=1e-6, D=dp.VectorDomain[dp.AtomDomain[int]])

    # memory usage remains the same when this line is commented,
    # supporting that AnyObject's free recursively frees children
    meas = dp.c.make_basic_composition([meas])

    # watch for leaked AnyObjects with 1 million i32 values
    # memory would jump by ~40mb every iteration
    for i in range(1000):
        print('iteration', i)
        meas([0] * 10_000_000)
    

def test_make_basic_composition_approx():
    composed_fixed = dp.c.make_basic_composition([
        dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(dp.m.make_base_gaussian(1.)), 1e-7)
    ] * 2)
    print(composed_fixed.map(1.))


def test_cast_zcdp_approxdp():
    base_gaussian = dp.m.make_base_gaussian(10., MO=dp.ZeroConcentratedDivergence[float])

    print(base_gaussian.map(1.))

    smd_gaussian = dp.c.make_zCDP_to_approxDP(base_gaussian)

    print(smd_gaussian.map(1.).epsilon(1e-6))
    

def test_make_pureDP_to_fixed_approxDP():
    meas = dp.c.make_basic_composition([
        dp.c.make_pureDP_to_fixed_approxDP(dp.m.make_base_laplace(10.)),
        dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(dp.m.make_base_gaussian(10.)), delta=1e-6)
    ])

    print(meas.map(1.))


def test_make_pureDP_to_zCDP():
    meas = dp.c.make_basic_composition([
        dp.c.make_pureDP_to_zCDP(dp.m.make_base_laplace(10.)),
        dp.m.make_base_gaussian(10.)
    ])

    print(meas.map(1.))

if __name__ == "__main__":
    test_make_pureDP_to_fixed_approxDP()
