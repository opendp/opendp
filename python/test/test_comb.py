import pytest
from opendp.mod import enable_features
from opendp.meas import *
from opendp.trans import *
from opendp.typing import AllDomain, L1Distance, VectorDomain

enable_features("floating-point", "contrib")


def test_amplification():
    from opendp.trans import make_sized_bounded_mean
    from opendp.comb import make_population_amplification

    meas = make_sized_bounded_mean(size=10, bounds=(0., 10.)) >> make_base_laplace(scale=0.5)

    amplified = make_population_amplification(meas, population_size=100)
    print("amplified base laplace:", amplified([1.] * 10))
    assert meas.check(2, 2. + 1e-6)
    assert not meas.check(2, 2.)
    assert amplified.check(2, 1.494)
    assert not amplified.check(2, .494)


def test_fix_delta():
    from opendp.comb import make_fix_delta

    base_gaussian = make_base_gaussian(10.)
    print(base_gaussian.map(1.).epsilon(1e-6))
    fixed_base_gaussian = make_fix_delta(base_gaussian, 1e-6)

    print(fixed_base_gaussian.map(1.))


def test_make_sequential_composition_static_distances():
    from opendp.comb import make_sequential_composition_static_distances
    composed = make_sequential_composition_static_distances([
        make_count(TIA=int, TO=int) >> make_base_geometric(scale=2.), 
        make_count(TIA=int, TO=int) >> make_base_geometric(scale=200.), 
        make_cast_default(int, bool) >> make_cast_default(bool, int) >> make_count(TIA=int, TO=int) >> make_base_geometric(scale=2.), 
        make_cast_default(int, float) >> make_clamp((0., 10.)) >> make_bounded_sum((0., 10.)) >> make_base_laplace(scale=2.), 

        make_sequential_composition_static_distances([
            make_count(TIA=int, TO=int) >> make_base_geometric(scale=2.), 
            make_count(TIA=int, TO=float) >> make_base_laplace(scale=2.),
            make_cast_default(int, str) >> make_count_by_categories(categories=["0", "12", "22"]) >> make_base_geometric(scale=2., D=VectorDomain[AllDomain[int]]),
        ])
    ])

    print("Check:", composed.check(1, 2.))
    print("Forward map:", composed.map(3))
    print("Invocation:", composed.invoke([22, 12]))


@pytest.mark.skip(reason="long-running process to detect potential memory leaks")
def test_make_sequential_composition_static_distances_leak():
    from opendp.comb import make_sequential_composition_static_distances

    # choose a vector-valued mechanism that should run quickly for large inputs
    # we want to add as little noise as possible, so that execution time is small
    meas = make_base_geometric(scale=1e-6, D=VectorDomain[AllDomain[int]])

    # memory usage remains the same when this line is commented,
    # supporting that AnyObject's free recursively frees children
    meas = make_sequential_composition_static_distances([meas])

    # watch for leaked AnyObjects with 1 million i32 values
    # memory would jump by ~40mb every iteration
    for i in range(1000):
        print('iteration', i)
        meas([0] * 10_000_000)

if __name__ == "__main__":
    test_make_sequential_composition_static_distances()

