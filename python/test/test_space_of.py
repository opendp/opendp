import opendp.prelude as dp

def test_typed_space_of():
    # metric defaults to symmetric_distance on vector_domain
    space = dp.space_of(list[int])
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.symmetric_distance())

    # can specify metric explicitly. If not fully specified, will infer distance type from domain
    space = dp.space_of(list[int], dp.L2Distance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.l2_distance(T=dp.i32))

    # not all metrics are parmeterized
    space = dp.space_of(list[int], dp.HammingDistance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.hamming_distance())
    space = dp.space_of(list[int], dp.InsertDeleteDistance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.insert_delete_distance())
    space = dp.space_of(list[int], dp.ChangeOneDistance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.change_one_distance())

    # when type is scalar, domain defaults to atom_domain and metric defaults to absolute_distance
    space = dp.space_of(float)
    assert space == (dp.atom_domain(T=dp.f64), dp.absolute_distance(T=dp.f64))

    # the variable from typing and a string are interchangeable
    space = dp.space_of(int, "DiscreteDistance")
    assert space == (dp.atom_domain(T=dp.i32), dp.discrete_distance())

    # can also pass an actual metric
    space = dp.space_of(int, dp.discrete_distance())
    assert space == (dp.atom_domain(T=dp.i32), dp.discrete_distance())

    space = dp.space_of(dict[str, int], dp.L1Distance[int])
    assert space == (dp.map_domain(dp.atom_domain(T=dp.String), dp.atom_domain(T=dp.i32)), dp.l1_distance(T=dp.i32))

def test_infer_space_of():
    space = dp.space_of([1, 3], infer=True)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.symmetric_distance())

    space = dp.space_of([1, 3], infer=True, M=dp.L1Distance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.l1_distance(T=dp.i32))

    space = dp.space_of(1.0, infer=True)
    assert space == (dp.atom_domain(T=dp.f64), dp.absolute_distance(T=dp.f64))

    space = dp.space_of(1, infer=True, M=dp.DiscreteDistance)
    assert space == (dp.atom_domain(T=dp.i32), dp.discrete_distance())

    space = dp.space_of(1, infer=True, M="DiscreteDistance")
    assert space == (dp.atom_domain(T=dp.i32), dp.discrete_distance())
