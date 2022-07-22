import pytest
from opendp.mod import enable_features
from opendp.meas import *
from opendp.trans import *
from opendp.typing import AllDomain, VectorDomain

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


def test_make_basic_composition():
    from opendp.comb import make_basic_composition
    composed = make_basic_composition([
        make_count(TIA=int, TO=int) >> make_basic_composition([
            make_base_geometric(scale=2.), 
            make_base_geometric(scale=200.)
        ]), 
        make_cast_default(int, bool) >> make_cast_default(bool, int) >> make_count(TIA=int, TO=int) >> make_base_geometric(scale=2.), 
        make_cast_default(int, float) >> make_clamp((0., 10.)) >> make_bounded_sum((0., 10.)) >> make_base_laplace(scale=2.), 

        make_basic_composition([
            make_count(TIA=int, TO=int) >> make_base_geometric(scale=2.), 
            make_count(TIA=int, TO=float) >> make_base_laplace(scale=2.),
            (
                make_cast_default(int, str) >> 
                make_count_by_categories(categories=["0", "12", "22"]) >> 
                make_base_geometric(scale=2., D=VectorDomain[AllDomain[int]])
            )
        ])
    ])

    print("Check:", composed.check(1, 2.))
    print("Forward map:", composed.map(3))
    print("Invocation:", composed.invoke([22, 12]))


@pytest.mark.skip(reason="long-running process to detect potential memory leaks")
def test_make_basic_composition_leak():
    from opendp.comb import make_basic_composition

    # choose a vector-valued mechanism that should run quickly for large inputs
    # we want to add as little noise as possible, so that execution time is small
    meas = make_base_geometric(scale=1e-6, D=VectorDomain[AllDomain[int]])

    # memory usage remains the same when this line is commented,
    # supporting that AnyObject's free recursively frees children
    meas = make_basic_composition([meas])

    # watch for leaked AnyObjects with 10 million i32 values
    # memory would jump significantly every iteration
    for i in range(1000):
        print('iteration', i)
        meas([0] * 10_000_000)


def test_make_map_partitions():
    from opendp.comb import make_partition_map_trans, make_partition_map_meas

    meas = make_split_dataframe(
        separator=",", 
        col_names=["strat id", "values"]
    ) >> make_partition_by(
        identifier_column="strat id",
        partition_keys=list(map(str, range(4))),
        keep_columns=["values"],
    ) >> make_partition_map_trans([
        make_select_column("values", TOA=str) >> 
        make_cast_default(TIA=str, TOA=int) >> 
        make_clamp((0, 1))
    ] * 5) >> make_partition_map_meas([
        make_bounded_sum((0, 1)) >>
        make_base_geometric(1.)
    ] * 5)

    # build some synthetic data:
    from random import randint, choice
    data_length = 500
    strat_ids = [randint(0, 5) for _ in range(data_length)]
    values = [choice([0, 1]) for _ in range(data_length)]
    data = "\n".join(f"{k},{v}" for k, v in zip(strat_ids, values))
    # print(data)
    
    # release noisy sums!
    print(meas(data))


# 2-way partitioning!
def test_make_map_partitions_nested():
    from opendp.comb import make_partition_map_meas

    meas = make_split_dataframe(
        separator=",", 
        col_names=["strat id 1", "strat id 2", "values"]
    ) >> make_partition_by(
        identifier_column="strat id 1",
        partition_keys=list(map(str, range(4))),
        keep_columns=["strat id 2", "values"],
    ) >> make_partition_map_meas([
        make_partition_by(
            identifier_column="strat id 2",
            partition_keys=list(map(str, range(4))),
            keep_columns=["values"],
        ) >> make_partition_map_meas([
            make_select_column("values", TOA=str) >> 
            make_cast_default(TIA=str, TOA=int) >> 
            make_clamp((0, 1)) >>
            make_bounded_sum((0, 1)) >>
            make_base_geometric(1.)
        ] * 5)
    ] * 5)

    # build some synthetic data:
    from random import randint, choice
    data_length = 500
    strat_ids_1 = [randint(0, 5) for _ in range(data_length)]
    strat_ids_2 = [randint(0, 5) for _ in range(data_length)]
    values = [choice([0, 1]) for _ in range(data_length)]
    data = "\n".join(f"{k1},{k2},{v}" for k1, k2, v in zip(strat_ids_1, strat_ids_2, values))
    # print(data)
    
    # release noisy sums!
    print(meas(data))


def test_tm_chainer():
    chain = make_base_geometric(1., D=VectorDomain[AllDomain[int]]) >> make_cast_default(TIA=int, TOA=float)
    print(chain([1, 2, 3]))


if __name__ == "__main__":
    test_make_map_partitions_nested()

