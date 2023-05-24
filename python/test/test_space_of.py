import opendp.prelude as dp


def test_space_of_infer():
    space = dp.space_of([1, 3], infer=True)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.symmetric_distance())


def test_space_of_typed():
    space = dp.space_of(list[int])
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.symmetric_distance())

    space = dp.space_of(list[int], M=dp.L1Distance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.l1_distance(T=dp.i32))

    space = dp.space_of(float)
    assert space == (dp.atom_domain(T=dp.f64), dp.absolute_distance(T=dp.f64))
