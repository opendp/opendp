import opendp.prelude as dp

def test_space_of_typed():
    # metric defaults to symmetric_distance on vector_domain
    space = dp.space_of(list[int])
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.symmetric_distance())

    # can specify metric explicitly. If not fully specified, will infer distance type from domain
    space = dp.space_of(list[int], M=dp.L1Distance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.l1_distance(T=dp.i32))

    # when type is scalar, domain defaults to atom_domain and metric defaults to absolute_distance
    space = dp.space_of(float)
    assert space == (dp.atom_domain(T=dp.f64), dp.absolute_distance(T=dp.f64))

    space = dp.space_of(int, M=dp.DiscreteDistance)
    assert space == (dp.atom_domain(T=dp.f64), dp.discrete_distance())

    space = dp.space_of(int, M="DiscreteDistance")
    assert space == (dp.atom_domain(T=dp.f64), dp.discrete_distance())


def test_space_of_infer():
    space = dp.space_of([1, 3], infer=True)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.symmetric_distance())

    space = dp.space_of([1, 3], infer=True, M=dp.L1Distance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.l1_distance(T=dp.i32))

    space = dp.space_of(1.0, infer=True)
    assert space == (dp.atom_domain(T=dp.f64), dp.absolute_distance(T=dp.f64))

    space = dp.space_of(1, infer=True, M=dp.DiscreteDistance)
    assert space == (dp.atom_domain(T=dp.f64), dp.discrete_distance())

    space = dp.space_of(1, infer=True, M="DiscreteDistance")
    assert space == (dp.atom_domain(T=dp.f64), dp.discrete_distance())

    
test_space_of_typed()